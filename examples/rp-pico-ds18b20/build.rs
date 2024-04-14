use std::{env, error::Error, fs::File, io::prelude::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    // Make `memory.x` available to the linker.
    let out_dir = env::var("OUT_DIR")?;
    let out_dir = PathBuf::from(out_dir);

    let memory_x = include_bytes!("memory.x").as_ref();
    File::create(out_dir.join("memory.x"))?.write_all(memory_x)?;

    // Tell Cargo where to find the file.
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Tell Cargo to rebuild if `memory.x` is updated.
    println!("cargo:rerun-if-changed=memory.x");

    Ok(())
}