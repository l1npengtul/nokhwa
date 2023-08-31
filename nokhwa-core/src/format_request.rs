use crate::frame_format::SourceFrameFormat;
use crate::types::Range;
use crate::{
    frame_format::FrameFormat,
    types::{ApiBackend, CameraFormat, Resolution},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum CustomFormatRequestType {
    HighestFPS,
    HighestResolution,
    Closest,
}

#[derive(Clone, Debug, Default, PartialOrd, PartialEq)]
pub struct FormatRequest {
    resolution: Option<Range<Resolution>>,
    frame_rate: Option<Range<u32>>,
    frame_format: Option<Vec<FrameFormat>>,
    req_type: Option<CustomFormatRequestType>,
}

impl FormatRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_resolution(mut self, resolution: Resolution, exact: bool) -> Self {
        self.resolution = Some(resolution);
        self.resolution_exact = exact;
        self
    }

    pub fn reset_resolution(mut self) -> Self {
        self.resolution = None;
        self.resolution_exact = false;
        self
    }
    pub fn with_frame_rate(mut self, frame_rate: u32, exact: bool) -> Self {
        self.frame_rate = Some(frame_rate);
        self.frame_rate_exact = exact;
        self
    }

    pub fn with_standard_frame_rate() {}

    pub fn reset_frame_rate(mut self) -> Self {
        self.frame_rate = None;
        self.frame_rate_exact = false;
        self
    }
    pub fn with_frame_formats(mut self, frame_formats: Vec<FrameFormat>) -> Self {
        self.frame_format = Some(frame_formats);
        self
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
            if let Some(idx) = ffs.iter().position(frame_format) {
                ffs.remove(idx)
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
}

pub fn resolve_format_request(
    request: FormatRequest,
    availible_formats: Vec<CameraFormat>,
) -> CameraFormat {
    // filter out by
}
