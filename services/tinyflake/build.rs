use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let proto_dir = PathBuf::from("../../proto");
    println!("cargo:rerun-if-changed=../../proto/tinyflake");

    let protos: Vec<PathBuf> = glob::glob("../../proto/tinyflake/**/*.proto")?
        .map(|entry| Ok(entry?))
        .collect::<Result<_, Box<dyn Error>>>()?;

    tonic_prost_build::configure().compile_protos(&protos, &[proto_dir])?;

    Ok(())
}
