use interpreter::Circuit;
use interpreter::Field64;
use interpreter::Interpreter;
use interpreter::PoseidonHash;
use interpreter::Transaction;
use interpreter::WitnessWrite;
use plonky2::hash::hash_types::HashOutTarget;

fn main() {
    let c = Circuit::new(|builder| {
        let this = builder.add_virtual_hash_public_input();
        let root = builder.add_virtual_hash_public_input();
        let path: [HashOutTarget; 256] = builder.add_virtual_hashes(256).try_into().unwrap();
        let new = builder.add_virtual_public_input();
        let old = builder.add_virtual_target();
        let index_bits = this.elements.map(|v| 
            builder.split_le(v, 64)).concat();
        let one = builder.sub(new, old);

        let leaf_hash = builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![old]); // 生成叶子哈希约束
        let mut current_hash = leaf_hash;
        let mut current_index = index_bits.clone();

        for (i, sibling) in path.iter().enumerate() {
            let depth = 256 - i as u16 - 1;
            let bit_pos = 255 - depth as usize;
            let bit = index_bits[bit_pos];
        
            let left = HashOutTarget {
                elements: core::array::from_fn(|i| builder.select(bit, sibling.elements[i], current_hash.elements[i])),
            };
            let right = HashOutTarget {
                elements: core::array::from_fn(|i| builder.select(bit, current_hash.elements[i], sibling.elements[i])),
            };

            current_hash = builder.hash_n_to_hash_no_pad::<PoseidonHash>
            (vec![left.elements.to_vec(), right.elements.to_vec()].concat());
            
            current_index[bit_pos] = builder.constant_bool(false);
        }
    
        builder.connect_hashes(root, current_hash);
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
            (0..256).try_for_each(|i| w.set_hash_target(t.2[i], path[i]))?;
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
