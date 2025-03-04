use criterion::{black_box, criterion_group, criterion_main, Criterion};
use plonky2::{field::{goldilocks_field::GoldilocksField, types::Field}, plonk::config::Hasher};
pub type Hash = plonky2::hash::hash_types::HashOut<GoldilocksField>;
use rand::{rng, Rng};
use std::collections::HashMap;
use plonky2::hash::poseidon::PoseidonHash;

fn random_hash<R: Rng>(rng: &mut R) -> Hash {
    Hash {
        elements: std::array::from_fn(|_| GoldilocksField::from_canonical_u64(rng.random())),
    }
}

fn random_value<R: Rng>(rng: &mut R) -> [GoldilocksField; 8] {
    std::array::from_fn(|_| GoldilocksField::from_canonical_u64(rng.random()))
}

fn benchmark(c: &mut Criterion) {
    let mut rng = rng();
    let data_size = 100_000;

    let pairs: Vec<(Hash, Hash)> = (0..data_size)
        .map(|_| (random_hash(&mut rng), random_hash(&mut rng)))
        .collect();
    
    let values: Vec<[GoldilocksField; 8]> = (0..data_size)
        .map(|_| random_value(&mut rng))
        .collect();

    c.bench_function("tuple_key", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for ((k1, k2), v) in pairs.iter().zip(values.iter()) {
                map.insert(black_box((k1.clone(), k2.clone())), black_box(*v));
            }
            black_box(map)
        });
    });

    c.bench_function("hash_keys", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for ((k1, _), v) in pairs.iter().zip(values.iter()) {
                map.insert(black_box(k1), black_box(*v));
            }
            black_box(map)
        });
    });

    c.bench_function("poseidon_merge_keys", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for ((k1, k2), v) in pairs.iter().zip(values.iter()) {
                let merged_key = PoseidonHash::two_to_one(k1.clone(), k2.clone());
                map.insert(black_box(merged_key), black_box(*v));
            }
            black_box(map)
        });
    });

    let mut tuple_map = HashMap::new();
    let mut single_map = HashMap::new();
    let mut merged_map = HashMap::new();
    for ((k1, k2), v) in pairs.iter().zip(values.iter()) {
        tuple_map.insert((k1.clone(), k2.clone()), *v);
        single_map.insert(k1.clone(), v);
        let merged_key = PoseidonHash::two_to_one(k1.clone(), k2.clone());
        merged_map.insert(merged_key.clone(), *v);
    }

    c.bench_function("tuple_key_lookup", |b| {
        b.iter(|| {
            for (k1, k2) in &pairs {
                black_box(tuple_map.get(&black_box((k1.clone(), k2.clone()))));
            }
        });
    });

    c.bench_function("single_key_lookup", |b| {
        b.iter(|| {
            for ((k1, _), _) in pairs.iter().zip(values.iter()) {
                black_box(single_map.get(&black_box(k1.clone())));
            }
        });
    });

    c.bench_function("merged_key_lookup", |b| {
        b.iter(|| {
            for ((k1, k2), _) in pairs.iter().zip(values.iter()) {
                let merged_key = PoseidonHash::two_to_one(k1.clone(), k2.clone());
                black_box(single_map.get(&black_box(merged_key)));
            }
        });
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
