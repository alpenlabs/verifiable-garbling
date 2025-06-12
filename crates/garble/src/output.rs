use rkyv::{Archive, Deserialize, Serialize};

// #[derive(serde::Serialize, serde::Deserialize)]
#[derive(Archive, Serialize, Deserialize)]
pub struct GarbledTables {
    pub and_tables: Vec<AndGateTable>,
    pub not_tables: Vec<NotGateTable>,
}

// #[derive(serde::Serialize, serde::Deserialize)]
#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct AndGateTable {
    pub gate: usize,
    pub in0: usize,
    pub in1: usize,
    pub out: usize,
    // four ciphertexts ordered (a=0,b=0) .. (1,1)
    pub table: [[u8; 16]; 4],
}

// #[derive(serde::Serialize, serde::Deserialize)]
#[derive(Archive, Serialize, Deserialize)]
pub struct NotGateTable {
    pub gate: usize,
    pub input: usize,
    pub out: usize,
    pub table: [[u8; 16]; 2],
}
