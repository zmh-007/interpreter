use plonky2::{
    field::goldilocks_field::GoldilocksField,
    hash::{
        hash_types::HashOut,
        poseidon::PoseidonHash,
    }, plonk::config::Hasher,
};
use std::collections::HashMap;
use plonky2::field::types::Field;

type F = GoldilocksField;
type Index256 = [u8; 32];
type Key = [u8; 34];
pub const TREE_DEPTH: u16 = 256;

#[derive(Copy, Clone)]
pub struct PathMask {
    pub byte_pos: usize,
    pub bit_mask: u8,
}

pub const PATH_MASKS: [PathMask; TREE_DEPTH as usize] = {
    let mut masks = [PathMask { byte_pos: 0, bit_mask: 0 }; TREE_DEPTH as usize];
    let mut depth = 0;
    while depth < TREE_DEPTH {
        let bit_pos = 255 - depth;
        masks[depth as usize] = PathMask {
            byte_pos: (bit_pos / 8) as usize,
            bit_mask: 0x01 << (bit_pos % 8),
        };
        depth += 1;
    }
    masks
};

#[derive(Debug, Clone)]
pub struct SparseMerkleTree {
    default_hashes: [HashOut<F>;TREE_DEPTH as usize + 1],
    nodes: HashMap<Key, HashOut<F>>,
    leaves: HashMap<Index256, F>,
}

impl SparseMerkleTree {
    const DEFAULT_VALUE: F = F::ZERO;

    pub fn new() -> Self {
        let mut default_hashes = [Self::default_leaf_hash(); TREE_DEPTH as usize + 1];
        for i in (0..= TREE_DEPTH as usize - 1).rev() {
            default_hashes[i] = Self::combine(&default_hashes[i + 1], &default_hashes[i + 1]);
        }

        Self {
            default_hashes,
            nodes: HashMap::new(),
            leaves: HashMap::new(),
        }
    }

    pub fn insert(&mut self, index: Index256, value: F) {
        self.leaves.insert(index, value);

        let key = Self::composite_key(TREE_DEPTH, &index);
        let leaf_hash = PoseidonHash::hash_no_pad(&[value]);
        self.nodes.insert(key, leaf_hash);
        self.update_path(TREE_DEPTH, index);
    }

    pub fn get(&self, index: &Index256) -> Option<&F> {
        self.leaves.get(index)
    }

    pub fn root(&self) -> HashOut<F> {
        let root_index = [0u8; 32];
        self.get_node(0, &root_index)
            .unwrap_or_else(|| self.default_hashes[0])
    }

    pub fn prove(&self, index: &Index256) -> (F, [HashOut<F>; 256]) {
        let mut proof = [HashOut::ZERO; TREE_DEPTH as usize];
        let mut current_index = *index;
        let value = self.leaves.get(index).unwrap_or(&Self::DEFAULT_VALUE);
        
        for depth in (1..=TREE_DEPTH).rev() {
            let sibling_index = Self::get_sibling_index(depth, &current_index);
            proof[(TREE_DEPTH - depth) as usize] = self.get_node(depth, &sibling_index).unwrap_or_else(|| 
                self.default_hashes[depth as usize]
            );
            current_index = Self::parent_index(depth, &current_index);
        }
        
        (value.clone(), proof)
    }

    pub fn verify_proof(&self, root_hash: &HashOut<F>, index: &Index256, proof: &[HashOut<F>; 256]) -> bool {
        let value = match self.get(index) {
            Some(v) => *v,
            None => return false,
        };

        let leaf_hash = PoseidonHash::hash_no_pad(&[value]);
        let mut current_hash = leaf_hash;
        let mut current_index = *index;
    
        for (depth, sibling_hash) in proof.iter().enumerate() {
            let depth = TREE_DEPTH - depth as u16 - 1;
            let is_right = SparseMerkleTree::is_right_child(depth + 1, &current_index);
    
            current_hash = if is_right {
                SparseMerkleTree::combine(sibling_hash, &current_hash)
            } else {
                SparseMerkleTree::combine(&current_hash, sibling_hash)
            };
    
            current_index = SparseMerkleTree::parent_index(depth + 1, &current_index);
        }
    
        current_hash == *root_hash
    }

