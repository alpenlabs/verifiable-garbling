use crate::input::{Circuit, GateDef, Label, LabelInputs, WireLabels};
use crate::output::{AndGateTable, GarbledTables, NotGateTable};
use sha2::{Digest, Sha256};

// this xors the 128 bit labels
pub fn xor_labels(a: &Label, b: &Label) -> Label {
    let mut r = [0u8; 16];
    for i in 0..16 {
        r[i] = a[i] ^ b[i];
    }
    r
}

pub fn gen_label_hash(labels: Vec<Label>) -> Vec<[u8; 32]> {
    //generate the hash for the labels corresponding to both bits of the input wires
    let mut input_labels_hash = Vec::with_capacity(2 * labels.len());
    for label in labels {
        let hash_zero_label: [u8; 32] = Sha256::digest(label).into();
        input_labels_hash.push(hash_zero_label);
    }
    input_labels_hash
}

/// sha256-based pad: H(ka || kb)
// This is used to get the masking value for the output gate labels
// if two gates might share the same inputs, we need to append the gate_id to the value being hashed to get differnt table entries.
// TODO: @mukesh (optimization) For Not gates, only 1 label is enough. currently, I just duplicate the same label. this function
// signature can be overloaded to handle both.
fn pad_sha(ka: &Label, kb: &Label) -> Label {
    // let mut h = Hasher::new(); //for blake3
    let mut h = Sha256::new();
    h.update(ka);
    h.update(kb);
    let digest = h.finalize(); // 32 bytes
    let mut out = [0u8; 16]; // only 16 bytes are needed since our labels are 16 bytes
    out.copy_from_slice(&digest[..16]);
    out
}

/// Returns garbled tables corresponding to a circuit, delta and label_list for input and inner gate wires
/// (except for XOR).
pub fn garble_ckt(ckt_inputs: Circuit, label_inputs: LabelInputs) -> GarbledTables {
    let wcnt = ckt_inputs.total_wire_count;
    let _gcnt = ckt_inputs.total_gate_count;
    let in1 = ckt_inputs.input1_count;
    let in2 = ckt_inputs.input2_count;
    let gates = ckt_inputs.gates;

    let delta = label_inputs.delta;
    let mut inner_iter = label_inputs.inner_labels.into_iter();

    // pre-allocate wire slots
    let mut wires: Vec<Option<WireLabels>> = Vec::with_capacity(wcnt);
    // 1) load input labels
    for k0 in label_inputs.input_labels {
        let k1 = xor_labels(&k0, &delta);
        wires.push(Some(WireLabels { k0, k1 }));
    }
    // 2) the rest start empty
    for _ in (in1 + in2)..wcnt {
        wires.push(None);
    }

    // 3) Prepare output tables
    let mut and_tables = Vec::new();
    let mut not_tables = Vec::new();

    //generate the garbled table
    for (idx, gate) in gates.iter().enumerate() {
        match *gate {
            GateDef::Xor { in0, in1, out } => {
                // free‐XOR: just assign labels
                let lu = wires[in0].as_ref().unwrap();
                let lv = wires[in1].as_ref().unwrap();
                let k0 = xor_labels(&lu.k0, &lv.k0);
                let k1 = xor_labels(&k0, &delta);
                wires[out] = Some(WireLabels { k0, k1 });
            }

            GateDef::And { in0, in1, out } => {
                let lu = wires[in0].clone().unwrap();
                let lv = wires[in1].clone().unwrap();

                let k0_out = inner_iter.next().unwrap();
                let k1_out = xor_labels(&k0_out, &delta);
                wires[out] = Some(WireLabels {
                    k0: k0_out,
                    k1: k1_out,
                });

                let mut table: [[u8; 16]; 4] = [[0u8; 16]; 4];
                let combos = [(0u8, 0u8), (0, 1), (1, 0), (1, 1)];
                for (i, (a, b)) in combos.iter().enumerate() {
                    let ka = if *a == 0 { lu.k0 } else { lu.k1 };
                    let kb = if *b == 0 { lv.k0 } else { lv.k1 };
                    let out_bit = a & b;
                    let kout = if out_bit == 0 { k0_out } else { k1_out };
                    let p = pad_sha(&ka, &kb);
                    let ct = xor_labels(&p, &kout);
                    table[i] = ct;
                }

                and_tables.push(AndGateTable {
                    gate: idx,
                    in0,
                    in1,
                    out,
                    table,
                });
            }

            GateDef::Not { input, out } => {
                let lu = wires[input].clone().unwrap();

                let k0_out = inner_iter.next().unwrap();
                let k1_out = xor_labels(&k0_out, &delta);
                wires[out] = Some(WireLabels {
                    k0: k0_out,
                    k1: k1_out,
                });

                let mut table: [[u8; 16]; 2] = [[0u8; 16]; 2];
                for (i, &a) in [0u8, 1].iter().enumerate() {
                    let ka = if a == 0 { lu.k0 } else { lu.k1 };
                    let out_bit = 1 - a;
                    let kout = if out_bit == 0 { k0_out } else { k1_out };
                    let p = pad_sha(&ka, &ka); // unary, duplicate
                                               // let p = pad_poseidon(&ka, &ka);
                    let ct = xor_labels(&p, &kout);
                    table[i] = ct;
                }

                not_tables.push(NotGateTable {
                    gate: idx,
                    input,
                    out,
                    table,
                });
            }
        }
    }

    GarbledTables {
        and_tables,
        not_tables,
    }
}

