#![allow(clippy::let_underscore_untyped, clippy::uninlined_format_args)]

use git2::{DescribeFormatOptions, DescribeOptions, Repository};
use regex::Regex;
use semver::Version;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::{DirEntry, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{self, Command};

fn generate_new_version(last_version: &str, commit_sha: &str) -> Result<String, String> {
    // sanitize version string (add a patch number if missing)
    let mut version = last_version.to_string();
    let parts = last_version.split('.').collect::<Vec<&str>>();
    // add ".0" one or 2 times
    for _ in 0..(3 - parts.len()) {
        version.push_str(".0");
    }

    let mut version = Version::parse(&version)
        .map_err(|e| format!("Failed to parse {:?}: {}", last_version, e.to_string()))?;

    version.patch += 1;

    version.pre = semver::Prerelease::new("alpha")
        .map_err(|e| format!("Failed to add pre-release: {}", e))?;
    version.build = semver::BuildMetadata::new(commit_sha)
        .map_err(|e| format!("Failed to add build-metadata: {}", e))?;

    Ok(version.to_string())
}

fn latest_git_tag(repo_path: &str) -> Result<(String, Option<String>), Box<dyn Error>> {
    let repo = Repository::open(repo_path)?;

    let mut describe_options = DescribeOptions::new();
    describe_options.describe_tags();

    let mut format_options = DescribeFormatOptions::new();
    format_options.abbreviated_size(7); // Full tag name

    let describe = repo.describe(&describe_options)?;
    let result = describe.format(Some(&format_options))?;
    let result = result.trim_start_matches('v');
    let result: Vec<&str> = result.split('-').collect();
    // if there's more than one part, it's a post-release
    if result.len() > 1 {
        Ok((result[0].to_string(), Some(result[2].to_string())))
    } else {
        Ok((result[0].to_string(), None))
    }
}
fn function_names(header_path: &str) -> impl Iterator<Item = String> {
    let file = File::open(header_path).expect("Unable to open file");
    let reader = BufReader::new(file);
    let re = Regex::new(r"^([a-zA-Z].*\s+)?(fy_[a-zA-Z_][a-zA-Z0-9_]+)\([^)]").unwrap();

    reader.lines().filter_map(move |line| {
        line.ok().and_then(|l| {
            re.captures(&l).and_then(|cap| {
                cap.get(2).map(|m| m.as_str().to_string()) // Convert to owned String
            })
        })
    })
}
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

    let mut bindings = bindgen::builder()
        .header(header)
        .allowlist_recursively(false)
        .allowlist_function("fy_.*")
        .allowlist_type("fy_.*");

    // Variadic functions that use `va_list`.
    // Blocked on https://github.com/rust-lang/rust/issues/44930.
    let all_function_names = function_names("libfyaml/include/libfyaml.h");

    let re =
        Regex::new(r"^.*_v(report|log|printf|buildf|scanf|anchorf|log|event|scalarf|create|diag)$")
            .unwrap();
    for function_name in all_function_names.filter(|name| re.is_match(&name)) {
        bindings = bindings.blocklist_function(function_name);
    }

    let bindings = bindings
        .prepend_enum_name(false)
        .generate_comments(false)
        .formatter(bindgen::Formatter::Prettyplease)
        .generate()
        .unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_file = Path::new(&out_dir).join("bindings.rs");
    bindings.write_to_file(out_file).unwrap();

    let mut build = cc::Build::new();

    let build_dirs = vec!["lib", "xxhash", "util"];
    let build_dirs = build_dirs
        .into_iter()
        .map(|d| format!("libfyaml/src/{}", d))
        .map(|d| PathBuf::from(d))
        .filter(|d| d.exists() && d.is_dir());
    for build_dir in build_dirs {
        add_c_files(&mut build, &build_dir);
        build.include(build_dir.to_str().unwrap());
    }

    build.include("libfyaml/include");
    build.flag_if_supported("-Wno-type-limits");
    build.flag_if_supported("-Wno-unused-but-set-parameter");
    build.flag_if_supported("-Wno-unused-parameter");

    let (version, commit) = latest_git_tag("libfyaml").unwrap();
    if let Some(commit) = commit {
        let new_version = generate_new_version(&version, &commit).unwrap();
        build.define("VERSION", format!("{:?}", new_version).as_str());
    } else {
        build.define("VERSION", format!("{:?}", version).as_str());
    }
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
                println!("cargo:rerun-if-changed={}", path.display());
                build.file(path);
            }
        }
    }
}
