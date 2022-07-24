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

use crate::pixel_format::PixelFormat;
use crate::{mjpeg_to_rgb, yuyv422_to_rgb, FrameFormat, NokhwaError, Resolution};
use image::{ImageBuffer, Pixel};
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

    pub fn to_image<F: PixelFormat>(self) -> Result<ImageBuffer<F::Output, Vec<u8>>, NokhwaError> {
        let new_data = F::buffer_to_output(self.source_frame_format, &self.buffer)?;
    }

    #[cfg(feature = "input-opencv")]
    pub fn to_opencv_mat(self) -> Result<Mat, NokhwaError> {
        let buffer = match self.source_frame_format {
            FrameFormat::MJPEG => mjpeg_to_rgb(&self.buffer, use_alpha)?,
            FrameFormat::YUYV => yuyv422_to_rgb(&self.buffer, use_alpha)?,
            FrameFormat::GRAY8 => {
                if use_alpha {
                    self.buffer.into_iter().flat_map(|x| [x, 255])
                } else {
                    self.buffer
                }
            }
        };

        Ok(match self.source_frame_format {
            FrameFormat::MJPEG | FrameFormat::YUYV => Mat::from_slice_2d(
                buffer
                    .as_rgb()
                    .chunks(self.resolution.height_y as usize)
                    .collect::<&[&[RGB<u8>]]>(),
            ),
            FrameFormat::GRAY8 => Mat::from_slice_2d(
                buffer
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
