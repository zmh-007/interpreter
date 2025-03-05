use std::array::from_fn;
use std::time::Instant;
use interpreter::Hash;
use interpreter::State;
use interpreter::Transaction;
use interpreter::VerificationKey;
use plonky2::field::types::Field64;
use plonky2::hash::merkle_proofs::MerkleProofTarget;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::config::Hasher;
fn main() {
    let (vk, pf) = VerificationKey::setup::<true>(|builder, [this, root, mesg]| {
        let hash_zero = builder.constant_hash(Hash::ZERO);
        let contract_storage_digest_index = this.elements.iter().flat_map(|&v| builder.split_le(v, 64)).collect::<Vec<_>>();
        let contract_storage_slot_index = hash_zero.elements.iter().flat_map(|&v| builder.split_le(v, 64)).collect::<Vec<_>>();
        let contract_storage_digest = builder.add_virtual_hash();
        let contract_storage_digest_path = MerkleProofTarget { siblings: builder.add_virtual_hashes(256) };
        let contract_storage_slot = builder.add_virtual_target_arr::<8>();
        let contract_storage_slot_path = MerkleProofTarget { siblings: builder.add_virtual_hashes(256) };
        let contract_storage_slot_new = builder.add_virtual_target_arr::<8>();
        let update_digest = vec![(hash_zero, contract_storage_slot_new)].iter().fold(hash_zero, |left, (key, value)| builder.hash_n_to_hash_no_pad::<PoseidonHash>([&left.elements[..], &key.elements[..], &value[..]].concat()));
        let message = builder.hash_n_to_hash_no_pad::<PoseidonHash>([hash_zero.elements, update_digest.elements].concat());
        let one = builder.sub(contract_storage_slot_new[0], contract_storage_slot[0]);
        builder.assert_one(one);
        (1..8).for_each(|i| builder.assert_zero(contract_storage_slot[i]));
        (1..8).for_each(|i| builder.assert_zero(contract_storage_slot_new[i]));
        builder.verify_merkle_proof::<PoseidonHash>(contract_storage_digest.elements.into(), &contract_storage_digest_index, root, &contract_storage_digest_path);
        builder.verify_merkle_proof::<PoseidonHash>(contract_storage_slot.into(), &contract_storage_slot_index, contract_storage_digest, &contract_storage_slot_path);
        builder.connect_hashes(mesg, message);
        let targets = [];
        let slots = [contract_storage_slot, contract_storage_slot_new];
        let hashes = [contract_storage_digest];
        let paths = [contract_storage_digest_path, contract_storage_slot_path];
        paths.into_iter().flat_map(|v| v.siblings).chain(hashes.into_iter()).flat_map(|v| v.elements).chain(slots.into_iter().flatten()).chain(targets).collect()
    });
    let addr = vk.address();
    let key = Hash::ZERO;
    let mut s = State::default();
    let start = Instant::now();
    for i in 0..16 {
        eprintln!("GENERATING TX: {i} ...");
        let contract_storage_digest = s.contract_storage_digest(&addr);
        eprintln!("contract_storage_digest: {contract_storage_digest:?}");
        let contract_storage_digest_path = s.contract_storage_digest_path(&addr);
        let contract_storage_slot = s.contract_storage_slot(&addr, &key);
        eprintln!("contract_storage_slot: {contract_storage_slot:?}");
        let contract_storage_slot_path = s.contract_storage_slot_path(&addr, &key);
        let contract_storage_slot_new = from_fn(|i| if i == 0 { contract_storage_slot[i].add_one() } else { contract_storage_slot[i] });
        eprintln!("contract_storage_slot_new: {contract_storage_slot_new:?}");
        let targets = [];
        let slots = [contract_storage_slot, contract_storage_slot_new];
        let hashes = [contract_storage_digest];
        let paths = [contract_storage_digest_path, contract_storage_slot_path];
        let witnesses = paths.into_iter().flatten().chain(hashes.into_iter()).flat_map(|v| v.elements).chain(slots.into_iter().flatten()).chain(targets).collect();
        let updates = vec![(Hash::ZERO, contract_storage_slot_new)];
        let mesg = PoseidonHash::two_to_one(Hash::ZERO, updates.iter().fold(Hash::ZERO, |left, (key, value)| PoseidonHash::hash_no_pad(&[&left.elements[..], &key.elements[..], &value[..]].concat())));
        if let Ok(proof) = pf([addr, s.root(), mesg], witnesses) {
            let tx = Transaction { vk: vk.clone(), proof, updates };
            eprintln!("tx: {:?} : {:?}", tx.clone(), s.transit(tx));
        } else {
            panic!("PROVING FAILURE");
        }
    }
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
}
