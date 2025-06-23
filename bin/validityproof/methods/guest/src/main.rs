use garble::garble::{garble_ckt, gen_label_hash};
use garble::input::{Circuit, LabelInputs};
use risc0_zkvm::guest::env;
use rkyv::{api::high::to_bytes_with_alloc, deserialize, rancor::Error, ser::allocator::Arena};
use sha2::{Digest, Sha256};
use validityproof_core::GuestOutput;

fn main() {
    // Read circuit size and circuit bytes
    let mut circuit_size_bytes = vec![0u8; 4];
    env::read_slice(&mut circuit_size_bytes);
    let circuit_size = u32::from_le_bytes(circuit_size_bytes.try_into().unwrap());

    let mut circuit_bytes = vec![0u8; circuit_size.try_into().unwrap()];
    env::read_slice(&mut circuit_bytes);

    // Hash the circuit bytes BEFORE deserializing to save memory
    let mut hasher = Sha256::new();
    hasher.update(&circuit_bytes);
    let circuit_hash: [u8; 32] = hasher.finalize().into();

    // Read labels size and labels bytes
    let mut labels_size_bytes = vec![0u8; 4];
    env::read_slice(&mut labels_size_bytes);
    let labels_size = u32::from_le_bytes(labels_size_bytes.try_into().unwrap());

    let mut labels_bytes = vec![0u8; labels_size.try_into().unwrap()];
    env::read_slice(&mut labels_bytes);

    println!("Circuit Bytes Length: {} bytes", circuit_bytes.len());
    println!("Labels Bytes Length: {} bytes", labels_bytes.len());

    // Deserialize circuit and labels separately
    let circuit_archived =
        rkyv::access::<rkyv::Archived<Circuit>, Error>(&circuit_bytes[..]).unwrap();
    let circuit = deserialize::<Circuit, Error>(circuit_archived).unwrap();

    let labels_archived =
        rkyv::access::<rkyv::Archived<LabelInputs>, Error>(&labels_bytes[..]).unwrap();
    let labels = deserialize::<LabelInputs, Error>(labels_archived).unwrap();

    // Drop the serialized circuit bytes to save memory (we only need the hash now)
    drop(circuit_bytes);

    // compute hash of the input labels
    let label_hashes = gen_label_hash(&labels.input_labels);

    // compute garbled tables
    let garbled_tables = garble_ckt(circuit, labels);

    // create a struct to store the values that need to be committed as public
    // NOTE: Only commit to circuit_hash, not labels
    let public_values = GuestOutput {
        circuit_hash,
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
