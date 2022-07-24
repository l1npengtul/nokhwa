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
    buf_mjpeg_to_rgb, buf_yuyv422_to_rgb, mjpeg_to_rgb, yuyv422_to_rgb, yuyv444_to_rgba,
    FrameFormat, NokhwaError,
};
use image::{Luma, LumaA, Primitive};
use image::{Pixel, Rgb, Rgba};
use std::{fmt::Debug, hash::Hash};

pub trait PixelFormat: Copy + Clone + Debug + Default + Hash + Send + Sync {
    type Output: Pixel;

    fn write_output(fcc: FrameFormat, data: &[u8]) -> Result<Vec<u8>, NokhwaError>;

    fn write_output_buffer(
        fcc: FrameFormat,
        data: &[u8],
        dest: &mut [u8],
    ) -> Result<(), NokhwaError>;
}

#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct RgbFormat;

impl PixelFormat for RgbFormat {
    type Output = Rgb<u8>;

    fn write_output(fcc: FrameFormat, data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => mjpeg_to_rgb(data, false),
            FrameFormat::YUYV => yuyv422_to_rgb(data, false),
            FrameFormat::GRAY8 => data
                .into_iter()
                .flat_map(|x| {
                    let pxv = *x;
                    [pxv, pxv, pxv]
                })
                .collect(),
        }
    }

    fn write_output_buffer(
        fcc: FrameFormat,
        data: &[u8],
        dest: &mut [u8],
    ) -> Result<(), NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => buf_mjpeg_to_rgb(data, dest, false),
            FrameFormat::YUYV => buf_yuyv422_to_rgb(data, dest, false),
            FrameFormat::GRAY8 => {}
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct RgbaFormat;

impl PixelFormat for RgbaFormat {
    type Output = Rgba<u8>;

    fn buffer_to_output(fcc: FrameFormat, data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => mjpeg_to_rgb(data, true),
            FrameFormat::YUYV => yuyv422_to_rgb(data, true),
            FrameFormat::GRAY8 => data
                .into_iter()
                .flat_map(|x| {
                    let pxv = *x;
                    [pxv, pxv, pxv, 255]
                })
                .collect(),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct LumaFormat;

impl PixelFormat for LumaFormat {
    type Output = Luma<u8>;

    fn buffer_to_output(fcc: FrameFormat, data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => Ok(mjpeg_to_rgb(data, false)?
                .as_slice()
                .chunks_exact(3)
                .flat_map(|x| {
                    let mut avg = 0;
                    x.into_iter().for_each(|v| avg += v as u16);
                    (avg / 3) as u8
                })
                .collect()),
            FrameFormat::YUYV => Ok(yuyv422_to_rgb(data, false)?
                .as_slice()
                .chunks_exact(3)
                .flat_map(|x| {
                    let mut avg = 0;
                    x.into_iter().for_each(|v| avg += v as u16);
                    (avg / 3) as u8
                })
                .collect()),
            FrameFormat::GRAY8 => data.to_vec(),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct LumaAFormat;

impl PixelFormat for LumaAFormat {
    type Output = LumaA<u8>;

    fn buffer_to_output(fcc: FrameFormat, data: &[u8]) -> Result<Vec<u8>, NokhwaError> {
        match fcc {
            FrameFormat::MJPEG => Ok(mjpeg_to_rgb(data, false)?
                .as_slice()
                .chunks_exact(3)
                .flat_map(|x| {
                    let mut avg = 0;
                    x.into_iter().for_each(|v| avg += v as u16);
                    [(avg / 3) as u8, 255]
                })
                .collect()),
            FrameFormat::YUYV => Ok(yuyv422_to_rgb(data, false)?
                .as_slice()
                .chunks_exact(3)
                .flat_map(|x| {
                    let mut avg = 0;
                    x.into_iter().for_each(|v| avg += v as u16);
                    [(avg / 3) as u8, 255]
                })
                .collect()),
            FrameFormat::GRAY8 => data.into_iter().flat_map(|x| [*x, 255]).collect(),
        }
    }
}
