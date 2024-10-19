use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/proto/dnn_message.proto"], &["src/proto/"])?;
    Ok(())
}