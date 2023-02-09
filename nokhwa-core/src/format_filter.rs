use crate::frame_format::FrameFormat;
use crate::types::{CameraFormat, Resolution};
use std::collections::HashMap;


/// Tells the init function what camera format to pick.
/// - `AbsoluteHighestResolution`: Pick the highest [`Resolution`], then pick the highest frame rate of those provided.
/// - `AbsoluteHighestFrameRate`: Pick the highest frame rate, then the highest [`Resolution`].
/// - `HighestResolution(Resolution)`: Pick the highest [`Resolution`] for the given framerate (the `Option<u32>`).
/// - `HighestFrameRate(u32)`: Pick the highest frame rate for the given [`Resolution`] (the `Option<Resolution>`).
/// - `None`: Pick a random [`CameraFormat`]
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RequestedFormatType {
    AbsoluteHighestResolution,
    AbsoluteHighestFrameRate,
    HighestResolution(Resolution),
    HighestFrameRate(u32),
    None,
}


pub struct FormatFilter {
    filter_pref:
}
