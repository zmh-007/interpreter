use anyhow::Result;
pub use plonky2::field::goldilocks_field::GoldilocksField;
pub use plonky2::field::types::Field;
pub use plonky2::field::types::Field64;
use plonky2::iop::witness::PartialWitness;
pub use plonky2::iop::witness::WitnessWrite;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::circuit_data::CircuitData;
use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2::util::serialization::DefaultGateSerializer;
type Config = plonky2::plonk::config::PoseidonGoldilocksConfig;
pub type Proof = plonky2::plonk::proof::Proof<GoldilocksField, Config, 2>;
#[derive(Clone)]
pub struct VerifyingKey(plonky2::plonk::circuit_data::VerifierCircuitData<GoldilocksField, Config, 2>);
impl VerifyingKey {
    pub fn verify(&self, public_inputs: Vec<GoldilocksField>, proof: Proof) -> Result<()> { self.0.verify(ProofWithPublicInputs { proof, public_inputs }) }
    pub fn to_bytes(&self) -> Vec<u8> { self.0.to_bytes(&DefaultGateSerializer).unwrap() }
}
pub struct Circuit<T> {
    c: CircuitData<GoldilocksField, Config, 2>,
    t: T,
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
