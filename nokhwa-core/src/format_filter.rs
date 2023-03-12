use crate::frame_format::SourceFrameFormat;
use crate::{
    frame_format::FrameFormat,
    types::{ApiBackend, CameraFormat, Resolution},
};
use std::collections::{BTreeMap, BTreeSet};

/// Tells the init function what camera format to pick.
/// - `AbsoluteHighestResolution`: Pick the highest [`Resolution`], then pick the highest frame rate of those provided.
/// - `AbsoluteHighestFrameRate`: Pick the highest frame rate, then the highest [`Resolution`].
/// - `HighestResolution(Resolution)`: Pick the highest [`Resolution`] for the given framerate.
/// - `HighestFrameRate(u32)`: Pick the highest frame rate for the given [`Resolution`].
/// - `Exact`: Pick the exact [`CameraFormat`] provided.
/// - `Closest`: Pick the closest [`CameraFormat`] provided in order of [`Resolution`], and FPS.
/// - `ClosestGreater`: Pick the closest [`CameraFormat`] provided in order of  [`Resolution`], and FPS. The returned format's [`Resolution`] **and** FPS will be **greater than or equal to** the provided [`CameraFormat`]
/// - `ClosestLess`: Pick the closest [`CameraFormat`] provided in order of [`Resolution`], and FPS.The returned format's [`Resolution`] **and** FPS will be **less than or equal to** the provided [`CameraFormat`]
/// - `None`: Pick a random [`CameraFormat`]
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum RequestedFormatType {
    AbsoluteHighestResolution,
    AbsoluteHighestFrameRate,
    HighestResolution(u32),
    HighestFrameRate(Resolution),
    Exact(CameraFormat),
    ClosestGreater(CameraFormat),
    ClosestLess(CameraFormat),
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
                FrameFormat::Yuv422.into(),
                30,
            )),
            fcc_primary: BTreeSet::from([FrameFormat::Yuv422]),
            fcc_platform: Default::default(),
        }
    }
}

fn format_fulfill(
    sources: impl AsRef<[CameraFormat]>,
    filter: FormatFilter,
) -> Option<CameraFormat> {
    let mut sources = sources
        .as_ref()
        .into_iter()
        .filter(|cam_filter| match cam_filter.format() {
            SourceFrameFormat::FrameFormat(fmt) => filter.fcc_primary.contains(&fmt),
            SourceFrameFormat::PlatformSpecific(plat) => filter
                .fcc_platform
                .get(&plat.backend())
                .map(|x| x.contains(&plat.format()))
                .unwrap_or(false),
        });

    match filter.filter_pref {
        RequestedFormatType::AbsoluteHighestResolution => {
            let mut sources = sources.collect::<Vec<&CameraFormat>>();
            sources.sort_by(|a, b| a.resolution().cmp(&b.resolution()));
            sources.last().copied().copied()
        }
        RequestedFormatType::AbsoluteHighestFrameRate => {
            let mut sources = sources.collect::<Vec<&CameraFormat>>();
            sources.sort_by(|a, b| a.frame_rate().cmp(&b.frame_rate()));
            sources.last().copied().copied()
        }
        RequestedFormatType::HighestResolution(filter_fps) => {
            let mut sources = sources
                .filter(|format| format.frame_rate() == filter_fps)
                .collect::<Vec<&CameraFormat>>();
            sources.sort();
            sources.last().copied().copied()
        }
        RequestedFormatType::HighestFrameRate(filter_res) => {
            let mut sources = sources
                .filter(|format| format.resolution() == filter_res)
                .collect::<Vec<&CameraFormat>>();
            sources.sort();
            sources.last().copied().copied()
        }
        RequestedFormatType::Exact(exact) => {
            sources.filter(|format| format == &&exact).last().copied()
        }
        RequestedFormatType::Closest(closest) => {
            let mut sources = sources
                .map(|format| {
                    let dist = distance_3d_camerafmt_relative(closest, *format);
                    (dist, *format)
                })
                .collect::<Vec<(f64, CameraFormat)>>();
            sources.sort_by(|a, b| a.0.total_cmp(&b.0));
            sources.first().copied().map(|(_, cf)| cf)
        }
        RequestedFormatType::ClosestGreater(closest) => {
            let mut sources = sources
                .filter(|format| {
                    format.resolution() >= closest.resolution()
                        && format.frame_rate() >= closest.frame_rate()
                })
                .map(|format| {
                    let dist = distance_3d_camerafmt_relative(closest, *format);
                    (dist, *format)
                })
                .collect::<Vec<(f64, CameraFormat)>>();
            sources.sort_by(|a, b| a.0.total_cmp(&b.0));
            sources.first().copied().map(|(_, cf)| cf)
        }
        RequestedFormatType::ClosestLess(closest) => {
            let mut sources = sources
                .filter(|format| {
                    format.resolution() <= closest.resolution()
                        && format.frame_rate() <= closest.frame_rate()
                })
                .map(|format| {
                    let dist = distance_3d_camerafmt_relative(closest, *format);
                    (dist, *format)
                })
                .collect::<Vec<(f64, CameraFormat)>>();
            sources.sort_by(|a, b| a.0.total_cmp(&b.0));
            sources.first().copied().map(|(_, cf)| cf)
        }
        RequestedFormatType::None => sources.nth(0).map(|x| *x),
    }
}

fn distance_3d_camerafmt_relative(a: CameraFormat, b: CameraFormat) -> f64 {
    let res_x_diff = b.resolution().x() - a.resolution().x();
    let res_y_diff = b.resolution().y() - a.resolution().y();
    let fps_diff = b.frame_rate() - a.frame_rate();

    let x = res_x_diff.pow(2) as f64;
    let y = res_y_diff.pow(2) as f64;
    let z = fps_diff.pow(2) as f64;

    x + y + z
}
