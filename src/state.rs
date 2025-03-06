use crate::GoldilocksField;
use crate::Hash;
use crate::Transaction;
use anyhow::Result;
use plonky2::field::types::Field;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::config::Hasher;
use std::array::from_fn;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct StateIndex {
    depth: u16,
    index: Vec<u8>,
}

#[derive(Debug, Default, Clone)]
pub struct State {
    leaves: HashMap<(Hash, Hash), [GoldilocksField; 8]>,
    digests: HashMap<StateIndex, Hash>,
}
impl State {
    pub fn contract_storage_digest(&self, addr: &Hash) -> Hash {
        self.get_digest(&StateIndex {
            depth: 256,
            index: Self::hash_to_index(addr),
        })
    }
    pub fn contract_storage_digest_path(&self, addr: &Hash) -> [Hash; 256] {
        self.proof(&StateIndex {
            depth: 256,
            index: Self::hash_to_index(addr),
        })
    }
    pub fn contract_storage_slot(&self, addr: &Hash, key: &Hash) -> [GoldilocksField; 8] {
        self.leaves.get(&(*addr, *key)).cloned().unwrap_or_default()
    }
    pub fn contract_storage_slot_path(&self, addr: &Hash, key: &Hash) -> [Hash; 256] {
        self.proof(&StateIndex {
            depth: 512,
            index: Self::hash_to_index(&addr)
                .iter()
                .chain(Self::hash_to_index(&key).iter())
                .cloned()
                .collect(),
        })
    }
    pub fn root(&self) -> Hash {
        self.get_digest(&StateIndex {
            depth: 0,
            index: vec![],
        })
    }
    pub fn transit(&mut self, tx: Transaction) -> Result<()> {
        tx.vk.verify(
            self.root(),
            PoseidonHash::two_to_one(
                Hash::ZERO,
                tx.updates.iter().fold(Hash::ZERO, |left, (key, value)| {
                    PoseidonHash::hash_no_pad(
                        &[&left.elements[..], &key.elements[..], &value[..]].concat(),
                    )
                }),
            ),
            tx.proof,
        )?;
        Ok(tx
            .updates
            .into_iter()
            .for_each(|(key, value)| self.update(tx.vk.address(), key, value)))
    }
    fn update(&mut self, addr: Hash, key: Hash, value: [GoldilocksField; 8]) {
        let mut index = StateIndex {
            depth: 512,
            index: Self::hash_to_index(&addr)
                .iter()
                .chain(Self::hash_to_index(&key).iter())
                .cloned()
                .collect(),
        };
        self.digests
            .insert(index.clone(), PoseidonHash::hash_no_pad(&value));
        self.leaves.insert((addr, key), value);
        for _ in 0..512 {
            let sibling_index = Self::get_sibling_index(&index);
            let [current, sibling] = [&index, &sibling_index].map(|idx| self.get_digest(&idx));
            let (left, right) = if Self::is_right_child(&index) {
                (sibling, current)
            } else {
                (current, sibling)
            };
            Self::parent_index(&mut index);
            self.digests
                .insert(index.clone(), PoseidonHash::two_to_one(left, right));
        }
    }
    fn get_digest(&self, state_index: &StateIndex) -> Hash {
        static DEFAULTS: LazyLock<[Hash; 513]> = LazyLock::new(|| {
            let mut defaults = [PoseidonHash::hash_no_pad(&[GoldilocksField::ZERO; 8]); 513];
            (0..512).rev().for_each(|i| {
                defaults[i] = PoseidonHash::two_to_one(defaults[i + 1], defaults[i + 1])
            });
            defaults
        });
        self.digests
            .get(state_index)
            .cloned()
            .unwrap_or(DEFAULTS[state_index.depth as usize])
    }
    fn proof(&self, state_index: &StateIndex) -> [Hash; 256] {
        let mut current_index = state_index.clone();
        from_fn(|_| {
            let hash = self.get_digest(&Self::get_sibling_index(&current_index));
            Self::parent_index(&mut current_index);
            hash
        })
    }
    fn hash_to_index(hash: &Hash) -> Vec<u8> {
        hash.elements
            .iter()
            .rev()
            .flat_map(|element| element.0.to_be_bytes())
            .collect()
    }
    fn is_right_child(state_index: &StateIndex) -> bool {
        (state_index.index[state_index.index.len() - 1]
            & PATH_MASKS[512 - state_index.depth as usize])
            != 0
    }
    fn parent_index(state_index: &mut StateIndex) {
        let mask = PATH_MASKS[512 - state_index.depth as usize];
        *state_index.index.last_mut().unwrap() &= !mask;
        if mask == 0x80 {
            state_index.index.pop();
        }
        state_index.depth -= 1;
    }
    fn get_sibling_index(state_index: &StateIndex) -> StateIndex {
        let mut sibling_index = state_index.clone();
        *sibling_index.index.last_mut().unwrap() ^= PATH_MASKS[512 - state_index.depth as usize];
        sibling_index
    }
}

pub const PATH_MASKS: [u8; 512] = {
    let mut masks = [0; 512];
    let mut depth = 0;
    while depth < 512 {
        let bit_pos = depth;
        masks[depth as usize] = 0x01 << (bit_pos % 8);
        depth += 1;
    }
    masks
};
