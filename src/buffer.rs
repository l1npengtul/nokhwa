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

use crate::pixel_format::{PixelFormat};
use crate::{FrameFormat, NokhwaError, Resolution};
use image::ImageBuffer;
#[cfg(feature = "input-opencv")]
use opencv::core::Mat;
#[cfg(feature = "input-opencv")]
use rgb::{FromSlice, RGB};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Hash, PartialOrd, PartialEq)]
#[cfg_attr(feature = "serde", Serialize, Deserialize)]
pub struct Buffer {
    resolution: Resolution,
    buffer: Vec<u8>,
    source_frame_format: FrameFormat,
}

impl Buffer {
    pub fn new(res: Resolution, buf: Vec<u8>, source_frame_format: FrameFormat) -> Self {
        Self {
            resolution: res,
            buffer: buf,
            source_frame_format,
        }
    }

    pub fn to_image_with_custom_format<F>(
        self,
    ) -> Result<ImageBuffer<F::Output, Vec<u8>>, NokhwaError>
    where
        F: PixelFormat,
    {
        if self.source_frame_format != F::CODE {
            return Err(NokhwaError::ProcessFrameError {
                src: self.source_frame_format,
                destination: F::CODE.to_string(),
                error: "Assertion failed, wrong source!".to_string(),
            });
        }
        ImageBuffer::from_raw(
            self.resolution.width_x,
            self.resolution.height_y,
            self.buffer,
        )
        .ok_or(NokhwaError::ProcessFrameError {
            src: F::CODE,
            destination: stringify!(I::Output).to_string(),
            error: "Buffer too small".to_string(),
        })
    }

    #[cfg(feature = "input-opencv")]
    pub fn to_opencv_mat(self) -> Result<Mat, NokhwaError> {
        Ok(match self.source_frame_format {
            FrameFormat::MJPEG | FrameFormat::YUYV => Mat::from_slice_2d(
                self.buffer
                    .as_rgb()
                    .chunks(self.resolution.height_y as usize)
                    .collect::<&[&[RGB<u8>]]>(),
            ),
            FrameFormat::GRAY8 => Mat::from_slice_2d(
                self.buffer
                    .chunks(self.resolution.height_y as usize)
                    .collect::<&[&[u8]]>(),
            ),
        }
        .map_err(|why| NokhwaError::ProcessFrameError {
            src: self.source_frame_format,
            destination: "OpenCV Mat".to_string(),
            error: why.to_string(),
        })?)
    }
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
    pub fn source_frame_format(&self) -> FrameFormat {
        self.source_frame_format
    }
}
