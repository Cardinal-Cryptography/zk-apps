use ark_std::{end_timer, start_timer};
use halo2_base::{
    gates::circuit::{builder::BaseCircuitBuilder, BaseCircuitParams, CircuitBuilderStage},
    halo2_proofs::{
        dev::MockProver,
        halo2curves::bn256::{Bn256, Fr, G1Affine},
        plonk::{keygen_pk, keygen_vk, verify_proof, Circuit, ProvingKey, VerifyingKey},
        poly::{
            commitment::ParamsProver,
            kzg::{
                commitment::{KZGCommitmentScheme, ParamsKZG},
                multiopen::VerifierSHPLONK,
                strategy::SingleStrategy,
            },
        },
    },
    utils::BigPrimeField,
    AssignedValue,
};
use rand::{rngs::StdRng, SeedableRng};
use snark_verifier_sdk::{
    halo2::{gen_snark_shplonk, PoseidonTranscript},
    NativeLoader, Snark,
};

pub fn gen_srs(k: u32) -> ParamsKZG<Bn256> {
    ParamsKZG::<Bn256>::setup(k, StdRng::from_seed(Default::default()))
}

/// An interface for circuit generator using halo2-lib
/// See the examples in the `circuit_runner/examples` directory for usage
pub trait CircuitGenerator<F: BigPrimeField> {
    type WitnessInput: Default + Clone;
    fn run(
        &self,
        builder: &mut BaseCircuitBuilder<F>,
        input: Self::WitnessInput,
        make_public: &mut Vec<AssignedValue<F>>,
    );
}

fn create_circuit<F: BigPrimeField, CG: CircuitGenerator<F>>(
    cg: &CG,
    stage: CircuitBuilderStage,
    lookup_bits: Option<usize>,
    // It seems to me this `k` does not need to be the same as `k` from srs, but it should not be larger
    // Supposedly this is max number of rows the halo2-lib circuit generator will use in a single column
    // If the circuit is larger than this, it will be split into multiple columns
    k_circuit: usize,
    input: CG::WitnessInput,
) -> BaseCircuitBuilder<F> {
    assert!(!matches!(stage, CircuitBuilderStage::Prover));
    let mut builder = BaseCircuitBuilder::from_stage(stage);
    builder.set_k(k_circuit);
    if let Some(lookup_bits) = lookup_bits {
        builder.set_lookup_bits(lookup_bits);
    }
    builder.set_instance_columns(1);

    let mut assigned_instances = vec![];
    cg.run(&mut builder, input, &mut assigned_instances);
    // Not sure what's the point of the below code
    if !assigned_instances.is_empty() {
        assert_eq!(
            builder.assigned_instances.len(),
            1,
            "num_instance_columns != 1"
        );
        builder.assigned_instances[0] = assigned_instances;
    }
    if !stage.witness_gen_only() {
        // now `builder` contains the execution trace, and we are ready to actually create the circuit
        // minimum rows is the number of rows used for blinding factors. This depends on the circuit itself, but we can guess the number and change it if something breaks (default 9 usually works)
        let minimum_rows = 20;
        builder.calculate_params(Some(minimum_rows));
    }
    builder
}

pub fn create_circuit_prover<F: BigPrimeField, CG: CircuitGenerator<F>>(
    cg: &CG,
    params: BaseCircuitParams,
    break_points: Vec<Vec<usize>>,
    input: CG::WitnessInput,
) -> BaseCircuitBuilder<F> {
    let mut builder = BaseCircuitBuilder::from_stage(CircuitBuilderStage::Prover);
    builder.set_params(params);
    builder.set_break_points(break_points);
    builder.set_instance_columns(1);
    let mut assigned_instances = vec![];
    cg.run(&mut builder, input, &mut assigned_instances);
    // Not sure what's the point of the below code
    if !assigned_instances.is_empty() {
        assert_eq!(
            builder.assigned_instances.len(),
            1,
            "num_instance_columns != 1"
        );
        builder.assigned_instances[0] = assigned_instances;
    }
    builder
}

pub fn create_circuit_keygen<F: BigPrimeField, CG: CircuitGenerator<F>>(
    cg: &CG,
    lookup_bits: Option<usize>,
    k_circuit: usize,
) -> BaseCircuitBuilder<F> {
    create_circuit(
        cg,
        CircuitBuilderStage::Keygen,
        lookup_bits,
        k_circuit,
        CG::WitnessInput::default(),
    )
}

