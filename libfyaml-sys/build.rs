use std::env;
use std::path::Path;

fn main() {
    let header = "libfyaml/include/libfyaml.h";
    println!("cargo:rerun-if-changed={}", header);

    let bindings = bindgen::Builder::default()
        .header(header)
        .generate()
        .unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_file = Path::new(&out_dir).join("bindings.rs");
    bindings.write_to_file(out_file).unwrap();
}
