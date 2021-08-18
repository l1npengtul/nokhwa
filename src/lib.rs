/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
#![cfg_attr(feature = "test-fail-warning", deny(warnings))]
#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic)]
#![warn(clippy::all)]

#[cfg(feature = "small-wasm")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Raw access to each of Nokhwa's backends.
#[cfg(not(feature = "small-wasm"))]
pub mod backends;
#[cfg(not(feature = "small-wasm"))]
mod camera;
#[cfg(not(feature = "small-wasm"))]
mod camera_traits;
mod error;
#[cfg(feature = "input-jscam")]
/// A camera that uses native browser APIs meant for WASM applications.
pub mod js_camera;
#[cfg(feature = "input-ipcam")]
/// A camera that uses `OpenCV` to access IP (rtsp/http) on the local network
pub mod network_camera;
#[cfg(not(feature = "small-wasm"))]
mod query;
mod utils;

#[cfg(not(feature = "small-wasm"))]
pub use camera::Camera;
#[cfg(not(feature = "small-wasm"))]
pub use camera_traits::*;
pub use error::NokhwaError;
#[cfg(not(feature = "small-wasm"))]
pub use query::query_devices;
pub use utils::*;
