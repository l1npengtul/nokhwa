/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! # nokhwa
//! Nokhwa(녹화): Korean word meaning "to record".
//!
//! A Simple-to-use, cross-platform Rust Webcam Capture Library
//!
//! ## Using nokhwa
//! You will need the latest stable Rust and Cargo.
//!
//! Nokhwa can be added to your crate by adding it to your `Cargo.toml`:
//! ```.ignore
//! [dependencies.nokhwa]
//! // TODO: replace the "*" with the latest version of `nokhwa`
//! version = "*"
//! // TODO: add some features
//! features = [""]
//! ```
//!
//! Most likely, you will only use functionality provided by the `Camera` struct. If you need lower-level access, you may instead opt to use the raw capture backends found at `nokhwa::backends::capture::*`.
//! ## API Support
//! The table below lists current Nokhwa API support.
//! - The `Backend` column signifies the backend.
//! - The `Input` column signifies reading frames from the camera
//! - The `Query` column signifies system device list support
//! - The `Query-Device` column signifies reading device capabilities
//! - The `Platform` column signifies what Platform this is availible on.
//!
//! | Backend                             | Input              | Query              | Query-Device       | Platform            |
//! |-------------------------------------|--------------------|--------------------|--------------------|---------------------|
//! | `Video4Linux`(`input-v4l`)          | YES                | YES                | YES                | Linux               |
//! | `libuvc`(`input-uvc`)               | YES                | YES                | YES                | Linux, Windows, Mac |
//! | `OpenCV`(`input-opencv`)^           | YES                | NO                 | NO                 | Linux, Windows, Mac |
//! | `IPCamera`(`input-ipcam`/`OpenCV`)^ | YES                | NO                 | NO                 | Linux, Windows, Mac |
//! | `GStreamer`(`input-gst`)^           | YES                | YES                | YES                | Linux, Windows, Mac |
//! | `FFMpeg`                            |        *           |         *          |         *          | Linux, Windows, Mac |
//! | `AVFoundation`                      |        *           |         *          |         *          | Mac                 |
//! | MSMF                                |        *           |         *          |         *          | Windows             |
//! | JS/WASM                             |        *           |         *          |         *          | Web                 |
//!  
//!  *: Planned/WIP
//!
//!
//!  ^ = No CameraFormat setting support.
//!
//! ## Feature
//! The default feature includes nothing. Anything starting with `input-*` is a feature that enables the specific backend.
//! As a general rule of thumb, you would want to keep at least `input-uvc` or other backend that has querying enabled so you can get device information from `nokhwa`.
//!
//! `input-*` features:
//! - `input-v4l`: Enables the `Video4Linux` backend (linux)
//! - `input-uvc`: Enables the `libuvc` backend (cross-platform, libuvc statically-linked)
//! - `input-opencv`: Enables the `opencv` backend (cross-platform)
//! - `input-ipcam`: Enables the use of IP Cameras, please see the `NetworkCamera` struct. Note that this relies on `opencv`, so it will automatically enable the `input-opencv` feature.
//! - `input-gst`: Enables the `gstreamer` backend (cross-platform).
//!
//! Conversely, anything that starts with `output-*` controls a feature that controls the output of something (usually a frame from the camera)
//!
//! `output-*` features:
//!  - `output-wgpu`: Enables the API to copy a frame directly into a `wgpu` texture.
//!
//! You many want to pick and choose to reduce bloat.
//! ## Example
//! ```.ignore
//! // set up the Camera
//! let mut camera = Camera::new(
//!     0, // index
//!     Some(CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30)), // format
//!     CaptureAPIBackend::AUTO, // what backend to use (let nokhwa decide for itself)
//! )
//! .unwrap();
//! // open stream
//! camera.open_stream().unwrap();
//! loop {
//!     let frame = camera.get_frame().unwrap();
//!     println!("{}, {}", frame.width(), frame.height());
//! }
//! ```
//! They can be found in the `examples` folder.

#![deny(clippy::pedantic)]
#![warn(clippy::all)]
#![allow(clippy::must_use_candidate)]

/// Raw access to each of Nokhwa's backends.
pub mod backends;
mod camera;
mod camera_traits;
mod error;
#[cfg(feature = "input-ipcam")]
mod network_camera;
mod query;
mod utils;

pub use camera::Camera;
pub use camera_traits::*;
pub use error::NokhwaError;
#[cfg(feature = "input-ipcam")]
pub use network_camera::NetworkCamera;
pub use query::query_devices;
pub use utils::*;