    pub fn verify_proof_with_data(root_hash: &HashOut<F>, index: &Index256, value: F, proof: &[HashOut<F>; 256]) -> bool {
        let leaf_hash = PoseidonHash::hash_no_pad(&[value]);
        let mut current_hash = leaf_hash;
        let mut current_index = *index;
    
        for (i, sibling_hash) in proof.iter().enumerate() {
            let depth = TREE_DEPTH - i as u16 - 1;
            let is_right = SparseMerkleTree::is_right_child(depth + 1, &current_index);
    
            current_hash = if is_right {
                SparseMerkleTree::combine(sibling_hash, &current_hash)
            } else {
                SparseMerkleTree::combine(&current_hash, sibling_hash)
            };
    
            current_index = SparseMerkleTree::parent_index(depth + 1, &current_index);
        }
    
        current_hash == *root_hash
    }

    fn update_path(&mut self, mut depth: u16, mut index: Index256) {
        while depth > 0 {
            let parent_depth = depth - 1;
            let parent_index = Self::parent_index(depth, &index);
            let sibling_index = Self::get_sibling_index(depth, &index);
    
            let current_hash = self.get_node(depth, &index)
                .unwrap_or_else(|| self.default_hashes[depth as usize]);
            let sibling_hash = self.get_node(depth, &sibling_index)
                .unwrap_or_else(|| self.default_hashes[depth as usize]);
    
            let parent_hash = if Self::is_right_child(depth, &index) {
                Self::combine(&sibling_hash, &current_hash)
            } else {
                Self::combine(&current_hash, &sibling_hash)
            };
    
            self.nodes.insert(Self::composite_key(parent_depth, &parent_index), parent_hash);
            depth = parent_depth;
            index = parent_index;
        }
    }

    fn is_right_child(depth: u16, index: &Index256) -> bool {
        Self::get_bit(index, depth)
    }

    fn parent_index(depth: u16, index: &Index256) -> Index256 {
        let mut parent = *index;
        Self::clear_bit(&mut parent, depth);
        parent
    }

    fn get_sibling_index(depth: u16, index: &Index256) -> Index256 {
        let mut sibling = *index;
        Self::flip_bit(&mut sibling, depth);
        sibling
    }

    fn get_bit(index: &Index256, depth: u16) -> bool {
        let mask = &PATH_MASKS[depth as usize - 1];
        (index[mask.byte_pos] & mask.bit_mask) != 0
    }

    fn clear_bit(index: &mut Index256, depth: u16) {
        let mask = &PATH_MASKS[depth as usize - 1];
        index[mask.byte_pos] &= !mask.bit_mask;
    }

    fn flip_bit(index: &mut Index256, depth: u16) {
        let mask = &PATH_MASKS[depth as usize - 1];
        index[mask.byte_pos] ^= mask.bit_mask;
    }

    fn default_leaf_hash() -> HashOut<F> {
        PoseidonHash::hash_no_pad(&[Self::DEFAULT_VALUE])
    }

    fn combine(left: &HashOut<F>, right: &HashOut<F>) -> HashOut<F> {
        let mut inputs = left.elements.to_vec();
        inputs.extend_from_slice(&right.elements);
        PoseidonHash::hash_no_pad(&inputs)
    }

    fn composite_key(depth: u16, index: &Index256) -> [u8; 34] {
        let mut key = [0u8; 34];
        key[0..2].copy_from_slice(&depth.to_le_bytes());
        key[2..].copy_from_slice(index);
        key
    }

    fn get_node(&self, depth: u16, index: &Index256) -> Option<HashOut<F>> {
        self.nodes.get(&Self::composite_key(depth, index)).copied()
    }
}