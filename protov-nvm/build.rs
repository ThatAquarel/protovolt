use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let memory_x = if env::var("CARGO_FEATURE_BOOTLOADER").is_ok() {
        include_str!("linker/memory-bootloader.x")
    } else if env::var("CARGO_FEATURE_APPLICATION").is_ok() {
        include_str!("linker/memory-application.x")
    } else {
        println!(
            "cargo:warning=protov-nvm: enable feature `bootloader` or `application` to emit memory.x"
        );
        return;
    };

    fs::write(out.join("memory.x"), memory_x).unwrap();
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rerun-if-changed=linker/memory-bootloader.x");
    println!("cargo:rerun-if-changed=linker/memory-application.x");
    println!("cargo:rerun-if-changed=build.rs");
}
