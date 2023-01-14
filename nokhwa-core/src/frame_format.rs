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
