use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
    fs,
    io::{self, Write},
    path::Path,
};

use anyhow::Context;
use rand::{prelude::*, rng};

/// A wire is just an index into the global wire pool.
pub type WireId = usize;

/// gate types
#[derive(Debug, Clone, Copy)]
pub enum GateType {
    And,
    Xor,
    Inv,
}

impl Display for GateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            GateType::And => "AND",
            GateType::Xor => "XOR",
            GateType::Inv => "INV",
        };
        write!(f, "{s}")
    }
}

/// One gate in the circuit. The gates are assumed to have fan-out of 1
#[derive(Debug, Clone)]
pub struct Gate {
    pub gate_type: GateType,
    /// indices of input wires
    pub inputs: Vec<WireId>,
    /// Output wire id,
    pub output: WireId,
}

impl Gate {
    /// Render this gate as: fan_in fan_out in0 ...in_n out
    pub fn to_bristol_string(&self) -> String {
        let out = self.output;
        let fan_in = self.inputs.len();
        let inputs = self
            .inputs
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        format!("{} 1 {} {} {}", fan_in, inputs, out, self.gate_type)
    }
}

/// The full circuit.
#[derive(Debug, Clone)]
pub struct Circuit {
    /// total wires (so valid WireId ∈ 0..num_wires)
    pub num_wires: usize,
    /// which wires are inputs
    pub inputs: Vec<WireId>,
    /// which wires are outputs
    pub outputs: Vec<WireId>,
    /// all gates in order
    pub gates: Vec<Gate>,
}

impl Circuit {
    // TODO: @mukesh (yet to implement)
    // pub fn enumerate_io(&self) -> (Vec<GateId>, Vec<GateId>) { /* … */ }

    // pub fn fan_in(&self, gate: GateId) -> usize { /* … */ }
    // pub fn fan_out(&self, gate: GateId) -> usize { /* … */ }

    pub fn get_gate_count(&self) -> usize {
        self.gates.len()
    }

    pub fn get_wire_count(&self) -> usize {
        self.num_wires
    }

    pub fn get_input_wire_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn get_output_wire_count(&self) -> usize {
        self.outputs.len()
    }

    /// Parse a Bristol-format file at `path`, ignore the declared IO lines,
    /// and compute primary inputs/outputs from gate topology.
    pub fn from_bristol_file(path: &Path) -> anyhow::Result<Self> {
        // 1) Read the whole file
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read Bristol file `{}`", path.display()))?;

        let mut lines = text.lines().filter(|l| !l.trim().is_empty()).map(str::trim);

        // Parse header_line1: <num_gates> <num_wires>
        let header: Vec<&str> = lines
            .next()
            .context("missing header line")?
            .split_ascii_whitespace()
            .collect();
        let num_gates: usize = header[0].parse()?;
        let num_wires: usize = header[1].parse()?;

        // Parse header_line2: Input description
        // TODO: (mukesh) considers only 1 input, generalize it.
        let header: Vec<&str> = lines
            .next()
            .context("missing header line")?
            .split_ascii_whitespace()
            .collect();
        let _num_inputs: usize = header[0].parse()?;
        let _num_input_wires: usize = header[1].parse()?;

        // Parse header_line3: Output description
        // TODO: (mukesh) considers only 1 output, generalize it.
        let header: Vec<&str> = lines
            .next()
            .context("missing header line")?
            .split_ascii_whitespace()
            .collect();
        let _num_outputs: usize = header[0].parse()?;
        let _num_output_wires: usize = header[1].parse()?;

        // Parse all the gates
        let mut gates = Vec::with_capacity(num_gates);
        for line in lines.take(num_gates) {
            let mut tok = line.split_ascii_whitespace();
            let in_count: usize = tok.next().unwrap().parse()?;
            let _out_count: usize = tok.next().unwrap().parse()?;

            let inputs = (0..in_count)
                .map(|_| tok.next().unwrap().parse().unwrap())
                .collect();

            let output: usize = tok.next().unwrap().parse()?;

            let op = tok.next().unwrap();
            let gate_type = match op {
                "AND" => GateType::And,
                "XOR" => GateType::Xor,
                "INV" => GateType::Inv,
                other => anyhow::bail!("unsupported gate op `{}`", other),
            };
            gates.push(Gate {
                gate_type,
                inputs,
                output,
            });
        }

        // Build sets of *all* wires used as an output, and as an input
        let mut driven: HashSet<WireId> = HashSet::new();
        let mut used_in: HashSet<WireId> = HashSet::new();
        for g in &gates {
            driven.insert(g.output);
            for &w in &g.inputs {
                used_in.insert(w);
            }
        }

        // Primary inputs = those never driven by any gate
        let inputs: Vec<WireId> = (0..num_wires).filter(|w| !driven.contains(w)).collect();

        // Primary outputs = wires driven by some gate but never used as input
        let outputs: Vec<WireId> = driven
            .into_iter()
            .filter(|w| !used_in.contains(w))
            .collect();

        // // Ensure validity of the header values

        // // Constraint 1: total wire count = total gate count + input wire count
        // assert_eq!(num_wires, num_gates + num_input_wires, "Header and Circuit mismatch!");
        // assert_eq!(num_input_wires, inputs.len(), "Input Count mismatch");

        Ok(Circuit {
            num_wires,
            inputs,
            outputs,
            gates,
        })
    }

