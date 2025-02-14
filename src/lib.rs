mod transaction;
pub use transaction::Transaction;

pub struct Interpreter {
    s: [u8; 256],
}

impl Interpreter {
    pub fn new() -> Self { Self { s: [0; 256] } }
    pub fn transit(&mut self, tx: Transaction) { self.s[tx.data as usize] += 1; }
}
