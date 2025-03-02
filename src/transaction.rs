use crate::GoldilocksField;
use crate::Hash;
use crate::Proof;
use crate::VerificationKey;
use serde::Deserialize;
use serde::Serialize;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub proof: Proof,
    pub vk: VerificationKey,
    pub updates: Vec<(Hash, [GoldilocksField; 8])>,
}
