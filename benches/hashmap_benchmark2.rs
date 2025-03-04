use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};
use plonky2::field::{goldilocks_field::GoldilocksField, types::Field};
pub type Hash = plonky2::hash::hash_types::HashOut<GoldilocksField>;
use rand::{rng, Rng};
use std::collections::HashMap;
use bitvec::prelude::*;

fn generate_bits<R: Rng>(rng: &mut R, len: usize) -> (Vec<bool>, BitVec) {
    let vec_bool: Vec<bool> = (0..len).map(|_| rng.random()).collect();
    let mut bitvec = BitVec::<_, Lsb0>::new();
    for b in &vec_bool {
        bitvec.push(*b);
    }
    (vec_bool, bitvec)
}

fn generate_value<R: Rng>(rng: &mut R) -> [GoldilocksField; 8] {
    std::array::from_fn(|_| GoldilocksField::from_canonical_u64(rng.random()))
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("vecbit_vs_vecbool");
    group.sampling_mode(SamplingMode::Flat);
    
    let mut rng = rng();
    const SAMPLES_PER_SIZE: usize = 10_000;

    for key_len in [1, 8, 32, 64, 128, 256, 512] {
        let mut test_data = Vec::with_capacity(SAMPLES_PER_SIZE);
        for _ in 0..SAMPLES_PER_SIZE {
            let (vec_key, bitvec_key) = generate_bits(&mut rng, key_len);
            let value = generate_value(&mut rng);
            test_data.push((vec_key, bitvec_key, value));
        }

        group.bench_function(format!("vecbool_insert/{}", key_len), |b| {
            b.iter(|| {
                let mut map = HashMap::new();
                for (vec_key, _, value) in &test_data {
                    map.insert(black_box(vec_key.clone()), black_box(*value));
                }
                black_box(map);
            });
        });

        group.bench_function(format!("bitvec_insert/{}", key_len), |b| {
            b.iter(|| {
                let mut map = HashMap::new();
                for (_, bitvec_key, value) in &test_data {
                    map.insert(black_box(bitvec_key.clone()), black_box(*value));
                }
                black_box(map);
            });
        });

        let mut vec_map = HashMap::new();
        let mut bitvec_map = HashMap::new();
        for (vec_key, bitvec_key, value) in &test_data {
            vec_map.insert(vec_key.clone(), *value);
            bitvec_map.insert(bitvec_key.clone(), *value);
        }

        group.bench_function(format!("vecbool_lookup/{}", key_len), |b| {
            b.iter(|| {
                for (vec_key, _, _) in &test_data {
                    black_box(vec_map.get(black_box(vec_key)));
                }
            });
        });

        group.bench_function(format!("bitvec_lookup/{}", key_len), |b| {
            b.iter(|| {
                for (_, bitvec_key, _) in &test_data {
                    black_box(bitvec_map.get(black_box(bitvec_key)));
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
