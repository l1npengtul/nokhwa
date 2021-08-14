/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
#![cfg_attr(feature = "test-fail-warning", deny(warnings))]
#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]
#![warn(clippy::all)]

/// Raw access to each of Nokhwa's backends.
pub mod backends;
mod camera;
mod camera_traits;
mod error;
#[cfg(feature = "input-jscam")]
/// A camera that uses native browser APIs meant for WASM applications.
pub mod js_camera;
#[cfg(feature = "input-ipcam")]
/// A camera that uses OpenCV to access IP (rtsp/http) on the local network
pub mod network_camera;
mod query;
mod utils;

pub use camera::Camera;
pub use camera_traits::*;
pub use error::NokhwaError;
pub use query::query_devices;
pub use utils::*;
