use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::parse::parse_bristol;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha12Rng;
use rkyv::{Archive, Deserialize, Serialize};

/// fixed-width 128-bit label
pub type Label = [u8; 16];

/// Stores the circuit information after parsing the input ckt
#[derive(
    Debug, Archive, Serialize, Deserialize, Default, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub struct Circuit {
    pub total_gate_count: usize,
    pub and_gate_count: usize,
    pub not_gate_count: usize,
    pub xor_gate_count: usize,
    pub total_wire_count: usize,
    pub input1_count: usize,
    pub input2_count: usize,
    pub output_wire_count: usize,
    pub gates: Vec<GateDef>,
}

impl Circuit {
    /// Number of primary input wires = garbler + evaluator inputs
    pub fn get_input_wire_count(&self) -> usize {
        self.input1_count + self.input2_count
    }

    /// Number of inner labels you must supply: one per AND and one per NOT
    pub fn get_inner_wire_count(&self) -> usize {
        self.and_gate_count + self.not_gate_count
    }
}

/// Struct to hold the two labels for each wires
#[derive(Clone)]
pub struct WireLabels {
    pub k0: Label,
    pub k1: Label,
}

/// One gate as parsed from Bristol, before garbling.
#[derive(
    Debug, Archive, Serialize, Deserialize, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub enum GateDef {
    And { in0: usize, in1: usize, out: usize },
    Xor { in0: usize, in1: usize, out: usize },
    Not { input: usize, out: usize },
}

#[derive(Archive, Serialize, Deserialize, Clone, serde::Serialize, serde::Deserialize)]
pub struct LabelInputs {
    //global delta for free XOR
    pub delta: Label,
    // zero labels for input wires
    pub input_labels: Vec<Label>,
    // zero labels for output of AND and NOT gates
    pub inner_labels: Vec<Label>,
}

//generate the labels
pub fn gen_labels(seed: [u8; 32], input_wire_count: usize, inner_wire_count: usize) -> LabelInputs {
    // The seed value is used to initialize a Chacha12 RNG which is cryptographically secure, which uses an internal 64 bit counter
    let mut rng = ChaCha12Rng::from_seed(seed);

    //initialize delta with random value. this is the global offset required for free-xor
    let mut delta = [0u8; 16];
    rng.fill_bytes(&mut delta);

    //generate the input labels
    let mut input_labels = Vec::with_capacity(input_wire_count);
    for _ in 0..input_wire_count {
        let mut k0 = [0u8; 16];
        rng.fill_bytes(&mut k0);
        input_labels.push(k0);
    }

    let mut inner_labels = Vec::with_capacity(inner_wire_count);
    for _ in 0..inner_wire_count {
        let mut k0 = [0u8; 16];
        rng.fill_bytes(&mut k0);
        inner_labels.push(k0);
    }

    LabelInputs {
        delta,
        inner_labels,
        input_labels,
    }
}

/// read the circuit
pub fn read_input_ckt(path_to_bristol: PathBuf) -> Circuit {
    parse_bristol(path_to_bristol).unwrap()
}

pub fn load_seed<P: AsRef<Path>>(path: P) -> io::Result<[u8; 32]> {
    // Open the file in read-only mode.
    let mut f = File::open(path)?;
    // Prepare a 32-byte array to fill.
    let mut seed = [0u8; 32];
    // Read exactly 32 bytes (returns Err if the file is shorter).
    f.read_exact(&mut seed)?;
    Ok(seed)
}
