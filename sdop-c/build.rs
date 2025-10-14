extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Build a C-only configuration
    let config = cbindgen::Config {
        language: cbindgen::Language::C,
        include_guard: Some("SDOP_H".to_string()),
        ..Default::default() // keep other defaults
    };

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("../c_headers/sdop.h");
}
