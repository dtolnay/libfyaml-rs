use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let header = "libfyaml/include/libfyaml.h";
    println!("cargo:rerun-if-changed={}", header);

    if let Ok(false) = Path::new(header).try_exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init", "libfyaml"])
            .status();
    }

    let bindings = bindgen::Builder::default()
        .header(header)
        .generate_comments(false)
        .generate()
        .unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_file = Path::new(&out_dir).join("bindings.rs");
    bindings.write_to_file(out_file).unwrap();
}
