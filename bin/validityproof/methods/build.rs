fn main() {
    let is_clippy_driver = std::env::var("RUSTC_WORKSPACE_WRAPPER")
        .map(|val| val.contains("clippy-driver"))
        .unwrap_or(false);
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = std::path::Path::new(&out_dir);
    let methods_path = out_dir.join("methods.rs");

    let elf = r#"
            pub const FREEXORGARBLE_ELF: &[u8] = &[];
            pub const FREEXORGARBLE_ID: [u32; 8] = [0u32; 8];
        "#;

    std::fs::write(methods_path, elf).expect("Failed to write mock rollup elf");

    if is_clippy_driver {
    } else {
        risc0_build::embed_methods();
    }
}
