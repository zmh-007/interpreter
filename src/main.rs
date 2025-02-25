use interpreter::Circuit;
use interpreter::Field64;
use interpreter::Interpreter;
use interpreter::MerkleProofTarget;
use interpreter::PoseidonHash;
use interpreter::Transaction;
use interpreter::WitnessWrite;

fn main() {
    let c = Circuit::new(|builder| {
        let this = builder.add_virtual_hash_public_input();
        let root = builder.add_virtual_hash_public_input();
        let new = builder.add_virtual_public_input();
        let old = builder.add_virtual_target();
        let path = MerkleProofTarget { siblings: builder.add_virtual_hashes(256) };
        let index = root.elements.map(|v| builder.split_le(v, 64)).concat();
        let one = builder.sub(new, old);
        builder.verify_merkle_proof::<PoseidonHash>(vec![old], &index, root, &path);
        builder.assert_one(one);
        (this, root, path, old, new)
    });
    let vk = c.vk();
    let mut s = Interpreter::new();
    for i in 0..16 {
        let (old, path) = s.prove(vk.address());
        let new = old.add_one();
        if let Ok((proof, _)) = c.prove(|w, t| {
            w.set_hash_target(t.0, vk.address())?;
            w.set_hash_target(t.1, s.root())?;
            (0..256).try_for_each(|i| w.set_hash_target(t.2.siblings[i], path[i]))?;
            w.set_target(t.3, old)?;
            w.set_target(t.4, new)
        }) {
            let tx = Transaction { new, proof, vk: vk.clone() };
            eprintln!("transaction[{i}]: {:?}", s.transit(tx));
        } else {
            panic!("PROVING FAILURE");
        }
    }
}
