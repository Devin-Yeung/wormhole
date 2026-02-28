use glob::glob;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    // collect all proto files
    let protos = glob("../../proto/**/*.proto")?
        .map(|entry| {
            let path = entry?;
            println!("cargo:rerun-if-changed={}", path.display());
            Ok(path)
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    // get the include path
    let proto_dir = PathBuf::from("../../proto");

    // Compile all protos together so imports are resolved
    tonic_prost_build::configure().compile_protos(&protos, &[proto_dir])?;

    Ok(())
}
