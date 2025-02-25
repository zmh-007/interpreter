use anyhow::Result;
pub use plonky2::field::goldilocks_field::GoldilocksField;
pub use plonky2::field::types::Field;
pub use plonky2::field::types::Field64;
use plonky2::hash::hash_types::HashOut;
pub use plonky2::hash::merkle_proofs::MerkleProofTarget;
pub use plonky2::hash::poseidon::PoseidonHash;
use plonky2::iop::witness::PartialWitness;
pub use plonky2::iop::witness::WitnessWrite;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::circuit_data::CircuitData;
pub use plonky2::plonk::config::Hasher;
use plonky2::plonk::proof::ProofWithPublicInputs;
type Config = plonky2::plonk::config::PoseidonGoldilocksConfig;
pub type Hash = HashOut<GoldilocksField>;
pub type Proof = plonky2::plonk::proof::Proof<GoldilocksField, Config, 2>;
#[derive(Clone)]
pub struct VerifyingKey(plonky2::plonk::circuit_data::VerifierCircuitData<GoldilocksField, Config, 2>);
pub struct Circuit<T> {
    c: CircuitData<GoldilocksField, Config, 2>,
    t: T,
}
impl VerifyingKey {
    pub fn verify(&self, root: Hash, new: GoldilocksField, proof: Proof) -> Result<()> {
        let [[a, b, c, d], [x, y, z, w]] = [self.address().elements, root.elements];
        self.0.verify(ProofWithPublicInputs { proof, public_inputs: vec![a, b, c, d, x, y, z, w, new] })
    }
    pub fn address(&self) -> Hash { PoseidonHash::hash_no_pad(&self.0.verifier_only.constants_sigmas_cap.0.iter().map(|v| v.elements).flatten().chain(self.0.verifier_only.circuit_digest.elements).collect::<Vec<_>>()) }
}
impl<T> Circuit<T> {
    pub fn new<Func>(f: Func) -> Self
    where Func: FnOnce(&mut CircuitBuilder<GoldilocksField, 2>) -> T {
        let mut builder = CircuitBuilder::<GoldilocksField, 2>::new(CircuitConfig::standard_recursion_zk_config());
        let t = f(&mut builder);
        Self { c: builder.build::<Config>(), t }
    }
    pub fn prove<Func>(&self, f: Func) -> Result<(Proof, Vec<GoldilocksField>)>
    where Func: FnOnce(&mut PartialWitness<GoldilocksField>, &T) -> Result<()> {
        let mut w = PartialWitness::<GoldilocksField>::new();
        f(&mut w, &self.t)?;
        let pi = self.c.prove(w)?;
        Ok((pi.proof, pi.public_inputs))
    }
    pub fn vk(&self) -> VerifyingKey { VerifyingKey(self.c.verifier_data()) }
}
