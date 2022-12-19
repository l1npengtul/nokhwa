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

use crate::{
    error::NokhwaError,
    pixel_format::FormatDecoder,
    types::{FrameFormat, Resolution},
};
use bytes::Bytes;
use image::ImageBuffer;

/// A buffer returned by a camera to accommodate custom decoding.
/// Contains information of Resolution, the buffer's [`FrameFormat`], and the buffer.
///
/// Note that decoding on the main thread **will** decrease your performance and lead to dropped frames.
#[derive(Clone, Debug, Hash, PartialOrd, PartialEq, Eq)]
pub struct Buffer {
    resolution: Resolution,
    buffer: Bytes,
    source_frame_format: FrameFormat,
}

impl Buffer {
    /// Creates a new buffer with a [`&[u8]`].
    #[must_use]
    #[inline]
    pub fn new(res: Resolution, buf: &[u8], source_frame_format: FrameFormat) -> Self {
        Self {
            resolution: res,
            buffer: Bytes::copy_from_slice(buf),
            source_frame_format,
        }
    }

    /// Get the [`Resolution`] of this buffer.
    #[must_use]
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Get the data of this buffer.
    #[must_use]
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Get a owned version of this buffer.
    #[must_use]
    pub fn buffer_bytes(&self) -> Bytes {
        self.buffer.clone()
    }

    /// Get the [`FrameFormat`] of this buffer.
    #[must_use]
    pub fn source_frame_format(&self) -> FrameFormat {
        self.source_frame_format
    }

    /// Decodes a image with allocation using the provided [`FormatDecoder`].
    /// # Errors
    /// Will error when the decoding fails.
    #[inline]
    pub fn decode_image<F: FormatDecoder>(
        &self,
    ) -> Result<ImageBuffer<F::Output, Vec<u8>>, NokhwaError> {
        let new_data = F::write_output(self.source_frame_format, self.resolution, &self.buffer)?;
        let image =
            ImageBuffer::from_raw(self.resolution.width_x, self.resolution.height_y, new_data)
                .ok_or(NokhwaError::ProcessFrameError {
                    src: self.source_frame_format,
                    destination: stringify!(F).to_string(),
                    error: "Failed to create buffer".to_string(),
                })?;
        Ok(image)
    }

    /// Decodes a image with allocation using the provided [`FormatDecoder`] into a `buffer`.
    /// # Errors
    /// Will error when the decoding fails, or the provided buffer is too small.
    #[inline]
    pub fn decode_image_to_buffer<F: FormatDecoder>(
        &self,
        buffer: &mut [u8],
    ) -> Result<(), NokhwaError> {
        F::write_output_buffer(
            self.source_frame_format,
            self.resolution,
            &self.buffer,
            buffer,
        )
    }
    /// Decodes a image with allocation using the provided [`FormatDecoder`] into a [`Mat`](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html).
    ///
    /// Note that this does a clone when creating the buffer, to decouple the lifetime of the internal data to the temporary Buffer. If you want to avoid this, please see [`decode_opencv_mat`](Self::decode_opencv_mat).
    /// # Errors
    /// Will error when the decoding fails, or `OpenCV` failed to create/copy the [`Mat`](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html).
    /// # Safety
    /// This function uses `unsafe` in order to create the [`Mat`](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html). Please see [`Mat::new_rows_cols_with_data`](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html#method.new_rows_cols_with_data) for more.
    ///
    /// Most notably, the `data` **must** stay in scope for the duration of the [`Mat`](https://docs.rs/opencv/latest/opencv/core/struct.Mat.html) or bad, ***bad*** things happen.
    #[cfg(feature = "opencv-mat")]
    #[cfg_attr(feature = "docs-features", doc(cfg(feature = "opencv-mat")))]
    #[allow(clippy::cast_possible_wrap)]
    pub fn decode_opencv_mat<F: FormatDecoder>(
        &mut self,
    ) -> Result<opencv::core::Mat, NokhwaError> {
        use image::Pixel;
        use opencv::core::{Mat, Mat_AUTO_STEP, CV_8UC1, CV_8UC2, CV_8UC3, CV_8UC4};

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
                self.resolution.height_y as i32,
                self.resolution.width_x as i32,
                array_type,
                self.buffer.as_ref().as_ptr().cast_mut().cast(),
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
