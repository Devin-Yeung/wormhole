fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("../../api/shortener/v1/shortener.proto")?;
    Ok(())
}