pub fn create_circuit_mock<F: BigPrimeField, CG: CircuitGenerator<F>>(
    cg: &CG,
    lookup_bits: Option<usize>,
    k_circuit: usize,
    input: CG::WitnessInput,
) -> BaseCircuitBuilder<F> {
    create_circuit(cg, CircuitBuilderStage::Mock, lookup_bits, k_circuit, input)
}

#[derive(Clone)]
struct KeygenResult {
    params: ParamsKZG<Bn256>,
    pk: ProvingKey<G1Affine>,
    c_params: BaseCircuitParams,
    break_points: Vec<Vec<usize>>,
}

fn keygen<CG: CircuitGenerator<Fr>>(cg: &CG, k_circuit: usize, k_srs: u32) -> KeygenResult {
    assert!(k_circuit <= k_srs as usize, "k_circuit > k_srs");

    let srs_time = start_timer!(|| "Generating srs");
    let params = gen_srs(k_srs);
    end_timer!(srs_time);

    let generate_time = start_timer!(|| "Generating circuit");
    let circuit = create_circuit_keygen(cg, None, k_circuit);
    end_timer!(generate_time);

    let vk_time = start_timer!(|| "Generating vk");
    let vk = keygen_vk(&params, &circuit).unwrap();
    end_timer!(vk_time);

    let pk_time = start_timer!(|| "Generating pk");
    let pk = keygen_pk(&params, vk.clone(), &circuit).unwrap();
    end_timer!(pk_time);

    let c_params = circuit.params();
    let break_points = circuit.break_points();

    KeygenResult {
        params,
        pk,
        c_params,
        break_points,
    }
}

fn generate_snark<CG: CircuitGenerator<Fr>>(
    cg: &CG,
    kg: KeygenResult,
    input: CG::WitnessInput,
) -> Snark {
    let KeygenResult {
        params,
        pk,
        c_params,
        break_points,
        ..
    } = kg;
    let circuit = create_circuit_prover(cg, c_params, break_points, input);
    gen_snark_shplonk(&params, &pk, circuit, None::<String>)
}

fn verify_snark(
    vk: VerifyingKey<G1Affine>,
    snark: Snark,
    params: ParamsKZG<Bn256>,
    instance: Vec<Fr>,
) -> bool {
    let verifier_params = params.verifier_params();
    let strategy = SingleStrategy::new(&params);
    let mut transcript = PoseidonTranscript::<NativeLoader, &[u8]>::new::<0>(&snark.proof[..]);
    let instance = &instance;
    let verify_time = start_timer!(|| "Verifying proof");
    let res = verify_proof::<
        KZGCommitmentScheme<Bn256>,
        VerifierSHPLONK<'_, Bn256>,
        _,
        _,
        SingleStrategy<'_, Bn256>,
    >(
        verifier_params,
        &vk,
        strategy,
        &[&[instance]],
        &mut transcript,
    );
    end_timer!(verify_time);
    res.is_ok()
}

/// k_srs -- 2^{k_srs} is the size of the srs
/// k_circuit -- 2^{k_circuit} is the max number of rows the halo2-lib circuit generator will use in a single column
pub fn mock_run<F: BigPrimeField, CG: CircuitGenerator<F>>(
    cg: &CG,
    input: CG::WitnessInput,
    instance: Vec<F>,
    k_circuit: usize,
    k_srs: u32,
) {
    let circuit = create_circuit_mock(cg, None, k_circuit, input);
    let expected_instance_len = circuit.assigned_instances[0].len();
    assert_eq!(instance.len(), expected_instance_len);
    MockProver::run(k_srs, &circuit, vec![instance])
        .unwrap()
        .assert_satisfied();
    println!("Mock run successful");
}

/// k_srs -- 2^{k_srs} is the size of the srs
/// k_circuit -- 2^{k_circuit} is the max number of rows the halo2-lib circuit generator will use in a single column
pub fn run<CG: CircuitGenerator<Fr>>(
    cg: &CG,
    input: CG::WitnessInput,
    instance: Vec<Fr>,
    k_circuit: usize,
    k_srs: u32,
) {
    println!("Generating key...");
    let kg = keygen(cg, k_circuit, k_srs);
    let params = kg.params.clone();
    let vk = kg.pk.get_vk().clone();
    println!("Generating snark...");
    let snark = generate_snark(cg, kg, input.clone());
    println!("Verifying snark...");
    let res = verify_snark(vk, snark, params, instance);
    println!("Snark verification result: {}", res);
}
