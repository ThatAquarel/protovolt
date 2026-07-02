//! Linker script from protov-nvm (application profile).

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    fs::write(
        out.join("memory.x"),
        include_str!("../protov-nvm/linker/memory-application.x"),
    )
    .unwrap();

    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=../protov-nvm/linker/memory-application.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // With protov-bootloader, remove link-rp.x
    // such that .boot2 loads to proper address
    // arm-none-eabi-objdump -h target/thumbv6m-none-eabi/release/protov | grep .boot2
    // println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
}
