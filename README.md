# Garbled Circuits and its Validity Proofs

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache-blue.svg)](https://opensource.org/licenses/apache-2-0)
[![ci](https://github.com/alpenlabs/verifiable-garbling/actions/workflows/lint.yml/badge.svg?event=push)](https://github.com/alpenlabs/verifiable-garbling/actions)

> [!IMPORTANT]
> This software is a work-in-progress meant for research and as such, is _not_ meant to be used in a production environment!

> [!WARNING]
> **Security Notice**: CPU/Single GPU proving now uses risc0-zkvm v2.1.0. However, multi-GPU proving via bento_cli still uses v2.0.2 due to the vulnerability reported [here](https://github.com/risc0/risc0/security/advisories/GHSA-g3qg-6746-3mg9). The bento_cli upgrade is pending.

This is an implementation of garbled circuit with free-xor optimization as well as a zk proof of correct garbling using risczero zkvm.

This ensures that garbling circuit protocol is secure against malicious adversaries.
We need to ensure that the garbler is constructing the garbling table for the agreed upon boolean circuit and also ensure that the table is constructed correctly.

This means that, even before evaluation, anyone can verify that the evaluation will succeed and correspond to the agreed-upon boolean circuit

## Key Features

- Free-XOR optimization for efficient XOR operations
- RISC Zero integration for proof generation  
- Support for Bristol Fashion circuit format
- Scalable to circuits with 30+ million gates
- Multi-GPU proof generation support

## Quick Start

### Prerequisites

- Rust (latest stable version)
- Git
- 16GB+ RAM recommended for larger circuits

### Basic Usage

1. Clone the repository:

   ```bash
   git clone https://github.com/alpenlabs/verifiable-garbling
   cd verifiable-garbling
   ```

2. Run a simple example:

   ```bash
   RUST_LOG=info RISC0_DEV_MODE=1 cargo run -p validityproof circuits/example1/example1.bristol seed.bin
   ```

3. Expected output:

   ```bash
   INFO garbling circuit with 4 gates...
   INFO proof generation completed
   INFO saved to elf_and_inputs/input.bin
   ```

![FlowChart for Garbling and OT proofs](./gc_flow.png)

The above diagram can be accessed by the [permalink](https://excalidraw.com/#json=am-3JTklHgd7PQt2yk6Rd,WMfQXMzK2kjoWY0FMF0lpA) or excalidraw file `gc_flow.excalidraw`

## Crates and Directories

- **bin/circuit-utils**\
   Utilities to work with boolean circuit. Includes a parser for bristol fashion circuits and way to generate random boolean circuits with desired number of gates.
- **circuits**\
  Contains few example circuits and their description
- **crates/garble**\
  the main crate that parses bristol fashion files and generates garbled tables
- **bin/validityproof**\
  Generates proof of correct garbling using risc0
- **logs**\
  Stores info of runs of proof generation

## Generating garbled tables and proof

### running on CPU or Single GPU

To generate the garbled table and proof that garbling was done correctly using risc0, run:

```bash
RUST_LOG=info RISC0_INFO=1 cargo run -p validityproof <boolean_file> <seed_file>
```

The `boolean_file` is representation of the boolean circuit in bristol fashion as detailed [here](https://nigelsmart.github.io/MPC-Circuits/)

The `seed_file` is a 32 byte values used to initialize the CS-RNG to generate the labels.

```bash
RUST_LOG=info RISC0_DEV_MODE=1 RISC0_INFO=1 cargo run -p validityproof circuits/example1/example1.bristol seed.bin
```

Due to the env variable `RISC0_DEV_MODE=1`, the above command generates mock proof but allows to get details of cycle counts and also save the serialized input to file.

To generate actual proofs, set `RISC0_DEV_MODE=0`


### Running with Multiple GPUs

For distributed proof generation across multiple GPUs using [Bento](https://github.com/risc0/risc0/tree/main/bento), see our detailed [Multi-GPU Setup Guide](docs/MULTI_GPU_SETUP.md).

**Quick summary:**
1. Set up AWS instance with multiple GPUs (e.g., g6.12x)
2. Install dependencies: Docker, NVIDIA drivers, Rust, RISC Zero
3. Install `bento_cli` from our fork to ensure version compatibility
4. Configure Docker Compose for your GPU count
5. Run proof generation:
   ```bash
   RUST_LOG=info bento_cli -f ELF_file -i input.bin -s -o output_path
   ```

**File locations:**
- ELF file: `target/riscv-guest/garbling-methods/freexorgarble/riscv32im-risc0-zkvm-elf/release/freexorgarble.bin`
- Input file: Generated in `elf_and_inputs/input.bin` when running [CPU/Single GPU commands](#running-on-cpu-or-single-gpu)

## Using Circuit Utils

### Generating Random Circuits

```bash
cargo run --bin circuit-utils random -i 4 -g 10  -r 0.5  --output circuits/random/random_test.bristol
```

### Flags

`-i`: Number of input wires
`-g`: Number of gates
`-r`: Fraction of XOR gates among the total number of gates.

If `-r` is set to 0.9 then 90% of the total number of gates are XOR.


## Benchmarks

| Circuit            | Total Gates | AND Gates | XOR Gates | INV Gates | Cycle Counts       |
|--------------------|------------:|----------:|----------:|----------:|-------------------:|
| example1           |        4    |        2  |        2  |      0    |          65,536    |
| example2           |   28,032    |    8,128  |   19,904  |      0    |      42,991,616    |
| example3           |  344,671    |   57,947  |  286,724  |  4,946    |     367,067,136    |
| random_1mil_gates  |1,000,000    |  100,000  |  900,000  |      0    |   1,435,500,544    |
| random_10mil_gates |10,000,000   |  136,797  |9,863,203  |      0    |  12,047,089,664    |
| random_20mil_gates |20,000,000   |  274,873  |19,725,127 |      0    |  24,789,385,216    |
| random_30mil_gates |30,000,000   |  411,441  |29,588,559 |      0    |  37,706,006,528    |

More detailed benchmarks and estimates of time and cost of producing proofs is at [the google sheets](https://docs.google.com/spreadsheets/d/1eevdDvaPIOrKF8rlpQFpkSJ2ttlDV_-BcC1MkK_ywR4/edit?gid=855613280#gid=855613280).

## Troubleshooting

### Common Issues

**"No such file or directory" when running examples:**

- Ensure you're in the project root directory
- Check that the circuit file exists.
- Check that the seed file exists.

**GPU setup issues:**

- Verify NVIDIA drivers: `nvidia-smi`
- Check Docker is running: `docker ps`
- Ensure all GPUs are visible: `nvtop`

**Proof generation fails:**

- Try with `RISC0_DEV_MODE=1` first to test without actual proof generation
- Check available disk space (proofs can be large)

## Limitations, Optimizations and TODOs

- **TODO: Upgrade bento_cli to use risc0-zkvm v2.1.0**\
Update bento_cli dependency to use the latest secure risc0-zkvm version. CPU/Single GPU proving already uses v2.1.0, but multi-GPU proving via bento_cli still uses the vulnerable v2.0.2.

- **The guest program has a memory of 3 GB**\
If we exceed this, we might have to chunk the boolean circuit into smaller segments.

Due to this, the largest circuit size supported is around 30 mil gates with (1:72 ratio of AND:XOR).

- **Only AND, XOR and INV (NOT) gates are supported as of now.**\
Further gates can be added.
- **NOT gate is handled as a separate gate with two entries in garbled table.**\
More efficient ways to handle NOT by either absorbing it into inputs of other gates or emulating NOT using XOR can be done
- **The data sent from host to guest is deserialized by guest before use.**
Rkyv supports direct access without deserialization using Archived Types. We would need to ensure garbling works with these types.
- **Evaluation of Garbled Circuit has not been implemented**
- **Comprehensive testing is needed**\
Including unit tests for core components, integration tests for end-to-end workflows, and property-based tests to ensure circuit correctness and security guarantees.

## Contributing

Contributions are generally welcome.
If you intend to make larger changes please discuss them in an issue
before opening a PR to avoid duplicate work and architectural mismatches.

For more information please see [`CONTRIBUTING.md`](/CONTRIBUTING.md).

## License

This work is dual-licensed under MIT and Apache 2.0.
You may choose either license if you use this work.
