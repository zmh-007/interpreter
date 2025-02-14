mod transaction;
pub use transaction::Transaction;

pub struct Interpreter {
    s: u8,
}

impl Interpreter {
    pub fn new() -> Self { Self { s: 0 } }
    pub fn transit(&mut self, tx: Transaction) {
        if tx.data {
            self.s += 1
        }
    }
}
