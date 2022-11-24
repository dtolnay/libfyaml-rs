#![allow(
    improper_ctypes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]

mod bindings {
    use libc::FILE;

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use crate::bindings::{fy_event__bindgen_ty_1 as fy_event_data, *};

// Exclude the following types from being exported out of the bindings module.
#[allow(dead_code)]
struct fy_event__bindgen_ty_1;
