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

use crate::pixel_format::{PixelFormat, PixelFormats};
use crate::{FrameFormat, NokhwaError, Resolution};
use image::ImageBuffer;

#[derive(Clone, Debug, Default, Hash, PartialOrd, PartialEq)]
#[cfg_attr("serde", Serialize, Deserialize)]
pub struct Buffer {
    resolution: Resolution,
    buffer: Vec<u8>,
    
}

impl Buffer {
    pub fn new(res: Resolution, buf: Vec<u8>) -> Self {
        Self {
            resolution: res,
            buffer: buf,
        }
    }
    
    pub fn to_image_with_custom_format<I>(self) -> Result<ImageBuffer<I::Output, Vec<u8>>, NokhwaError>
    where
        I: PixelFormat,
    {
        ImageBuffer::from_raw(
            self.resolution.width_x,
            self.resolution.height_y,
            self.buffer,
        ).ok_or(NokhwaError::ProcessFrameError {
            src: ,
            destination: "".to_string(),
            error: "".to_string()
        })
    }
}
