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
}

#[derive(Clone, Debug, Default, PartialOrd, PartialEq)]
pub struct FormatRequest {
    resolution: Option<Range<Resolution>>,
    frame_rate: Option<Range<FrameRate>>,
    frame_format: Option<Vec<FrameFormat>>,
    req_type: Option<CustomFormatRequestType>,
}

impl FormatRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_standard_frame_formats(mut self) -> Self {
        self.append_frame_formats(&mut vec![
            FrameFormat::MJpeg,
            FrameFormat::Rgb8,
            FrameFormat::Yuv422,
            FrameFormat::Nv12,
        ])
    }

    pub fn push_frame_format(mut self, frame_format: FrameFormat) -> Self {
        match &mut self.frame_format {
            Some(ffs) => ffs.push(frame_format),
            None => self.frame_format = Some(vec![frame_format]),
        }

        self
    }

    pub fn remove_frame_format(mut self, frame_format: FrameFormat) -> Self {
        if let Some(ffs) = &mut self.frame_format {
            if let Some(idx) = ffs.iter().position(|ff| ff == &frame_format) {
                ffs.remove(idx);
            }
        }

        self
    }

    pub fn append_frame_formats(mut self, frame_formats: &mut Vec<FrameFormat>) -> Self {
        match &mut self.frame_format {
            Some(ffs) => ffs.append(frame_formats),
            None => self.frame_format = Some(frame_formats.clone()),
        }

        self
    }

    pub fn reset_frame_formats(mut self) -> Self {
        self.frame_format = None;
        self
    }

    pub fn with_request_type(mut self, request_type: CustomFormatRequestType) -> Self {
        self.req_type = Some(request_type);
        self
    }

    pub fn reset_request_type(mut self) -> Self {
        self.req_type = None;
        self
    }

    pub fn set_resolution_range(mut self, resolution_range: Range<Resolution>) -> Self {
        self.resolution = Some(resolution_range);
        self
    }

    pub fn reset_resolution_range(mut self) -> Self {
        self.resolution = None;
        self
    }

    pub fn set_frame_rate_range(mut self, frame_rate_range: Range<FrameRate>) -> Self {
        self.frame_rate = Some(frame_rate_range);
        self
    }

    pub fn reset_frame_rate_range(mut self) -> Self {
        self.frame_rate = None;
        self
    }

    pub fn satisfied_by_format(&self, format: &CameraFormat) -> bool {
        // check resolution
        let resolution_satisfied = match self.resolution {
            Some(res_range) => res_range.in_range(format.resolution()),
            None => true,
        };

        let frame_rate_satisfied = match self.frame_rate {
            Some(fps_range) => fps_range.in_range(format.frame_rate()),
            None => true,
        };

        let frame_format_satisfied = match &self.frame_format {
            Some(frame_formats) => frame_formats.contains(&format.format()),
            None => true,
        };

        // we ignore custom bc that only makes sense in multiple formats

        resolution_satisfied && frame_rate_satisfied && frame_format_satisfied
    }


    pub fn resolve(&self, list_of_formats: &[CameraFormat]) -> Option<CameraFormat> {
        // filter out bad results
        let mut remaining_formats = list_of_formats.iter().filter(|x| self.satisfied_by_format(*x)).copied().collect::<Vec<CameraFormat>>();

        match self.req_type {
            Some(request) => {
                match request {
                    CustomFormatRequestType::HighestFrameRate => {
                        remaining_formats.sort_by(|a, b| {
                            a.frame_rate().partial_cmp(&b.frame_rate()).unwrap_or(Ordering::Equal)
                        });
                        Some(remaining_formats[0])
                    }
                    CustomFormatRequestType::HighestResolution => {
                        remaining_formats.sort_by(|a, b| {
                            a.frame_rate().partial_cmp(&b.frame_rate()).unwrap_or(Ordering::Equal)
                        });
                        Some(remaining_formats[0])
                    }
                    CustomFormatRequestType::Closest => {
                        
                        let mut closest_type = match (&self.frame_rate, &self.resolution) {
                            (Some(_), Some(_)) => ClosestType::Both,
                            (Some(_), None) => ClosestType::FrameRate,
                            (None, Some(_)) => ClosestType::Resolution,
                            (None, None) => ClosestType::None,
                        };

                        match closest_type {
                            ClosestType::Resolution => {
                                let resolution_point = match self.resolution.map(|x| x.preferred()).flatten() {
                                    Some(r) => r,
                                    None => return None,
                                };

                                let mut distances = remaining_formats.into_iter().map(|fmt| (fmt.resolution().distance_from(&resolution_point), fmt)).collect::<Vec<_>>();
                                distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or_else(|| Ordering::Equal));
                                VecDeque::from(distances).pop_front().map(|x| x.1)
                            }

                            ClosestType::FrameRate => {
                                let frame_rate_point = match self.frame_rate.map(|x| x.preferred()).flatten() {
                                    Some(f) => f,
                                    None => return None,
                                };

                                let mut distances = remaining_formats.into_iter().map(|fmt| (fmt.frame_rate().distance_from(&frame_rate_point), fmt)).collect::<Vec<_>>();
                                distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or_else(|| Ordering::Equal));
                                VecDeque::from(distances).pop_front().map(|x| x.1)
                            }
                            ClosestType::Both => {
                                let resolution_point = match self.resolution.map(|x| x.preferred()).flatten() {
                                    Some(r) => r,
                                    None => return None,
                                };                            
                                
                                let frame_rate_point = match self.frame_rate.map(|x| x.preferred()).flatten() {
                                    Some(f) => f,
                                    None => return None,
                                };

                                // lets calcuate distance in 3 dimensions (add both resolution and frame_rate together)

                                let mut distances = remaining_formats.into_iter()
                                    .map(|fmt| {
                                        (fmt.frame_rate().distance_from(&frame_rate_point) + fmt.resolution().distance_from(&resolution_point) as f32, fmt)
                                    })
                                    .collect::<Vec<_>>();
                                distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or_else(|| Ordering::Equal));
                                VecDeque::from(distances).pop_front().map(|x| x.1)
                            }
                            ClosestType::None => {
                                Some(remaining_formats[0])
                            }
                        }
                    }
                }
            }
            None => {
                Some(remaining_formats[0])
            }
        }
    }
}
