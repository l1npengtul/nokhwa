/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
#![cfg_attr(feature = "test-fail-warning", deny(warnings))]

//! # nokhwa
//! A

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
pub mod js_camera;
/// A camera that uses `OpenCV` to access IP (rtsp/http) on the local network
#[cfg(feature = "input-ipcam")]
pub mod network_camera;
mod query;
/// A camera that runs in a different thread and can call your code based on callbacks.
#[cfg(feature = "output-threaded")]
mod threaded;
mod utils;

pub use camera::Camera;
pub use camera_traits::*;
pub use error::NokhwaError;
pub use init::*;
pub use query::*;
#[cfg(feature = "output-threaded")]
pub use threaded::ThreadedCamera;
pub use utils::*;
