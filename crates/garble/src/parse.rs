use crate::input::Circuit;
use crate::input::GateDef;
use std::path::Path;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
};

pub fn load_seed<P: AsRef<Path>>(path: P) -> io::Result<[u8; 32]> {
    // Open the file in read-only mode.
    let mut f = File::open(path)?;
    // Prepare a 32-byte array to fill.
    let mut seed = [0u8; 32];
    // Read exactly 32 bytes (returns Err if the file is shorter).
    f.read_exact(&mut seed)?;
    Ok(seed)
}

pub fn parse_bristol<P: AsRef<Path>>(path_to_bristol: P) -> anyhow::Result<Circuit> {
    let file = File::open(path_to_bristol)?;
    let mut rdr = BufReader::new(file);
    let mut line = String::new();

    // Parse header_line1: <num_gates> <num_wires>
    rdr.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let total_gate_count: usize = parts.next().unwrap().parse()?;
    let total_wire_count: usize = parts.next().unwrap().parse()?;
    line.clear();

    // Parse header_line2: Input description
    // TODO: (mukesh) considers only 1 input, generalize it.
    rdr.read_line(&mut line)?;
    let mut ins = line.split_whitespace();
    let _num_inputs: usize = ins.next().unwrap().parse()?;
    let input1_count: usize = ins.next().unwrap().parse()?;
    let input2_count: usize = 0;
    line.clear();

    // Parse header_line3: Output description
    // TODO: (mukesh) considers only 1 output, generalize it.
    rdr.read_line(&mut line)?;
    let mut ins = line.split_whitespace();
    let _num_outputs: usize = ins.next().unwrap().parse()?;
    let output_wire_count: usize = ins.next().unwrap().parse()?;
    line.clear();

    // store all the gates
    let mut gates = Vec::with_capacity(total_gate_count);

    // initialize variables to get the gate counts
    let mut and_gate_count: usize = 0;
    let mut xor_gate_count: usize = 0;
    let mut not_gate_count: usize = 0;

    // loop over lines in the ckt to parse the gate
    for _ in 0..total_gate_count {
        rdr.read_line(&mut line)?;

        let mut p = line.split_whitespace();

        let fan_in: usize = p.next().unwrap().parse()?;

        let _fan_out: usize = p.next().unwrap().parse()?;

        let in0: usize = p.next().unwrap().parse()?;

        let in1: Option<usize> = if fan_in == 1 {
            None
        } else {
            Some(p.next().unwrap().parse()?)
        };

        let output = p.next().unwrap().parse()?;

        let gstr = p.next().unwrap();

        match gstr {
            "AND" => and_gate_count += 1,
            "XOR" => xor_gate_count += 1,
            "INV" => not_gate_count += 1,
            other => {
                anyhow::bail!("unexpected gate `{}`", other)
            }
        }

        let gate = match gstr {
            "AND" => GateDef::And {
                in0,
                in1: in1.unwrap(),
                out: output,
            },
            "XOR" => GateDef::Xor {
                in0,
                in1: in1.unwrap(),
                out: output,
            },
            "INV" => GateDef::Not {
                input: in0,
                out: output,
            },
            other => {
                anyhow::bail!("unexpected gate `{}`", other)
            }
        };
        gates.push(gate);
        line.clear();
    }

    // TODO: @mukesh (sanity checks) The gate and wire counts as claimed by header and in the circuit
    // may be inconsistent. Can we trust the input? It might be better to either validate them, or
    // compute them from circuit description as needed. For now, its ok to have all values
    // once the garbling code is done, this can be updated.
    let ckt = Circuit {
        total_gate_count,
        and_gate_count,
        not_gate_count,
        xor_gate_count,
        total_wire_count,
        input1_count,
        input2_count,
        output_wire_count,
        gates,
    };
    Ok(ckt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rand_core::{OsRng, TryRngCore};
    use std::{fs, path::PathBuf};

    fn generate_and_store_seed<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        // Get 32 bytes from the OS random source
        let mut buf = [0u8; 32];
        let _ = OsRng.try_fill_bytes(&mut buf);

        // Write them exactly as raw bytes to "seed.bin"
        fs::write(path, buf)?;
        Ok(())
    }

    #[test]
    fn test_generate_then_load_seed() {
        let path: PathBuf = "seed.bin".into();
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }

        generate_and_store_seed(path.clone()).expect("generate_and_store_seed failed!");

        // Verify that the file now exists and is exactly 32 bytes on disk
        let metadata = fs::metadata(&path).expect("Failed to read metadata of seed file");
        assert_eq!(metadata.len(), 32, "Seed file should be exactly 32 bytes");

        // 2. Load the seed back from the file
        let seed = load_seed(path.clone()).expect("load_seed_from_file should succeed");

        // Assert that we got exactly 32 bytes back
        assert_eq!(seed.len(), 32, "Loaded seed array must have length 32");

        // (Optional) Ensure the loaded bytes match the on-disk contents exactly
        let raw = fs::read(&path).expect("Failed to read raw bytes");
        assert_eq!(&seed[..], &raw[..], "Loaded seed must match file contents");
    }
}