#[cfg(test)]
mod tests {
    use crate::garble::garble_ckt;
    use crate::garble::pad_sha;
    use crate::input::Circuit;
    use crate::input::GateDef;
    use crate::input::LabelInputs;

    use super::xor_labels;
    use super::Label;

    #[test]
    fn xor_zero_zero_is_zero() {
        let a: Label = [0; 16];
        let b: Label = [0; 16];
        let result = xor_labels(&a, &b);
        assert_eq!(result, [0; 16], "0 ⊕ 0 should be 0");
    }

    #[test]
    fn xor_with_zero_returns_same() {
        let a: Label = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let zero: Label = [0; 16];
        assert_eq!(xor_labels(&a, &zero), a, "a ⊕ 0 should be a");
        assert_eq!(xor_labels(&zero, &a), a, "0 ⊕ a should be a");
    }

    #[test]
    fn xor_same_label_is_zero() {
        let a: Label = [0xFF; 16];
        assert_eq!(xor_labels(&a, &a), [0; 16], "a ⊕ a should be 0");
    }

    #[test]
    fn xor_is_commutative() {
        let a: Label = [
            0x0F, 0xF0, 0xAA, 0x55, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
        ];
        let b: Label = [
            0xF0, 0x0F, 0x55, 0xAA, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
        ];
        let ab = xor_labels(&a, &b);
        let ba = xor_labels(&b, &a);
        assert_eq!(ab, ba, "a ⊕ b should equal b ⊕ a");
    }

    #[test]
    fn xor_matches_manual_bytewise() {
        let a: Label = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x0F, 0xED, 0xCB, 0xA9, 0x87, 0x65,
            0x43, 0x21,
        ];
        let b: Label = [0xFF; 16];
        let expected: Label = [
            0xED, 0xCB, 0xA9, 0x87, 0x65, 0x43, 0x21, 0x0F, 0xF0, 0x12, 0x34, 0x56, 0x78, 0x9A,
            0xBC, 0xDE,
        ];
        assert_eq!(xor_labels(&a, &b), expected);
    }

    #[test]
    fn test_garble_and_single_and() {
        // Circuit: 2 inputs (0,1), output at wire 2, one AND gate.
        let ckt = Circuit {
            total_wire_count: 3,
            total_gate_count: 1,
            input1_count: 1,
            input2_count: 1,
            gates: vec![GateDef::And {
                in0: 0,
                in1: 1,
                out: 2,
            }],
            and_gate_count: 1,
            not_gate_count: 0,
            xor_gate_count: 0,
            output_wire_count: 1,
        };

        let labels = LabelInputs {
            delta: [8u8; 16],
            inner_labels: vec![[7u8; 16]],
            input_labels: vec![[2u8; 16], [5u8; 16]],
        };

        let l_a0 = labels.input_labels[0];
        let l_a1 = xor_labels(&l_a0, &labels.delta);
        let l_b0 = labels.input_labels[1];
        let l_b1 = xor_labels(&l_b0, &labels.delta);
        let l_c0 = labels.inner_labels[0];
        let l_c1 = xor_labels(&l_c0, &labels.delta);

        let tbls = garble_ckt(ckt, labels.clone());
        // should have exactly one AND table
        assert_eq!(tbls.and_tables.len(), 1);
        let t = &tbls.and_tables[0];
        assert_eq!(t.gate, 0);
        assert_eq!(t.in0, 0);
        assert_eq!(t.in1, 1);
        assert_eq!(t.out, 2);
        // The table has 4 entries of 16 bytes each and are correctly formed
        assert_eq!(t.table.len(), 4);

        assert_eq!(&xor_labels(&pad_sha(&l_a0, &l_b0), &t.table[0]), &l_c0);
        assert_eq!(&xor_labels(&pad_sha(&l_a0, &l_b1), &t.table[1]), &l_c0);
        assert_eq!(&xor_labels(&pad_sha(&l_a1, &l_b0), &t.table[2]), &l_c0);
        assert_eq!(&xor_labels(&pad_sha(&l_a1, &l_b1), &t.table[3]), &l_c1);
    }
}
