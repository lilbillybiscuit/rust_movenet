use std::io::Result;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/proto/dnn_message.proto");
    prost_build::compile_protos(&["src/proto/dnn_message.proto"], &["src/proto/"])?;

    println!("cargo:rerun-if-changed=c_resources/wrapper.h");

    // bindgen generate bindings
    let bindings = bindgen::Builder::default()
        .header("c_resources/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new())) // Updated line
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_file = out_path.join("video2dev_bindings.rs");
    bindings
        .write_to_file(&bindings_file)
        .expect("Couldn't write bindings!");

    Ok(())
}