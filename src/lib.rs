#![deny(clippy::pedantic)]
#![warn(clippy::all)]
#![allow(clippy::module_name_repetitions)]
/*
 * Copyright 2022 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
#![cfg_attr(feature = "test-fail-warning", deny(warnings))]
#![cfg_attr(feature = "docs-features", feature(doc_cfg))]
//! # nokhwa
//! A Simple-to-use, cross-platform Rust Webcam Capture Library
//!
//! The raw backends can be found in [`backends`](crate::backends)
//!
//! The [`Camera`] struct is what you will likely use.
//!
//! Please read the README for more.

/// Raw access to each of Nokhwa's backends.
pub mod backends;
mod camera;
mod init;
/// A camera that uses native browser APIs meant for WASM applications.
#[cfg(feature = "input-jscam")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-jscam")))]
pub mod js_camera;

pub use nokhwa_core::pixel_format::FormatDecoder;
mod query;
/// A camera that runs in a different thread and can call your code based on callbacks.
#[cfg(feature = "output-threaded")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "output-threaded")))]
pub mod threaded;

#[cfg(feature = "input-ipcam")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-ipcam")))]
#[deprecated(
    since = "0.10.0",
    note = "please use `Camera` with `CameraIndex::String` and `input-opencv` enabled."
)]
pub use backends::capture::NetworkCamera;
pub use camera::Camera;
pub use init::*;
#[cfg(feature = "input-jscam")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-jscam")))]
pub use js_camera::JSCamera;
pub use nokhwa_core::buffer::Buffer;
pub use nokhwa_core::error::NokhwaError;
pub use query::*;
#[cfg(feature = "output-threaded")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "output-threaded")))]
pub use threaded::CallbackCamera;

pub mod utils {
    pub use nokhwa_core::types::*;
}

pub mod error {
    pub use nokhwa_core::error::NokhwaError;
}

pub mod camera_traits {
    pub use nokhwa_core::traits::*;
}

pub mod pixel_format {
    pub use nokhwa_core::pixel_format::*;
}

pub mod buffer {
    pub use nokhwa_core::buffer::*;
}
