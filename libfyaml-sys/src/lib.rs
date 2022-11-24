#![allow(
    improper_ctypes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]

use libc::FILE;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
