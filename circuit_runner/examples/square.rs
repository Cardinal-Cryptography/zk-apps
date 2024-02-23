use circuit_runner::{mock_run, run, CircuitGenerator};
use halo2_base::{
    gates::{circuit::builder::BaseCircuitBuilder, GateChip, GateInstructions},
    halo2_proofs::halo2curves::{bn256::Fr, ff::PrimeField},
    utils::BigPrimeField,
    AssignedValue,
};

#[derive(Clone, Debug)]
struct WitnessInput<F: BigPrimeField> {
    x: F,
}

impl<F: BigPrimeField> Default for WitnessInput<F> {
    fn default() -> Self {
        WitnessInput { x: F::from_u128(0) }
    }
}

struct SquareCircuit<F: BigPrimeField> {
    _marker: std::marker::PhantomData<F>,
}

impl<F: BigPrimeField> Default for SquareCircuit<F> {
    fn default() -> Self {
        SquareCircuit {
            _marker: std::marker::PhantomData,
        }
    }
}

// Checks that x*x = y, for x being the witness, and y the instance
impl<F: BigPrimeField> CircuitGenerator<F> for SquareCircuit<F> {
    type WitnessInput = WitnessInput<F>;

    fn run(
        &self,
        builder: &mut BaseCircuitBuilder<F>,
        input: Self::WitnessInput,
        make_public: &mut Vec<AssignedValue<F>>,
    ) {
        let ctx = builder.main(0);
        let cx = ctx.load_witness(input.x);
        let gate = GateChip::<F>::default();
        let cy = gate.mul(ctx, cx, cx);
        make_public.push(cy);
    }
}

fn mock_run_circuit() {
    let k_srs = 6;
    let k_circuit = 5;
    let cg = SquareCircuit::<Fr>::default();
    let input = WitnessInput {
        x: Fr::from_u128(2),
    };

    let instance = vec![Fr::from_u128(4)];
    println!("Running on correct data, this should succeed:");
    mock_run(&cg, input.clone(), instance, k_circuit, k_srs);

    println!("Running on incorrect data, this should panic:");
    let instance = vec![Fr::from_u128(5)];
    mock_run(&cg, input, instance, k_circuit, k_srs);
}

fn run_circuit() {
    let k_srs = 6;
    let k_circuit = 5;
    let cg = SquareCircuit::<Fr>::default();
    let input = WitnessInput {
        x: Fr::from_u128(2),
    };

    let instance = vec![Fr::from_u128(4)];
    run(&cg, input.clone(), instance, k_circuit, k_srs);
}

fn main() {
    run_circuit();
    mock_run_circuit();
}
