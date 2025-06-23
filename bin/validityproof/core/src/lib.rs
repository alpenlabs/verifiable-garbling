use garble::output::GarbledTables;
use rkyv::{Archive, Deserialize, Serialize};

/// Struct to store the public inputs that the guest generates
#[derive(Archive, Serialize, Deserialize)]
pub struct GuestOutput {
    pub circuit_hash: [u8; 32],
    pub label_hashes: Vec<[u8; 32]>,
    pub garbled_tables: GarbledTables,
}
