mod transaction;
mod zk;

pub use crate::transaction::Transaction;
pub use crate::zk::Circuit;
pub use crate::zk::Field;
pub use crate::zk::Field64;
pub use crate::zk::GoldilocksField;
pub use crate::zk::Hash;
pub use crate::zk::MerkleProofTarget;
pub use crate::zk::PoseidonHash;
pub use crate::zk::WitnessWrite;
use anyhow::Result;
pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self { todo!() }
    pub fn prove(&self, addr: Hash) -> (GoldilocksField, [Hash; 256]) { todo!() }
    pub fn root(&self) -> Hash { todo!() }
    pub fn transit(&mut self, tx: Transaction) -> Result<()> {
        tx.vk.verify(self.root(), tx.new, tx.proof)?;
        Ok(self.insert(tx.vk.address(), tx.new))
    }
    fn insert(&self, addr: Hash, value: GoldilocksField) { todo!() }
}
