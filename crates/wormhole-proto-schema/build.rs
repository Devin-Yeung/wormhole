fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/v1/shortener.proto")?;
    tonic_prost_build::compile_protos("proto/v1/redirector.proto")?;
    Ok(())
}
