#![deny(clippy::pedantic)]
#![warn(clippy::all)]
#![allow(clippy::must_use_candidate)]

use std::ffi::OsString;
use thiserror::Error;

#[macro_use]
extern crate lazy_static;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::pub_enum_variant_names)]
#[derive(Error, Debug, Clone)]
pub enum BindingError {
    #[error("Failed to set GUID: {0}")]
    GUIDSetError(String),
    #[error("Attribute Error: {0}")]
    AttributeError(String),
    #[error("Failed to enumerate: {0}")]
    EnumerateError(String),
    #[error("Failed to open device: {0}")]
    DeviceOpenFailError(String),
    #[error("Failed to query device: {0}")]
    DeviceQueryError(String),
}

#[cfg(all(windows, not(feature = "docs-only")))]
include!(concat!(env!("OUT_DIR"), "/uvc_bindings.rs"));
