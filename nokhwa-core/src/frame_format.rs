use crate::error::NokhwaError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Describes a frame format (i.e. how the bytes themselves are encoded). Often called `FourCC`.
/// - YUYV is a mathematical color space. You can read more [here.](https://en.wikipedia.org/wiki/YCbCr)
/// - NV12 is same as above. Note that a partial compression (e.g. [16, 235] may be coerced to [0, 255].
/// - MJPEG is a motion-jpeg compressed frame, it allows for high frame rates.
/// - GRAY is a grayscale image format, usually for specialized cameras such as IR Cameras.
/// - RAWRGB is a Raw RGB888 format.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameFormat {
    /// - YUYV is a mathematical color space. You can read more [here.](https://en.wikipedia.org/wiki/YCbCr)
    YUYV,
    /// - NV12 is same as above. Note that a partial compression (e.g. [16, 235] may be coerced to [0, 255].
    NV12,
    /// - MJPEG is a motion-jpeg compressed frame, it allows for high frame rates.
    MJPEG,
    /// - GRAY is a grayscale image format, usually for specialized cameras such as IR Cameras.
    GRAY,
    /// - RAWRGB is a Raw RGB888 format.
    RAWRGB,
}

impl Display for FrameFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameFormat::MJPEG => {
                write!(f, "MJPEG")
            }
            FrameFormat::YUYV => {
                write!(f, "YUYV")
            }
            FrameFormat::GRAY => {
                write!(f, "GRAY")
            }
            FrameFormat::RAWRGB => {
                write!(f, "RAWRGB")
            }
            FrameFormat::NV12 => {
                write!(f, "NV12")
            }
        }
    }
}
impl FromStr for FrameFormat {
    type Err = NokhwaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MJPEG" => Ok(FrameFormat::MJPEG),
            "YUYV" => Ok(FrameFormat::YUYV),
            "GRAY" => Ok(FrameFormat::GRAY),
            "RAWRGB" => Ok(FrameFormat::RAWRGB),
            "NV12" => Ok(FrameFormat::NV12),
            _ => Err(NokhwaError::StructureError {
                structure: "FrameFormat".to_string(),
                error: format!("No match for {s}"),
            }),
        }
    }
}

/// Returns all the frame formats
#[must_use]
pub const fn frame_formats() -> &'static [FrameFormat] {
    &[
        FrameFormat::MJPEG,
        FrameFormat::YUYV,
        FrameFormat::NV12,
        FrameFormat::GRAY,
        FrameFormat::RAWRGB,
    ]
}

/// Returns all the color frame formats
#[must_use]
pub const fn color_frame_formats() -> &'static [FrameFormat] {
    &[
        FrameFormat::MJPEG,
        FrameFormat::YUYV,
        FrameFormat::NV12,
        FrameFormat::RAWRGB,
    ]
}
