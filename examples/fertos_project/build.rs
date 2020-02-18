use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    //build directory for this crate
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    //extend the library search path
    println!("cargo:rustc-link-search={}", out_dir.display());

    Ok(())
}
