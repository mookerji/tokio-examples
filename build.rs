use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    tonic_build::compile_protos("proto/service.proto")?;
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("service_descriptor.bin"))
        .compile(&["proto/service.proto"], &["proto"])?;
    Ok(())
}
