use std::{
    cmp::Ordering,
    collections::VecDeque
};
use crate::{
    frame_format::FrameFormat,
    types::{CameraFormat, Resolution, FrameRate, Range},
    traits::Distance
};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
enum ClosestType {
    Resolution,
    FrameRate,
    Both,
    None,
}

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum CustomFormatRequestType {
    HighestFrameRate,
    HighestResolution,
    Closest,
    Exact,
}

pub enum FormatRequest {
    Closest {
        resolution: Option<Range<Resolution>>,
        frame_rate: Option<Range<FrameRate>>,
        frame_format: Vec<FrameFormat>,
    },
    HighestFrameRate {
        frame_rate: Range<FrameRate>,
        frame_format: Vec<FrameFormat>,
    },
    HighestResolution {
        resolution: Range<Resolution>,
        frame_format: Vec<FrameFormat>,
    },
    Exact {
        resolution: Resolution,
        frame_rate: FrameRate,
        frame_format: Vec<FrameFormat>,
    },
}

impl FormatRequest {
    #[must_use]
    pub fn resolve(&self, list_of_formats: &[CameraFormat]) -> Option<CameraFormat> {
        if list_of_formats.is_empty() {
            return None;
        }

        match self {
            FormatRequest::Closest { resolution, frame_rate, frame_format } => {
                let resolution_point = resolution.map(|x| x.preferred())?;

                let frame_rate_point = frame_rate.map(|x| x.preferred())?;
                // lets calcuate distance in 3 dimensions (add both resolution and frame_rate together)

                let mut distances = list_of_formats.iter()
                    .filter(|x| {
                        frame_format.contains(&x.format())
                    })
                    .map(|fmt| {
                        ((fmt.frame_rate() - frame_rate_point).abs() + fmt.resolution().distance_from(&resolution_point) as f32, fmt)
                    })
                    .collect::<Vec<_>>();
                distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
                VecDeque::from(distances).pop_front().map(|x| x.1).copied()
            }
            FormatRequest::HighestFrameRate { frame_rate, frame_format } => {
                let mut formats = list_of_formats.iter().filter(|x| {
                    frame_format.contains(&x.format()) && frame_rate.in_range(x.frame_rate())
                }).collect::<Vec<_>>();
                formats.sort_by(|a, b| {
                    match a.frame_rate().partial_cmp(&b.frame_rate()) {
                        None | Some(Ordering::Equal) => a.resolution().cmp(&b.resolution()),
                        Some(ord) => ord,
                    }
                });
                formats.first().copied().copied()
            }
            FormatRequest::HighestResolution { resolution, frame_format } => {
                let mut formats = list_of_formats.iter().filter(|x| {
                    frame_format.contains(&x.format()) && resolution.in_range(x.resolution())
                }).collect::<Vec<_>>();
                formats.sort_by(|a, b| {
                    match a.resolution().partial_cmp(&b.resolution()) {
                        None | Some(Ordering::Equal) => a.frame_rate().partial_cmp(&b.frame_rate()).unwrap_or(Ordering::Equal),
                        Some(ord) => ord,
                    }
                });
                formats.first().copied().copied()
            }
            FormatRequest::Exact { resolution, frame_rate, frame_format } => {
                let mut formats = list_of_formats.iter().filter(|x| {
                    frame_format.contains(&x.format()) && resolution == &x.resolution() && frame_rate == &x.frame_rate()
                }).collect::<Vec<_>>();
                formats.sort_by(|a, b| {
                    match a.resolution().partial_cmp(&b.resolution()) {
                        None | Some(Ordering::Equal) => a.frame_rate().partial_cmp(&b.frame_rate()).unwrap_or(Ordering::Equal),
                        Some(ord) => ord,
                    }
                });
                formats.first().copied().copied()
            }
        }
    }
}
