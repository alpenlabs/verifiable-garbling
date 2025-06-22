use anyhow::Result;
use circuit_lib::Circuit;
use clap::{Parser, Subcommand};
use std::{fs::File, path::PathBuf};

#[derive(Parser)]
#[command(name = "circuit-tool", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a random circuit and store it in bristol fashion
    Random {
        /// Number of inputs
        #[arg(short = 'i', long = "inputs", value_name = "NUM_INPUT")]
        num_inputs: usize,

        /// Number of gates
        #[arg(short = 'g', long = "gates", value_name = "NUM_GATES")]
        num_gates: usize,

        /// path to write the simplified output (stdout if omitted)
        #[arg(short, long, value_name = "OUTPUT")]
        output: PathBuf,

        /// Ratio of XOR gates to AND gates
        #[arg(short, long, value_name = "RATIO")]
        ratio_xor_to_and: f64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Random {
            num_gates,
            num_inputs,
            output,
            ratio_xor_to_and,
        } => {
            let circuit = Circuit::random(num_inputs, num_gates, ratio_xor_to_and);

            let file = File::create(&output)
                .map_err(|e| anyhow::anyhow!("couldn't open {}: {}", output.display(), e))?;

            circuit.write_bristol_fashion(file)?;

            println!("Wrote random circuit to {}", output.display());
        }
    }
    Ok(())
}
