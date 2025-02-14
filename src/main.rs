use interpreter::Interpreter;
use interpreter::Transaction;
use rand::random;

fn main() {
    let mut s = Interpreter::new();
    for _ in 0..16 {
        let tx = Transaction { data: random() };
        s.transit(tx);
    }
}
