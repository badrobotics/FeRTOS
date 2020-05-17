use std::{env, error::Error, fs::File, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    //build directory for this crate
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    //extend the library search path
    println!("cargo:rustc-link-search={}", out_dir.display());
    if cfg!(feature = "tm4c123") {
        File::create(out_dir.join("link.x"))?.write_all(include_bytes!("link_123.x"))?;
    } else {
        File::create(out_dir.join("link.x"))?.write_all(include_bytes!("link_1294.x"))?;
    }

    Ok(())
}
