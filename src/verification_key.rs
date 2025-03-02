use anyhow::Result;
use plonky2::hash::hash_types::HashOutTarget;
use plonky2::hash::merkle_tree::MerkleCap;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::iop::target::Target;
use plonky2::iop::witness::PartialWitness;
use plonky2::iop::witness::WitnessWrite;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::circuit_data::CircuitData;
use plonky2::plonk::circuit_data::CommonCircuitData;
use plonky2::plonk::circuit_data::VerifierCircuitData;
use plonky2::plonk::circuit_data::VerifierOnlyCircuitData;
use plonky2::plonk::config::Hasher;
use plonky2::plonk::proof::ProofWithPublicInputs;
use plonky2::plonk::proof::ProofWithPublicInputsTarget;
use serde::Deserialize;
use serde::Serialize;
use std::array::from_fn;
use std::sync::LazyLock;
pub type Config = plonky2::plonk::config::PoseidonGoldilocksConfig;
pub type GoldilocksField = plonky2::field::goldilocks_field::GoldilocksField;
pub type Hash = plonky2::hash::hash_types::HashOut<GoldilocksField>;
pub type Proof = plonky2::plonk::proof::Proof<GoldilocksField, Config, 2>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationKey {
    constants_sigmas_cap: MerkleCap<GoldilocksField, PoseidonHash>,
    circuit_digest: Hash,
}
impl VerificationKey {
    pub fn setup<const ZK: bool>(f: impl FnOnce(&mut CircuitBuilder<GoldilocksField, 2>, [HashOutTarget; 3]) -> Vec<Target>) -> (Self, impl Fn([Hash; 3], Vec<GoldilocksField>) -> Result<Proof>) {
        let (co, ci, pis, targets, proof_with_pis_target) = Self::compile::<ZK>(f);
        (Self { constants_sigmas_cap: co.verifier_only.constants_sigmas_cap.clone(), circuit_digest: co.verifier_only.circuit_digest }, move |hashes, values| {
            let mut wi = PartialWitness::<GoldilocksField>::new();
            (0..3).try_for_each(|i| wi.set_hash_target(pis[i], hashes[i]))?;
            wi.set_target_arr(&targets, &values)?;
            let proof_with_pis = ci.prove(wi)?;
            let mut wo = PartialWitness::<GoldilocksField>::new();
            wo.set_proof_with_pis_target(&proof_with_pis_target, &proof_with_pis)?;
            Ok(co.prove(wo)?.proof)
        })
    }
    pub fn verify(&self, root: Hash, mesg: Hash, proof: Proof) -> Result<()> { (VerifierCircuitData { verifier_only: VerifierOnlyCircuitData { constants_sigmas_cap: self.constants_sigmas_cap.clone(), circuit_digest: self.circuit_digest }, common: Self::common() }).verify(ProofWithPublicInputs { proof, public_inputs: [self.address().elements, root.elements, mesg.elements].concat() }) }
    pub fn address(&self) -> Hash { PoseidonHash::hash_no_pad(&self.constants_sigmas_cap.0.iter().map(|v| v.elements).flatten().chain(self.circuit_digest.elements).collect::<Vec<_>>()) }
    fn common() -> CommonCircuitData<GoldilocksField, 2> {
        static COMMON: LazyLock<CommonCircuitData<GoldilocksField, 2>> = LazyLock::new(|| VerificationKey::compile::<true>(|_, _| vec![]).0.common);
        COMMON.clone()
    }
    fn compile<const ZK: bool>(f: impl FnOnce(&mut CircuitBuilder<GoldilocksField, 2>, [HashOutTarget; 3]) -> Vec<Target>) -> (CircuitData<GoldilocksField, Config, 2>, CircuitData<GoldilocksField, Config, 2>, [HashOutTarget; 3], Vec<Target>, ProofWithPublicInputsTarget<2>) {
        let mut bi = CircuitBuilder::new(if ZK { CircuitConfig::standard_recursion_zk_config() } else { CircuitConfig::standard_recursion_config() });
        let pis = from_fn(|_| bi.add_virtual_hash_public_input());
        let targets = f(&mut bi, pis);
        let ci = bi.build::<Config>();
        let mut bo = CircuitBuilder::new(CircuitConfig::standard_recursion_config());
        let proof_with_pis_target = bo.add_virtual_proof_with_pis(&ci.common);
        let inner_verifier_data = bo.constant_verifier_data(&ci.verifier_only);
        bo.register_public_inputs(&proof_with_pis_target.public_inputs);
        bo.verify_proof::<Config>(&proof_with_pis_target, &inner_verifier_data, &ci.common);
        let co = bo.build::<Config>();
        (co, ci, pis, targets, proof_with_pis_target)
    }
}
