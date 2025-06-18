use garble::garble::{garble_ckt, gen_label_hash};
use risc0_zkvm::guest::env;
use rkyv::{api::high::to_bytes_with_alloc, deserialize, rancor::Error, ser::allocator::Arena};
use validityproof_core::{ArchivedGuestInput, GuestInput, GuestOutput};

fn main() {
    // initialize a byte array of length 4 to receive the size of serialized input
    let mut input_size_bytes = vec![0u8; 4];
    env::read_slice(&mut input_size_bytes);

    // reconstruct u32 from the byte array
    let input_size = u32::from_le_bytes(input_size_bytes.try_into().unwrap());

    // initialize byte vectors to receive serialized value from host    
    let mut input_bytes = vec![0u8; input_size.try_into().unwrap()];
    env::read_slice(&mut input_bytes);

    println!("Input Bytes Length: {} bytes", input_bytes.len());

    // get zero copy deserialization of inputs
    let circuit_input_archieved =
        rkyv::access::<ArchivedGuestInput, Error>(&input_bytes[..]).unwrap();

    // deserialize inputs
    let input = deserialize::<GuestInput, Error>(circuit_input_archieved).unwrap();

    // compute hash of the input labels
    let label_hashes = gen_label_hash(input.labels.input_labels.clone());

    // compute garbled tables
    let garbled_tables = garble_ckt(input.input_circuit, input.labels);

    // create a struct to store the values that need to be committed as public
    let public_values = GuestOutput {
        serialized_circuit: input_bytes,
        label_hashes,
        garbled_tables,
    };

    // serialize the public values using rkyv since the default serde is slow
    let mut arena = Arena::new();
    let public_values_bytes =
        to_bytes_with_alloc::<_, Error>(&public_values, arena.acquire()).unwrap();

    // commit to the output garbled tables which commits to circuit, hash of input labels and garbled table
    // These values can be read from receipt's journal
    env::commit_slice(&public_values_bytes);
}
