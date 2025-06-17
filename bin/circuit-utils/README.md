# Circuit Utils

## Usage

To generate random boolean circuits:

```bash
cargo run --bin circuit-cli random -i 1600 -g 10000000  -r 0.9863  --output ../circuits/random/random_10milgates_72to1.bristol
```

### Flags

`-i`: Number of input wires
`-g`: Number of gates
`-r` frac of xor gates amongst total number of gates

If `-r` is set to 0.9 then 90% of the total number of gates are XOR.

## Limitations

Only AND and XOR gates are supported as of now.
