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

    if env::var("CARGO_FEATURE_FACTORY_PROGRAM").is_ok() {
        emit_factory_record(&out);
    }

    // With protov-bootloader, remove link-rp.x
    // such that .boot2 loads to proper address
    // arm-none-eabi-objdump -h target/thumbv6m-none-eabi/release/protov | grep .boot2
    // println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
}

fn emit_factory_record(out: &PathBuf) {
    for key in [
        "SERIAL_NUMBER",
        "HARDWARE_REV",
        "FACTORY_YEAR",
        "FACTORY_MONTH",
        "FACTORY_DAY",
        "SIGNATURE",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }

    let serial = required_env("SERIAL_NUMBER");
    let hw_rev = required_env("HARDWARE_REV");
    let year: u16 = required_env("FACTORY_YEAR")
        .parse()
        .expect("FACTORY_YEAR must be u16");
    let month: u8 = required_env("FACTORY_MONTH")
        .parse()
        .expect("FACTORY_MONTH must be u8");
    let day: u8 = required_env("FACTORY_DAY")
        .parse()
        .expect("FACTORY_DAY must be u8");
    let signature = parse_signature_hex(&required_env("SIGNATURE"));

    let image = protov_nvm::FactoryRecord::encode(&hw_rev, &serial, year, month, day, &signature)
        .expect("factory record encode failed");

    let bytes = image
        .iter()
        .map(|b| format!("0x{b:02X}"))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(out.join("factory_record.rs"), format!("[{bytes}]")).unwrap();
}

fn required_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        panic!("factory-program requires {key} to be set at compile time");
    })
}

fn parse_signature_hex(hex: &str) -> [u8; 64] {
    let hex = hex.strip_prefix("#H").unwrap_or(hex);
    assert_eq!(
        hex.len(),
        128,
        "SIGNATURE must be 128 hex digits (64 bytes)"
    );
    let mut out = [0u8; 64];
    for (i, byte) in out.iter_mut().enumerate() {
        let pair = &hex[i * 2..i * 2 + 2];
        *byte = u8::from_str_radix(pair, 16).expect("SIGNATURE must be hex");
    }
    out
}
