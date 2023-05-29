//! [![github]](https://github.com/dtolnay/libfyaml-rs)&ensp;[![crates-io]](https://crates.io/crates/libfyaml-sys)&ensp;[![docs-rs]](https://docs.rs/libfyaml-sys)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

#![doc(html_root_url = "https://docs.rs/libfyaml-sys/0.2.2+fy0.8.0")]
#![allow(
    improper_ctypes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]

#[allow(clippy::all, clippy::pedantic)]
mod bindings {
    use libc::FILE;

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use crate::bindings::{fy_event__bindgen_ty_1 as fy_event_data, *};

// Exclude the following types from being exported out of the bindings module.
#[allow(unknown_lints, dead_code, hidden_glob_reexports)]
struct fy_event__bindgen_ty_1;
#[allow(unknown_lints, dead_code, hidden_glob_reexports)]
struct __BindgenBitfieldUnit;
