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

// #[cfg(feature = "input-ipcam")]
// #[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-ipcam")))]
// #[deprecated(
//     since = "0.10.0",
//     note = "please use `Camera` with `CameraIndex::String` and `input-opencv` enabled."
// )]
// pub use backends::capture::NetworkCamera;
pub use camera::Camera;
pub use init::*;
pub use nokhwa_core::buffer::Buffer;
pub use nokhwa_core::error::NokhwaError;
pub use query::*;
#[cfg(feature = "output-threaded")]
#[cfg_attr(feature = "docs-features", doc(cfg(feature = "output-threaded")))]
pub use threaded::CallbackCamera;

pub mod utils {
    pub use nokhwa_core::types::*;
    #[cfg(feature = "input-opencv")]
    mod opencv_int {
        use image::Pixel;
        use nokhwa_core::{
            error::NokhwaError,
            pixel_format::FormatDecoder,
            types::{FrameFormat, Resolution},
        };
        use opencv::core::{Mat, Mat_AUTO_STEP, CV_8UC1, CV_8UC2, CV_8UC3, CV_8UC4};

        /// Decodes a image with allocation using the provided [`FormatDecoder`] into a [`Mat`](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html).
        ///
        /// Note that this does a clone when creating the buffer, to decouple the lifetime of the internal data to the temporary Buffer. If you want to avoid this, please see [`decode_as_opencv_mat`](Self::decode_as_opencv_mat).
        /// # Errors
        /// Will error when the decoding fails, or `OpenCV` failed to create/copy the [`Mat`](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html).
        /// # Safety
        /// This function uses `unsafe` in order to create the [`Mat`](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html). Please see [`Mat::new_rows_cols_with_data`](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html#method.new_rows_cols_with_data) for more.
        ///
        /// Most notably, the `data` **must** stay in scope for the duration of the [`Mat`](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html) or bad, ***bad*** things happen.
        #[cfg(feature = "input-opencv")]
        #[cfg_attr(feature = "docs-features", doc(cfg(feature = "input-opencv")))]
        pub fn decode_opencv_mat<F: FormatDecoder>(
            resolution: Resolution,
            data: &mut impl AsRef<[u8]>,
        ) -> Result<Mat, NokhwaError> {
            let array_type = match F::Output::CHANNEL_COUNT {
                1 => CV_8UC1,
                2 => CV_8UC2,
                3 => CV_8UC3,
                4 => CV_8UC4,
                _ => {
                    return Err(NokhwaError::ProcessFrameError {
                        src: FrameFormat::RAWRGB,
                        destination: "OpenCV Mat".to_string(),
                        error: "Invalid Decoder FormatDecoder Channel Count".to_string(),
                    })
                }
            };

            unsafe {
                // TODO: Look into removing this unnecessary copy.
                let mat1 = Mat::new_rows_cols_with_data(
                    resolution.height_y as i32,
                    resolution.width_x as i32,
                    array_type,
                    data.as_ref().as_mut_ptr().cast(),
                    Mat_AUTO_STEP,
                )
                .map_err(|why| NokhwaError::ProcessFrameError {
                    src: FrameFormat::RAWRGB,
                    destination: "OpenCV Mat".to_string(),
                    error: why.to_string(),
                })?;

                Ok(mat1)
            }
        }
    }
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
