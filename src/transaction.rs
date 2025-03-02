use crate::GoldilocksField;
use crate::Hash;
use crate::Proof;
use crate::VerificationKey;
#[derive(Debug, Clone)]
pub struct Transaction {
    pub proof: Proof,
    pub vk: VerificationKey,
    pub updates: Vec<(Hash, [GoldilocksField; 8])>,
}
