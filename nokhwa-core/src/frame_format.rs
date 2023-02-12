use crate::error::NokhwaError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC`.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameFormat {
    // Compressed Formats
    H265,
    H264,
    H263,
    Avc1,
    Mpeg1,
    Mpeg2,
    Mpeg4,
    MJpeg,
    XVid,
    VP8,
    VP9,

    // YCbCr formats

    // -> 422 16 BPP
    Yuv422,
    Uyv422,

    // 420
    Nv12,
    Nv21,
    Yv12,
    Imc2,
    Imc4,

    // Grayscale Formats
    Luma8,

    // RGB Formats
    Rgb8,
    RgbA8,

    // Custom
    Custom(u128),
}

impl FrameFormat {
    pub const ALL: &'static [FrameFormat] = &[
        FrameFormat::H263,
        FrameFormat::H264,
        FrameFormat::H265,
        FrameFormat::Avc1,
        FrameFormat::Mpeg1,
        FrameFormat::Mpeg2,
        FrameFormat::Mpeg4,
        FrameFormat::MJpeg,
        FrameFormat::XVid,
        FrameFormat::VP8,
        FrameFormat::VP9,
        FrameFormat::Yuv422,
        FrameFormat::Uyv422,
        FrameFormat::Nv12,
        FrameFormat::Nv21,
        FrameFormat::Yv12,
        FrameFormat::Imc2,
        FrameFormat::Imc4,
        FrameFormat::Luma8,
        FrameFormat::Rgb8,
        FrameFormat::RgbA8,
    ];

    pub const COMPRESSED: &'static [FrameFormat] = &[
        FrameFormat::H263,
        FrameFormat::H264,
        FrameFormat::H265,
        FrameFormat::Avc1,
        FrameFormat::Mpeg1,
        FrameFormat::Mpeg2,
        FrameFormat::Mpeg4,
        FrameFormat::MJpeg,
        FrameFormat::XVid,
        FrameFormat::VP8,
        FrameFormat::VP9,
    ];

    pub const CHROMA: &'static [FrameFormat] = &[
        FrameFormat::Yuv422,
        FrameFormat::Uyv422,
        FrameFormat::Nv12,
        FrameFormat::Nv21,
        FrameFormat::Yv12,
        FrameFormat::Imc2,
        FrameFormat::Imc4,
    ];

    pub const LUMA: &'static [FrameFormat] = &[FrameFormat::Luma8];

    pub const RGB: &'static [FrameFormat] = &[FrameFormat::Rgb8, FrameFormat::RgbA8];
}

impl Display for FrameFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

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
use crate::buffer::Buffer;
use crate::types::ApiBackend;
use image::{ImageBuffer, Luma, LumaA, Primitive, Rgb, Rgba};
use std::ops::Deref;

pub trait FormatDecoders: Send + Sync {
    const NAME: &'static str;

    const PRIMARY: &'static [FrameFormat];

    const PLATFORM_ACCEPTABLE: &'static [(ApiBackend, &'static [u128])];

    type Primitive: Primitive;

    type Container: Deref<Target = [Self::Primitive]>;
}

pub trait RgbDecoder: FormatDecoders {
    fn decode_rgb(&self, buffer: &Buffer) -> ImageBuffer<Rgb<Self::Primitive>, Self::Container>;
}

pub trait RgbADecoder: FormatDecoders {
    fn decode_rgba(&self, buffer: &Buffer) -> ImageBuffer<Rgba<Self::Primitive>, Self::Container>;
}

pub trait LumaDecoder: FormatDecoders {
    fn decode_luma(&self, buffer: &Buffer) -> ImageBuffer<Luma<Self::Primitive>, Self::Container>;
}

pub trait LumaADecoder: FormatDecoders {
    fn decode_luma_a(
        &self,
        buffer: &Buffer,
    ) -> ImageBuffer<LumaA<Self::Primitive>, Self::Container>;
}

// TODO: Wgpu Decoder
