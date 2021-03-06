use std::{env, error::Error, path::PathBuf};

use cc::Build;

fn main() -> Result<(), Box<dyn Error>> {
    //build directory for this crate
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    //The target architecture
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    //extend the library search path
    println!("cargo:rustc-link-search={}", out_dir.display());

    if arch == "arm" {
        Build::new().file("./src/arm.s").compile("asm");
    } else if arch == "riscv32" {
        Build::new().file("./src/riscv32.s").compile("asm");
    } else {
        println!("Invalid architecture: {}", arch);
    }

    Ok(())
}
