use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let proto_dir = PathBuf::from("../../proto");

    // Watch the entire proto tree so cargo re-runs this script when any .proto
    // changes. buf lint (run in CI / pre-commit) validates the same sources.
    println!("cargo:rerun-if-changed=../../proto");

    // Collect all .proto files under the buf module root. The directory layout
    // follows buf conventions: each package lives in a matching subdirectory
    // (e.g. shortcode/v1/shortcode.proto for package shortcode.v1).
    let protos: Vec<PathBuf> = glob::glob("../../proto/**/*.proto")?
        .map(|entry| Ok(entry?))
        .collect::<Result<_, Box<dyn Error>>>()?;

    tonic_prost_build::configure().compile_protos(&protos, &[proto_dir])?;

    Ok(())
}
