#![deny(clippy::pedantic)]
#![warn(clippy::all)]

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
//! ## Feature
//! The default feature includes nothing. Anything starting with `input-*` is a feature that enables the specific backend.
//!
//! `input-*` features:
//!  - `input-v4l`: Enables the `Video4Linux` backend (linux)
//!  - `input-uvc`: Enables the `libuvc` backend (cross-platform, libuvc statically-linked)
//!  - `input-opencv`: Enables the `opencv` backend (cross-platform)
//!  - `input-ipcam`: Enables the use of IP Cameras, please see the `NetworkCamera` struct. Note that this relies on `opencv`, so it will automatically enable the `input-opencv` feature.
//!
//! Conversely, anything that starts with `output-*` controls a feature that controls the output of something (usually a frame from the camera)
//!
//! `output-*` features:
//!  - `output-wgpu`: Enables the API to copy a frame directly into a `wgpu` texture.
//!
//! You many want to pick and choose to reduce bloat.
//! ## Example
//! ```rust
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

/// Raw access to each of Nokhwa's backends.
pub mod backends;
mod camera;
mod camera_traits;
mod error;
#[cfg(feature = "input-ipcam")]
mod network_camera;
mod query;
mod utils;
#[macro_use]
mod macros;

pub use camera::Camera;
pub use camera_traits::*;
pub use error::NokhwaError;
#[cfg(feature = "input-ipcam")]
pub use network_camera::NetworkCamera;
pub use utils::*;
