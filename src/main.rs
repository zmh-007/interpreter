use interpreter::Circuit;
use interpreter::Field64;
use interpreter::Interpreter;
use interpreter::Transaction;
use interpreter::WitnessWrite;

fn main() {
    let c = Circuit::new(|builder| {
        let (old, new) = (builder.add_virtual_public_input(), builder.add_virtual_public_input());
        let sub = builder.sub(new, old);
        builder.assert_one(sub);
        [old, new]
    });
    let vk = c.vk();
    let mut s = Interpreter::new();
    for i in 0..16 {
        let old = s.get(&vk);
        let new = old.add_one();
        let proof = c.prove(|w, t| w.set_target_arr(t, &[old, new])).unwrap().0;
        let tx = Transaction { new, proof, vk: vk.clone() };
        eprintln!("transaction[{i}]: {:?}", s.transit(tx));
    }
}
