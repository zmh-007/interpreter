use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};
use plonky2::field::{goldilocks_field::GoldilocksField, types::Field};
pub type Hash = plonky2::hash::hash_types::HashOut<GoldilocksField>;
use rand::{rng, Rng};
use std::collections::HashMap;
use bitvec::prelude::*;

fn generate_bits<R: Rng>(rng: &mut R, len: usize) -> (Vec<bool>, BitVec<u8>, (u16, Vec<u8>), (u16, [u8; 64])) {
    let vec_bool: Vec<bool> = (0..len).map(|_| rng.random()).collect();
    let mut bitvec = BitVec::<u8, Lsb0>::new();
    for b in &vec_bool {
        bitvec.push(*b);
    }

    let mut vec_u8 = Vec::new();
    let mut byte = 0u8;
    for (i, &bit) in vec_bool.iter().enumerate() {
        if bit {
            byte |= 1 << (7 - (i % 8));
        }
        if (i + 1) % 8 == 0 || i == vec_bool.len() - 1 {
            vec_u8.push(byte);
            byte = 0;
        }
    }

    let mut u8_array = [0u8; 64];
    let mut byte = 0u8;
    for (i, &bit) in vec_bool.iter().enumerate() {
        if bit {
            byte |= 1 << (7 - (i % 8));
        }
        if (i + 1) % 8 == 0 || i == vec_bool.len() - 1 {
            u8_array[i / 8] = byte;
            byte = 0;
        }
    }

    (vec_bool, bitvec, (len as u16, vec_u8), (len as u16, u8_array))
}

fn generate_value<R: Rng>(rng: &mut R) -> [GoldilocksField; 8] {
    std::array::from_fn(|_| GoldilocksField::from_canonical_u64(rng.random()))
}

fn composite_key1(depth: u16, index: Vec<u8>) -> Vec<u8> {
    let mut key = vec![0; 2 + index.len()];
    key[0..2].copy_from_slice(&depth.to_le_bytes());
    key[2..].copy_from_slice(&index);
    key
}

fn composite_key2(depth: u16, index: [u8; 64]) -> [u8; 66] {
    let mut key: [u8; 66] = [0u8; 66];
    key[0..2].copy_from_slice(&depth.to_le_bytes());
    key[2..].copy_from_slice(&index);
    key
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("vecbit_vs_vecbool");
    group.sampling_mode(SamplingMode::Flat);
    
    let mut rng = rng();
    const SAMPLES_PER_SIZE: usize = 10_000;

    for key_len in [1, 6, 32, 66, 128, 200, 279, 512] {
        let mut test_data: Vec<(Vec<bool>, BitVec<u8>, (u16, Vec<u8>), (u16, [u8; 64]), [GoldilocksField; 8])> = Vec::with_capacity(SAMPLES_PER_SIZE);
        for _ in 0..SAMPLES_PER_SIZE {
            let (vec_bool_key, bitvec_key, vec_u8_key, u8_array_key) = generate_bits(&mut rng, key_len);
            let value = generate_value(&mut rng);
            test_data.push((vec_bool_key, bitvec_key, vec_u8_key, u8_array_key, value));
        }

        group.bench_function(format!("vecbool_insert/{}", key_len), |b| {
            b.iter(|| {
                let mut map = HashMap::new();
                for (vec_bool_key, _, _, _, value) in &test_data {
                    map.insert(black_box(vec_bool_key.clone()), black_box(*value));
                }
                black_box(map);
            });
        });

        group.bench_function(format!("bitvec_insert/{}", key_len), |b| {
            b.iter(|| {
                let mut map = HashMap::new();
                for (_, bitvec_key, _, _, value) in &test_data {
                    map.insert(black_box(bitvec_key.clone()), black_box(*value));
                }
                black_box(map);
            });
        });

        group.bench_function(format!("vecu8_insert/{}", key_len), |b| {
            b.iter(|| {
                let mut map = HashMap::new();
                for (_, _, vec_u8_key, _, value) in &test_data {
                    map.insert(black_box(composite_key1(vec_u8_key.0, vec_u8_key.1.clone())), black_box(*value));
                }
                black_box(map);
            });
        });

        group.bench_function(format!("u8array_insert/{}", key_len), |b| {
            b.iter(|| {
                let mut map = HashMap::new();
                for (_, _, _, u8_array_key, value) in &test_data {
                    map.insert(black_box(composite_key2(u8_array_key.0, u8_array_key.1.clone())), black_box(*value));
                }
                black_box(map);
            });
        });  

        let mut vecbool_map = HashMap::new();
        let mut bitvec_map = HashMap::new();
        let mut vecu8_map = HashMap::new();
        let mut u8array_map = HashMap::new();
        for (vec_bool_key, bitvec_key, vec_u8_key, u8_array_key, value) in &test_data {
            vecbool_map.insert(vec_bool_key.clone(), *value);
            bitvec_map.insert(bitvec_key.clone(), *value);
            vecu8_map.insert(composite_key1(vec_u8_key.0, vec_u8_key.1.clone()), *value);
            u8array_map.insert(composite_key2(u8_array_key.0, u8_array_key.1.clone()), *value);
        }

        group.bench_function(format!("vecbool_lookup/{}", key_len), |b| {
            b.iter(|| {
                for (vec_bool_key, _, _, _, _) in &test_data {
                    black_box(vecbool_map.get(black_box(vec_bool_key)));
                }
            });
        });

        group.bench_function(format!("bitvec_lookup/{}", key_len), |b| {
            b.iter(|| {
                for (_, bitvec_key, _, _, _) in &test_data {
                    black_box(bitvec_map.get(black_box(bitvec_key)));
                }
            });
        });

        group.bench_function(format!("vecu8_lookup/{}", key_len), |b| {
            b.iter(|| {
                for (_, _, vec_u8_key, _, _) in &test_data {
                    black_box(vecu8_map.get(black_box(&composite_key1(vec_u8_key.0, vec_u8_key.1.clone()))));
                }
            });
        });

        group.bench_function(format!("u8array_lookup/{}", key_len), |b| {
            b.iter(|| {
                for (_, _, _, u8_array_key, _) in &test_data {
                    black_box(u8array_map.get(black_box(&composite_key2(u8_array_key.0, u8_array_key.1.clone()))));
                }
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
