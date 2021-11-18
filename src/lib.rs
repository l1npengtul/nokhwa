/*
 * Copyright 2021 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
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

#[cfg(feature = "small-wasm")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Raw access to each of Nokhwa's backends.
pub mod backends;
mod camera;
mod camera_traits;
mod error;
mod init;
/// A camera that uses native browser APIs meant for WASM applications.
#[cfg(feature = "input-jscam")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-jscam")))]
pub mod js_camera;
/// A camera that uses `OpenCV` to access IP (rtsp/http) on the local network
#[cfg(feature = "input-ipcam")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-ipcam")))]
pub mod network_camera;
mod query;
/// A camera that runs in a different thread and can call your code based on callbacks.
#[cfg(feature = "output-threaded")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "output-threaded")))]
mod threaded;
mod utils;

pub use camera::Camera;
pub use camera_traits::*;
pub use error::NokhwaError;
pub use init::*;
#[cfg(feature = "input-jscam")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-jscam")))]
pub use js_camera::JSCamera;
#[cfg(feature = "input-ipcam")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-ipcam")))]
pub use network_camera::NetworkCamera;
pub use query::*;
#[cfg(feature = "output-threaded")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "output-threaded")))]
pub use threaded::ThreadedCamera;
pub use utils::*;
