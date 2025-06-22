fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = std::path::Path::new(&out_dir);
    let methods_path = out_dir.join("methods.rs");

    let elf = r#"
            pub const FREEXORGARBLE_ELF: &[u8] = &[];
            pub const FREEXORGARBLE_ID: [u32; 8] = [0u32; 8];
        "#;

    // Check if RUSTC_WORKSPACE_WRAPPER is set to clippy-driver (i.e. if `cargo clippy` is the
    // current compiler). If so, don't execute `cargo prove build` because it breaks
    // rust-analyzer's `cargo clippy` feature.
    let is_clippy_driver = std::env::var("RUSTC_WORKSPACE_WRAPPER")
        .map(|val| val.contains("clippy-driver"))
        .unwrap_or(false);

    if is_clippy_driver {
        std::fs::write(methods_path, elf).expect("Failed to write mock rollup elf");
    } else {
        risc0_build::embed_methods();
    }
}
