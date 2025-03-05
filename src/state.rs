use std::array::from_fn;
use crate::GoldilocksField;
use crate::Hash;
use crate::Transaction;
use anyhow::Result;
use plonky2::field::types::Field;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::config::Hasher;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Default, Clone)]
pub struct State {
    leaves: HashMap<(Hash, Hash), [GoldilocksField; 8]>,
    digests: HashMap<[u8; 66], Hash>,
}
impl State {
    pub fn contract_storage_digest(&self, addr: &Hash) -> Hash { self.get_digest(256, &Self::hash_to_index(addr, &Hash::ZERO)) }
    pub fn contract_storage_digest_path(&self, addr: &Hash) -> [Hash; 256] { self.proof(256, &Self::hash_to_index(addr, &Hash::ZERO)) }
    pub fn contract_storage_slot(&self, addr: &Hash, key: &Hash) -> [GoldilocksField; 8] { self.leaves.get(&(*addr, *key)).cloned().unwrap_or_default() }
    pub fn contract_storage_slot_path(&self, addr: &Hash, key: &Hash) -> [Hash; 256] { self.proof(512, &Self::hash_to_index(addr, key)) }
    pub fn root(&self) -> Hash { self.get_digest(0, &[0u8; 64]) }
    pub fn transit(&mut self, tx: Transaction) -> Result<()> {
        tx.vk.verify(self.root(), PoseidonHash::two_to_one(Hash::ZERO, tx.updates.iter().fold(Hash::ZERO, |left, (key, value)| PoseidonHash::hash_no_pad(&[&left.elements[..], &key.elements[..], &value[..]].concat()))), tx.proof)?;
        Ok(tx.updates.into_iter().for_each(|(key, value)| self.update(tx.vk.address(), key, value)))
    }
    fn update(&mut self, addr: Hash, key: Hash, value: [GoldilocksField; 8]) {
        let mut index = Self::hash_to_index(&addr, &key);
        self.digests.insert(Self::composite_key(512, &index), PoseidonHash::hash_no_pad(&value));
        self.leaves.insert((addr, key), value);
        for depth in (1..=512).rev() {
            let parent_index = Self::parent_index(depth, &index);
            let sibling_index = Self::get_sibling_index(depth, &index);
            let (left, right) = {
                let [current, sibling] = [&index, &sibling_index].map(|idx| self.get_digest(depth, idx));
                [(current, sibling), (sibling, current)][Self::is_right_child(depth, &index) as usize]
            };
            self.digests.insert(Self::composite_key(depth - 1, &parent_index), PoseidonHash::two_to_one(left, right));
            index = parent_index;
        }
    }
    fn get_digest(&self, depth: u16, index: &[u8; 64]) -> Hash {
        static DEFAULTS: LazyLock<[Hash; 513]> = LazyLock::new(|| {
            let mut defaults = [PoseidonHash::hash_no_pad(&[GoldilocksField::ZERO; 8]); 513];
            (0..512).rev().for_each(|i| defaults[i] = PoseidonHash::two_to_one(defaults[i + 1], defaults[i + 1]));
            defaults
        });
        self.digests.get(&Self::composite_key(depth, index)).cloned().unwrap_or(DEFAULTS[depth as usize])
    }
    pub fn proof(&self, depth: u16, index: &[u8; 64]) -> [Hash; 256] {
        let mut current_index = *index;
        from_fn(|i| {
            let d = depth - i as u16;
            let hash = self.get_digest(d, &Self::get_sibling_index(d, &current_index));
            current_index = Self::parent_index(d, &current_index);
            hash
        })
    }
    fn hash_to_index(hash1: &Hash, hash2: &Hash) -> [u8; 64] {
        let mut result = [0u8; 64];
        for i in 0..4 {
            let element1 = hash1.elements[3 - i];
            let element2 = hash2.elements[3 - i];
            result[8*i..8*(i+1)].copy_from_slice(&element1.0.to_be_bytes());
            result[32 + 8*i..32 + 8*(i+1)].copy_from_slice(&element2.0.to_be_bytes());
        }
        result
    }
    fn is_right_child(depth: u16, index: &[u8; 64]) -> bool {
        Self::get_bit(index, depth)
    }
    fn parent_index(depth: u16, index: &[u8; 64]) -> [u8; 64] {
        let mut parent = *index;
        Self::clear_bit(&mut parent, depth);
        parent
    }
    fn get_sibling_index(depth: u16, index: &[u8; 64]) -> [u8; 64] {
        let mut sibling = *index;
        Self::flip_bit(&mut sibling, depth);
        sibling
    }
    fn get_bit(index: &[u8; 64], depth: u16) -> bool {
        let mask = &PATH_MASKS[depth as usize - 1];
        (index[mask.byte_pos] & mask.bit_mask) != 0
    }
    fn clear_bit(index: &mut [u8; 64], depth: u16) {
        let mask = &PATH_MASKS[depth as usize - 1];
        index[mask.byte_pos] &= !mask.bit_mask;
    }
    fn flip_bit(index: &mut [u8; 64], depth: u16) {
        let mask = &PATH_MASKS[depth as usize - 1];
        index[mask.byte_pos] ^= mask.bit_mask;
    }
    fn composite_key(depth: u16, index: &[u8; 64]) -> [u8; 66] {
        let mut key = [0u8; 66];
        key[0..2].copy_from_slice(&depth.to_le_bytes());
        key[2..].copy_from_slice(index);
        key
    }
}


#[derive(Copy, Clone)]
pub struct PathMask {
    pub byte_pos: usize,
    pub bit_mask: u8,
}

pub const PATH_MASKS: [PathMask; 512] = {
    let mut masks = [PathMask { byte_pos: 0, bit_mask: 0 }; 512];
    let mut depth = 0;
    while depth < 512 {
        let bit_pos = depth;
        masks[depth as usize] = PathMask {
            byte_pos: (bit_pos / 8) as usize,
            bit_mask: 0x01 << (7 - (bit_pos % 8)),
        };
        depth += 1;
    }
    masks
};
