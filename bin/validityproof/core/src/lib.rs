use garble::{
    input::{Circuit, LabelInputs},
    output::GarbledTables,
};
use rkyv::{Archive, Deserialize, Serialize};

/// Struct to store the inputs to the guest
#[derive(Archive, Serialize, Deserialize, serde::Serialize, serde::Deserialize)]
pub struct GuestInput {
    pub input_circuit: Circuit,
    pub labels: LabelInputs,
}
/// Struct to store the public inputs that the guest generates
#[derive(Archive, Serialize, Deserialize)]
pub struct GuestOutput {
    pub serialized_circuit: Vec<u8>,
    pub label_hashes: Vec<[u8; 32]>,
    pub garbled_tables: GarbledTables,
}
