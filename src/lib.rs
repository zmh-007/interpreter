mod transaction;
mod zk;

pub use crate::transaction::Transaction;
pub use crate::zk::Circuit;
pub use crate::zk::Field;
pub use crate::zk::Field64;
pub use crate::zk::GoldilocksField;
use crate::zk::VerifyingKey;
pub use crate::zk::WitnessWrite;
use anyhow::Result;
use std::collections::HashMap;

pub struct Interpreter {
    s: HashMap<Vec<u8>, GoldilocksField>,
}

impl Interpreter {
    pub fn new() -> Self { Self { s: HashMap::new() } }
    pub fn get(&self, vk: &VerifyingKey) -> GoldilocksField { self.s.get(&vk.to_bytes()).cloned().unwrap_or_default() }
    pub fn transit(&mut self, tx: Transaction) -> Result<()> {
        let entry = self.s.entry(tx.vk.to_bytes()).or_default();
        tx.vk.verify(vec![*entry, tx.new], tx.proof)?;
        Ok(*entry = tx.new)
    }
}
