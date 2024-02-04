use anyhow::Result;
use prost_build::compile_protos;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../proto");

    build_proto()?;

    Ok(())
}

fn build_proto() -> Result<()> {
    compile_protos(&["../proto/search.proto"], &["../proto/"])?;
    Ok(())
}
