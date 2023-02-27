#![allow(clippy::let_underscore_untyped, clippy::uninlined_format_args)]

use std::env;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::io;
use std::path::Path;
use std::process::{self, Command};

fn main() {
    let windows = env::var_os("CARGO_CFG_WINDOWS").is_some();
    if windows {
        eprintln!("libfyaml is not supported on Windows.");
        eprintln!("See https://github.com/pantoniou/libfyaml/issues/10");
        process::exit(1);
    }

    let header = "libfyaml/include/libfyaml.h";
    println!("cargo:rerun-if-changed={}", header);

    if let Ok(false) = Path::new(header).try_exists() {
        let _ = Command::new("git")
            .args(["submodule", "update", "--init", "libfyaml"])
            .status();
    }

    let bindings = bindgen::Builder::default()
        .header(header)
        .allowlist_recursively(false)
        .allowlist_function("fy_.*")
        .allowlist_type("fy_.*")
        .blocklist_function("fy_library_version")
        // Variadic functions that use `va_list`.
        // Blocked on https://github.com/rust-lang/rust/issues/44930.
        .blocklist_function("fy_diag_node_override_vreport")
        .blocklist_function("fy_diag_node_vreport")
        .blocklist_function("fy_diag_vprintf")
        .blocklist_function("fy_document_vbuildf")
        .blocklist_function("fy_document_vscanf")
        .blocklist_function("fy_emit_event_vcreate")
        .blocklist_function("fy_node_create_vscalarf")
        .blocklist_function("fy_node_override_vreport")
        .blocklist_function("fy_node_set_vanchorf")
        .blocklist_function("fy_node_vbuildf")
        .blocklist_function("fy_node_vreport")
        .blocklist_function("fy_node_vscanf")
        .blocklist_function("fy_parse_event_vcreate")
        .blocklist_function("fy_vdiag")
        .prepend_enum_name(false)
        .generate_comments(false)
        .generate()
        .unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_file = Path::new(&out_dir).join("bindings.rs");
    bindings.write_to_file(out_file).unwrap();

    let mut build = cc::Build::new();
    add_c_files(&mut build, Path::new("libfyaml/src/lib"));
    add_c_files(&mut build, Path::new("libfyaml/src/xxhash"));
    build.include("libfyaml/include");
    build.include("libfyaml/src/xxhash");
    build.flag_if_supported("-Wno-type-limits");
    build.flag_if_supported("-Wno-unused-but-set-parameter");
    build.flag_if_supported("-Wno-unused-parameter");
    build.define("VERSION", "NULL");
    build.define("__STDC_WANT_LIB_EXT2__", "1");
    build.compile("libfyaml");
}

fn add_c_files(build: &mut cc::Build, dir: &Path) {
    // Sort the C files to ensure a deterministic build for reproducible builds.
    let iter = dir.read_dir().unwrap();
    let mut paths = iter.collect::<io::Result<Vec<_>>>().unwrap();
    paths.sort_by_key(DirEntry::path);

    for entry in paths {
        if entry.file_type().unwrap().is_file() {
            let path = entry.path();
            if path.extension() == Some(OsStr::new("c")) {
                build.file(path);
            }
        }
    }
}
