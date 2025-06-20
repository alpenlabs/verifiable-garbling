# Multi-GPU Setup Guide

This guide provides detailed instructions for setting up distributed proof generation across multiple GPUs using [Bento](https://github.com/risc0/risc0/tree/main/bento).

## Overview

To get multi-GPU proof generation, we need to ensure that the STARK proof for the segments are being distributed to the GPUs and then collected. This orchestration is handled by Bento.

## Setup Steps

### 1. (Optional) Spin up an AWS instance

- Ensure at least 100 GB persistent storage
- Choose Ubuntu 22.04
- Use some multi-GPU instance type like g6.12x

### 2. Install build dependencies and `just`

```bash
sudo apt update
sudo apt upgrade
sudo apt install build-essential
sudo snap install just --classic
```

### 3. Install nvtop

```bash
sudo apt install nvtop
```

### 4. Install Rust

Follow the [official installation guide](https://www.rust-lang.org/tools/install) for more detailed instructions.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 5. Install RISC Zero

Follow the [official installation guide](https://dev.risczero.com/api/zkvm/install) for more detailed instructions.

```bash
curl -L https://risczero.com/install | bash
```

```bash
rzup install cargo-risczero 2.0.2
```

```bash
rzup install rust
```

### 6. Install `bento_cli`

```bash
cargo install --git https://github.com/alpenlabs/risc0 --branch mukesh/add_bento_to_v2.1 bento-client --bin bento_cli
```

**Note**: This installs `bento_cli` from a fork to ensure that the version of RISC Zero expected by Bento matches with RISC Zero installed previously.

### 7. Clone the boundless repo

Get the Docker Compose files and select the branch:

```bash
git clone https://github.com/alpenlabs/boundless
cd boundless
git checkout mukesh/multiple_gpu
```

**Note**: This has `compose.yml` configured for 4 GPUs. You can change it depending on your device by updating the `compose.yml` with instructions provided in the [docs](https://docs.beboundless.xyz/provers/quick-start#configuring-bento).

### 8. Install NVIDIA drivers and Docker

```bash
sudo ./scripts/setup.sh
```

**Important**: You will need to restart after this step.

### 9. Spin up the Docker images

```bash
just bento up
```

### 10. Run the test

Monitor the GPU usage to confirm that all GPUs are being utilized:

```bash
RUST_LOG=info bento_cli -s -c 4096
```

### 11. Generate proofs

If the test was successful, you can generate the SNARK proof by running:

```bash
RUST_LOG=info bento_cli -f ELF_file -i input.bin -s -o path_to_output
```

- Remove the `-s` flag to get a STARK proof instead
- The proofs along with public parameters (together called receipt) are saved at the path specified using the `-o` flag

## File Locations

- **ELF file**: `target/riscv-guest/garbling-methods/freexorgarble/riscv32im-risc0-zkvm-elf/release/freexorgarble.bin`
- **Input file**: Generated and saved to `elf_and_inputs/input.bin` when you run the CPU/Single GPU commands

## Troubleshooting

### GPU Issues

- Verify NVIDIA drivers: `nvidia-smi`
- Check Docker is running: `docker ps`
- Ensure all GPUs are visible: `nvtop`

### Bento Issues

- Check Docker Compose configuration matches your GPU count
- Ensure all containers are running: `docker ps`
- Monitor logs: `docker logs <container_name>`

### Performance Issues

- Verify all GPUs are being utilized with `nvtop`
- Check for memory constraints