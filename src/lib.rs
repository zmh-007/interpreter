mod transaction;
mod zk;
mod smt;

pub use crate::transaction::Transaction;
pub use crate::zk::Circuit;
pub use crate::zk::Field;
pub use crate::zk::Field64;
pub use crate::zk::GoldilocksField;
pub use crate::zk::Hash;
pub use crate::zk::MerkleProofTarget;
pub use crate::zk::PoseidonHash;
pub use crate::zk::WitnessWrite;
pub use crate::smt::SparseMerkleTree;
use anyhow::Result;
use plonky2::plonk::config::GenericHashOut;
pub struct Interpreter {
    tree: SparseMerkleTree,
}

impl Interpreter {
    pub fn new() -> Self {
        let tree = SparseMerkleTree::new();
        Self { tree }
    }
    pub fn prove(&self, addr: Hash) -> (GoldilocksField, [Hash; 256]) { self.tree.prove(&addr.to_bytes().try_into().unwrap()) }
    pub fn root(&self) -> Hash { self.tree.root() }
    fn insert(&mut self, addr: Hash, value: GoldilocksField) { self.tree.insert(addr.to_bytes().try_into().unwrap(), value); }
    pub fn transit(&mut self, tx: Transaction) -> Result<()> {
        tx.vk.verify(self.root(), tx.new, tx.proof)?;
        Ok(self.insert(tx.vk.address(), tx.new))
    }
}
