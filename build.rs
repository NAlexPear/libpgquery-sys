use std::{env, path::PathBuf};

const TARGET_HEADER: &str = "./libpg_query/pg_query.h";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get the output path from the environment
    let out_path = PathBuf::from(env::var("OUT_DIR")?);

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed={}", TARGET_HEADER);

    // make sure that the libraries are linked correctly
    println!("cargo:rustc-link-search=native=./libpg_query");
    println!("cargo:rustc-link-lib=static=pg_query");

    // generate the bindings
    bindgen::Builder::default()
        .header(TARGET_HEADER)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .map_err(|_| "Unable to generate bindings")?
        .write_to_file(out_path.join("bindings.rs"))?;

    // generate the proto definitions
    prost_build::compile_protos(
        &["./libpg_query/protobuf/pg_query.proto"],
        &["./libpg_query/protobuf"],
    )?;

    Ok(())
}
