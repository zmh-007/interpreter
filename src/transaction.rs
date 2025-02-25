use crate::zk::GoldilocksField;
use crate::zk::Proof;
use crate::zk::VerifyingKey;
pub struct Transaction {
    pub new: GoldilocksField,
    pub proof: Proof,
    pub vk: VerifyingKey,
}
