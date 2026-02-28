use glob::glob;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for entry in glob("../../proto/**/*.proto")? {
        let path = entry?;
        tonic_prost_build::compile_protos(path)?;
    }
    Ok(())
}
