use garble::input::{gen_labels, load_seed, read_input_ckt};
use garbling_methods::{FREEXORGARBLE_ELF, FREEXORGARBLE_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use rkyv::{api::high::to_bytes_with_alloc, deserialize, rancor::Error, ser::allocator::Arena};
use std::env;
use std::fs::File;
use std::io::Write;
use validityproof_core::{ArchivedGuestOutput, GuestOutput};

fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // Setup the inputs.
    let mut args = env::args();
    let _program_name = args.next();
    let path_for_bristol = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("input file not found!");
            std::process::exit(1);
        }
    };
    let path_for_seed = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("input file not found!");
            std::process::exit(1);
        }
    };

    // load the circuit
    let input_ckt = read_input_ckt(path_for_bristol.into());

    // load the seed value
    let seed = load_seed(path_for_seed).unwrap();

    // get the details about the circuit
    let input_wire_count = input_ckt.get_input_wire_count();
    let inner_wire_count = input_ckt.get_inner_wire_count();
    let and_gate_count = input_ckt.and_gate_count;
    let xor_gate_count = input_ckt.xor_gate_count;
    let gate_count = input_ckt.gates.len();

    // compute the delta, input labels and inner label (output of gates other than XOR) using seed
    let labels = gen_labels(seed, input_wire_count, inner_wire_count);

    // serialize circuit and labels separately using rkyv
    let mut arena = Arena::new();
    let circuit_bytes = to_bytes_with_alloc::<_, Error>(&input_ckt, arena.acquire()).unwrap();
    let labels_bytes = to_bytes_with_alloc::<_, Error>(&labels, arena.acquire()).unwrap();

    // prepare sizes
    let circuit_size = circuit_bytes.len() as u32;
    let labels_size = labels_bytes.len() as u32;
    let circuit_size_bytes = circuit_size.to_le_bytes();
    let labels_size_bytes = labels_size.to_le_bytes();

    //write data to a file input.bin to be used by bento
    {
        let mut file =
            File::create("elf_and_inputs/input.bin").expect("couldn't create input.bin file");
        file.write_all(&circuit_size_bytes)
            .expect("couldn't write circuit size to input.bin");
        file.write_all(&circuit_bytes)
            .expect("couldn't write circuit to input.bin");
        file.write_all(&labels_size_bytes)
            .expect("couldn't write labels size to input.bin");
        file.write_all(&labels_bytes)
            .expect("couldn't write labels to input.bin");
        file.flush().expect("couldn't flush input.bin file");
    }
    let total_bytes = circuit_size_bytes.len()
        + circuit_bytes.len()
        + labels_size_bytes.len()
        + labels_bytes.len();
    println!(
        "Wrote {} bytes to input.bin to use with bento_cli",
        total_bytes
    );

    // initialize the env and pass circuit and labels to guest
    let env = ExecutorEnv::builder()
        .write_slice(&circuit_size_bytes)
        .write_slice(&circuit_bytes)
        .write_slice(&labels_size_bytes)
        .write_slice(&labels_bytes)
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Proof information by proving the specified ELF binary.
    // This struct contains the receipt along with statistics about execution of the guest
    let prove_info = prover.prove(env, FREEXORGARBLE_ELF).unwrap();

    // extract the receipt.
    let receipt = prove_info.receipt;

    // store the details of execution in a log file
    let log_path =
        format!("logs/circuit_{gate_count}gates_{and_gate_count}and_{xor_gate_count}xor.txt");

    let details = format!(
        "Circuit: {} gates, {} AND gates, {} XOR gates\nInput Wire Count: {}\nInner Wire Count: {}\nInput Bytes Length: {:.2} MB\nCycles: {}\n",
        gate_count,
        and_gate_count,
        xor_gate_count,
        input_wire_count,
        inner_wire_count,
        total_bytes as f64 / (1024.0 * 1024.0),
        prove_info.stats.total_cycles,
    );
    {
        let mut file = File::create(&log_path).expect("Failed to create log file");
        file.write_all(details.as_bytes())
            .expect("Failed to write to log file");
        file.flush().expect("Failed to flush log file");
    }

    // TODO: (mukesh) Implement code for saving receipt to file and performing validation
    let public_values_bytes = receipt.clone().journal.bytes;

    // get zero copy deserialization of serialized public values
    let public_values_archieved =
        rkyv::access::<ArchivedGuestOutput, Error>(&public_values_bytes[..]).unwrap();

    // deserialize inputs
    let _public_values: GuestOutput =
        deserialize::<GuestOutput, Error>(public_values_archieved).unwrap();

    // The receipt was verified at the end of proving, but the below code is an
    // example of how someone else could verify this receipt.
    receipt.verify(FREEXORGARBLE_ID).unwrap();
}