    pub fn from_bristol_file_no_header(path: &Path) -> anyhow::Result<Self> {
        // Read the file
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read Bristol file `{}`", path.display()))?;

        // Collect only non-empty trimmed lines
        let lines = text.lines().filter(|l| !l.trim().is_empty()).map(str::trim);

        let mut gates = Vec::new();
        let mut max_wire = 0_usize;

        for line in lines {
            let mut tok = line.split_ascii_whitespace();
            let in_count: usize = tok.next().unwrap().parse()?;
            let _out_count: usize = tok.next().unwrap().parse()?;

            let inputs: Vec<WireId> = (0..in_count)
                .map(|_| {
                    let w: WireId = tok.next().unwrap().parse().unwrap();
                    max_wire = max_wire.max(w);
                    w
                })
                .collect();

            let output: WireId = tok.next().unwrap().parse()?;
            max_wire = max_wire.max(output);

            let op = tok.next().unwrap();
            let gate_type = match op {
                "AND" => GateType::And,
                "XOR" => GateType::Xor,
                "INV" => GateType::Inv,
                other => anyhow::bail!("unsupported gate op `{}`", other),
            };

            gates.push(Gate {
                gate_type,
                inputs,
                output,
            });
        }

        // Calculate total wires: highest wire index + 1
        let num_wires = max_wire + 1;

        let mut driven: HashSet<WireId> = HashSet::new();
        let mut used_in: HashSet<WireId> = HashSet::new();
        for g in &gates {
            driven.insert(g.output);
            for &w in &g.inputs {
                used_in.insert(w);
            }
        }

        let inputs: Vec<WireId> = (0..num_wires).filter(|w| !driven.contains(w)).collect();
        let outputs: Vec<WireId> = driven
            .into_iter()
            .filter(|w| !used_in.contains(w))
            .collect();

        Ok(Circuit {
            num_wires,
            inputs,
            outputs,
            gates,
        })
    }

    /// Generate a random Boolean circuit with `gates` gates, `depth` layers,
    /// and `num_inputs`, `num_outputs` wires.
    pub fn random(num_inputs: usize, num_gates: usize, ratio_xor_to_and: f64) -> Self {
        let mut rng = rng();
        let mut gates = Vec::with_capacity(num_gates);

        // The number of available wires is initially the number of input wires
        let mut available: usize = num_inputs;

        // total number of wires is equal to sum of input wires and number of gates, assuming each gate has fan-out of 1
        let num_wires = num_inputs + num_gates;

        for _ in 0..num_gates {
            // TODO: (mukesh) For now I am considering only two gate types.
            let x: f64 = rng.random();
            let gate_type = if x < ratio_xor_to_and {
                GateType::Xor
            } else {
                GateType::And
            };

            let in0 = rng.random_range(0..available);
            let in1 = rng.random_range(0..available);

            //increment the available wires to add the output of the gate being generated
            let out = available;
            available += 1;

            gates.push(Gate {
                gate_type,
                inputs: vec![in0, in1],
                output: out,
            });
        }

        let mut driven: HashSet<WireId> = HashSet::new();
        let mut used_in: HashSet<WireId> = HashSet::new();
        for g in &gates {
            driven.insert(g.output);
            for &w in &g.inputs {
                used_in.insert(w);
            }
        }

        let inputs: Vec<WireId> = (0..num_wires).filter(|w| !driven.contains(w)).collect();
        let outputs: Vec<WireId> = driven
            .into_iter()
            .filter(|w| !used_in.contains(w))
            .collect();

        Circuit {
            num_wires,
            inputs,
            outputs,
            gates,
        }
    }

    /// Write the circuit in standard bristol fashion, header and one gate per line
    pub fn write_bristol_fashion<W: Write>(&self, mut w: W) -> io::Result<()> {
        // Line 1: <Gate Count> <Wire Count>
        writeln!(w, "{} {}", self.get_gate_count(), self.get_wire_count())?;

        // Line 2: List the number of inputs and number of input wires for each input
        // for now, we will simply put all input wires as a single input val
        writeln!(w, "1 {}", self.get_input_wire_count())?;

        // Line 3: List the number of outputs and number of output wires for each output
        // for now, we will simply put all output wires as a single output val
        writeln!(w, "1 {}", self.get_output_wire_count())?;

        // write the gates one per line
        for gate in &self.gates {
            writeln!(w, "{}", gate.to_bristol_string())?;
        }
        Ok(())
    }

    /// Write the circuit in standard bristol fashion without header
    pub fn write_bristol_fashion_no_header<W: Write>(&self, mut w: W) -> io::Result<()> {
        // write the gates one per line
        for gate in &self.gates {
            writeln!(w, "{}", gate.to_bristol_string())?;
        }
        Ok(())
    }

    /// returns the io details of the circuit
    pub fn enumerate_io(&self) -> (usize, usize) {
        // TODO: (mukesh) Implement this!
        (5, 6)
    }
}
