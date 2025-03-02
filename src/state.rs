use crate::GoldilocksField;
use crate::Hash;
use crate::Transaction;
use anyhow::Result;
use plonky2::field::types::Field;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::config::Hasher;
use std::array::from_fn;
use std::collections::HashMap;
use std::iter::once;
use std::sync::LazyLock;
#[derive(Debug, Default, Clone)]
pub struct State {
    leaves: HashMap<(Hash, Hash), [GoldilocksField; 8]>,
    digests: HashMap<Vec<bool>, Hash>,
}
impl State {
    pub fn contract_storage_digest(&self, addr: &Hash) -> Hash { self.get_digest(&Self::hash_to_index(addr)) }
    pub fn contract_storage_digest_path(&self, addr: &Hash) -> [Hash; 256] { self.proof_by_index(&[], &Self::hash_to_index(addr)) }
    pub fn contract_storage_slot(&self, addr: &Hash, key: &Hash) -> [GoldilocksField; 8] { self.leaves.get(&(*addr, *key)).cloned().unwrap_or_default() }
    pub fn contract_storage_slot_path(&self, addr: &Hash, key: &Hash) -> [Hash; 256] { self.proof_by_index(&Self::hash_to_index(addr), &Self::hash_to_index(key)) }
    pub fn root(&self) -> Hash { self.get_digest(&vec![]) }
    pub fn transit(&mut self, tx: Transaction) -> Result<()> {
        tx.vk.verify(self.root(), PoseidonHash::two_to_one(Hash::ZERO, tx.updates.iter().fold(Hash::ZERO, |left, (key, value)| PoseidonHash::hash_no_pad(&[&left.elements[..], &key.elements[..], &value[..]].concat()))), tx.proof)?;
        Ok(tx.updates.into_iter().for_each(|(key, value)| self.update(tx.vk.address(), key, value)))
    }
    fn update(&mut self, addr: Hash, key: Hash, value: [GoldilocksField; 8]) {
        let mut index = [&addr, &key].map(Self::hash_to_index).concat();
        self.digests.insert(index.clone(), PoseidonHash::hash_no_pad(&value));
        self.leaves.insert((addr, key), value);
        for _ in 0..512 {
            index.pop();
            let [left, right] = [false, true].map(|v| self.get_digest(&index.iter().cloned().chain(once(v)).collect::<Vec<_>>()));
            self.digests.insert(index.clone(), PoseidonHash::two_to_one(left, right));
        }
    }
    fn get_digest(&self, index: &[bool]) -> Hash {
        static DEFAULTS: LazyLock<[Hash; 513]> = LazyLock::new(|| {
            let mut defaults = [PoseidonHash::hash_no_pad(&[GoldilocksField::ZERO; 8]); 513];
            (0..512).rev().for_each(|i| defaults[i] = PoseidonHash::two_to_one(defaults[i + 1], defaults[i + 1]));
            defaults
        });
        self.digests.get(index).cloned().unwrap_or(DEFAULTS[index.len()])
    }
    fn proof_by_index(&self, prefix: &[bool], index: &[bool; 256]) -> [Hash; 256] { from_fn(|i| self.get_digest(&prefix.iter().cloned().chain(index[0..255 - i].iter().cloned()).chain(once(!index[255 - i])).collect::<Vec<_>>())) }
    fn hash_to_index(hash: &Hash) -> [bool; 256] { from_fn(|i| hash.elements[3 - i / 64].0 >> (63 - i % 64) & 1 > 0) }
}
