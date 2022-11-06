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

/// A buffer returned by a camera to accomodate custom decoding.
/// Contains information of Resolution, the buffer's [`FrameFormat`], and the buffer.
#[derive(Clone, Debug, Hash, PartialOrd, PartialEq, Eq)]
pub struct Buffer {
    resolution: Resolution,
    buffer: Bytes,
    source_frame_format: FrameFormat,
}

impl Buffer {
    /// Creates a new buffer with a [`&[u8]`].
    #[must_use]
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
}
