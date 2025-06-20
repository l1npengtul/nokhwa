use crate::ranges::ValidatableRange;
use crate::utils::Distance;
use crate::{
    frame_format::FrameFormat,
    ranges::Range,
    types::{CameraFormat, FrameRate, Resolution},
};

/// A helper for choosing a [`CameraFormat`].
/// The use of this is completely optional - for a simpler way try [`crate::camera::Camera::enumerate_formats`].
///
/// The `frame_format` field filters out the [`CameraFormat`]s by [`FrameFormat`].
#[derive(Clone, Debug, PartialEq)]
pub enum FormatRequestType {
    /// Pick the closest [`CameraFormat`] to the one requested
    Closest {
        resolution: Option<Range<Resolution>>,
        preferred_resolution: Option<Resolution>,
        frame_rate: Option<Range<FrameRate>>,
        preferred_frame_rate: Option<FrameRate>,
    },
    HighestFrameRate {
        frame_rate: Range<FrameRate>,
    },
    HighestResolution {
        resolution: Range<Resolution>,
    },
    Exact {
        resolution: Resolution,
        frame_rate: FrameRate,
    },
    Any,
}

#[derive(Clone, Debug)]
pub struct FormatRequest {
    request_type: FormatRequestType,
    allowed_frame_formats: Vec<FrameFormat>,
}

impl FormatRequest {
    #[must_use] pub fn new(
        format_request_type: FormatRequestType,
        allowed_frame_formats: Vec<FrameFormat>,
    ) -> Self {
        Self {
            request_type: format_request_type,
            allowed_frame_formats,
        }
    }

    #[must_use] pub fn best<'a>(&self, camera_formats: &'a [CameraFormat]) -> Option<&'a CameraFormat> {
        camera_formats.first()
    }

    #[must_use] pub fn sort_foramts(&self, mut camera_formats: Vec<CameraFormat>) -> Vec<CameraFormat> {
        if camera_formats.is_empty() {
            return camera_formats;
        }

        match self.request_type {
            FormatRequestType::Closest {
                resolution,
                preferred_resolution,
                frame_rate,
                preferred_frame_rate,
            } => {
                // lets calcuate distance in 3 dimensions (add both resolution and frame_rate together)
                camera_formats.sort_by(|a, b| {
                    let a_distance =
                        format_distance_to_point(&preferred_resolution, &preferred_frame_rate, a);
                    let b_distance =
                        format_distance_to_point(&preferred_resolution, &preferred_frame_rate, b);

                    a_distance.total_cmp(&b_distance)
                });

                camera_formats
                    .into_iter()
                    .filter(|fmt| self.allowed_frame_formats.contains(fmt.format()))
                    .filter(|cam_fmt| {
                        if let Some(res_range) = resolution {
                            return res_range.validate(cam_fmt.resolution());
                        }

                        if let Some(frame_rate_range) = frame_rate {
                            return frame_rate_range.validate(cam_fmt.frame_rate());
                        }
                        true
                    })
                    .collect()
            }
            FormatRequestType::HighestFrameRate { frame_rate } => {
                camera_formats.sort_by(|a, b| a.frame_rate().cmp(b.frame_rate()));

                camera_formats
                    .into_iter()
                    .filter(|fmt| self.allowed_frame_formats.contains(fmt.format()))
                    .filter(|a| frame_rate.validate(a.frame_rate()))
                    .collect()
            }
            FormatRequestType::HighestResolution { resolution } => {
                camera_formats.sort_by(|a, b| a.resolution().cmp(b.resolution()));

                camera_formats
                    .into_iter()
                    .filter(|fmt| self.allowed_frame_formats.contains(fmt.format()))
                    .filter(|a| resolution.validate(a.resolution()))
                    .collect()
            }
            FormatRequestType::Exact {
                resolution,
                frame_rate,
            } => camera_formats
                .into_iter()
                .filter(|fmt| self.allowed_frame_formats.contains(fmt.format()))
                .filter(|a| resolution.eq(a.resolution()) && frame_rate.eq(a.frame_rate()))
                .collect(),
            FormatRequestType::Any => {
                // return as-is
                camera_formats
            }
        }
    }
}

#[must_use] 
#[allow(clippy::cast_precision_loss)]
pub fn format_distance_to_point(
    resolution: &Option<Resolution>,
    frame_rate: &Option<FrameRate>,
    format: &CameraFormat,
) -> f32 {
    let frame_rate_distance = match frame_rate {
        Some(f_point) => (format.frame_rate() - f_point)
            .approximate_float()
            .unwrap_or(f32::INFINITY)
            .abs(),
        None => 0_f32,
    };
    
    let resolution_point_distance = match resolution {
        Some(res_pt) => format.resolution().distance_from(res_pt) as f32,
        None => 0_f32,
    };

    frame_rate_distance + resolution_point_distance
}
