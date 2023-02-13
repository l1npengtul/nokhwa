use crate::{
    frame_format::FrameFormat,
    types::{ApiBackend, CameraFormat, Resolution},
};
use std::collections::{BTreeMap, BTreeSet};

/// Tells the init function what camera format to pick.
/// - `AbsoluteHighestResolution`: Pick the highest [`Resolution`], then pick the highest frame rate of those provided.
/// - `AbsoluteHighestFrameRate`: Pick the highest frame rate, then the highest [`Resolution`].
/// - `HighestResolution(Resolution)`: Pick the highest [`Resolution`] for the given framerate (the `Option<u32>`).
/// - `HighestFrameRate(u32)`: Pick the highest frame rate for the given [`Resolution`] (the `Option<Resolution>`).
/// - `Exact`: Pick the exact [`CameraFormat`] provided.
/// - `Closest`: Pick the closest [`CameraFormat`] provided in order of [`FrameFormat`], [`Resolution`], and FPS.
/// - `None`: Pick a random [`CameraFormat`]
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RequestedFormatType {
    AbsoluteHighestResolution,
    AbsoluteHighestFrameRate,
    HighestResolution(Resolution),
    HighestFrameRate(u32),
    Exact(CameraFormat),
    Closest(CameraFormat),
    None,
}

/// How you get your [`FrameFormat`] from the
#[derive(Clone, Debug)]
pub struct FormatFilter {
    filter_pref: RequestedFormatType,
    fcc_primary: BTreeSet<FrameFormat>,
    fcc_platform: BTreeMap<ApiBackend, BTreeSet<u128>>,
}

impl FormatFilter {
    pub fn new(fmt_type: RequestedFormatType) -> Self {
        Self {
            filter_pref: fmt_type,
            fcc_primary: Default::default(),
            fcc_platform: Default::default(),
        }
    }

    pub fn add_allowed_frame_format(&mut self, frame_format: FrameFormat) {
        self.fcc_primary.insert(frame_format);
    }

    pub fn add_allowed_frame_format_many(&mut self, frame_formats: impl AsRef<[FrameFormat]>) {
        self.fcc_primary.extend(frame_formats.as_ref().iter());
    }

    pub fn add_allowed_platform_specific(&mut self, platform: ApiBackend, frame_format: u128) {
        match self.fcc_platform.get_mut(&platform) {
            Some(fccs) => {
                fccs.insert(frame_format);
            }
            None => {
                self.fcc_platform
                    .insert(platform, BTreeSet::from([frame_format]));
            }
        };
    }

    pub fn add_allowed_platform_specific_many(
        &mut self,
        platform_specifics: impl AsRef<[(ApiBackend, u128)]>,
    ) {
        for (platform, frame_format) in platform_specifics.as_ref().into_iter() {
            match self.fcc_platform.get_mut(&platform) {
                Some(fccs) => {
                    fccs.insert(*frame_format);
                }
                None => {
                    self.fcc_platform
                        .insert(*platform, BTreeSet::from([*frame_format]));
                }
            };
        }
    }

    pub fn with_allowed_frame_format(mut self, frame_format: FrameFormat) -> Self {
        self.fcc_primary.insert(frame_format);
        self
    }

    pub fn with_allowed_frame_format_many(
        mut self,
        frame_formats: impl AsRef<[FrameFormat]>,
    ) -> Self {
        self.fcc_primary.extend(frame_formats.as_ref().iter());
        self
    }

    pub fn with_allowed_platform_specific(
        mut self,
        platform: ApiBackend,
        frame_format: u128,
    ) -> Self {
        self.add_allowed_platform_specific(platform, frame_format);
        self
    }

    pub fn with_allowed_platform_specific_many(
        mut self,
        platform_specifics: impl AsRef<[(ApiBackend, u128)]>,
    ) -> Self {
        self.add_allowed_platform_specific_many(platform_specifics);
        self
    }
}

impl Default for FormatFilter {
    fn default() -> Self {
        Self {
            filter_pref: RequestedFormatType::Closest(CameraFormat::new(
                Resolution::new(640, 480),
                FrameFormat::Yuv422,
                30,
            )),
            fcc_primary: BTreeSet::from([FrameFormat::Yuv422]),
            fcc_platform: Default::default(),
        }
    }
}

pub struct FormatFulfill {}
