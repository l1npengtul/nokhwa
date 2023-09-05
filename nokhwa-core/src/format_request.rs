use crate::frame_format::SourceFrameFormat;
use crate::types::Range;
use crate::{
    frame_format::FrameFormat,
    types::{ApiBackend, CameraFormat, Resolution},
};
use paste::paste;
use std::collections::{BTreeMap, BTreeSet};

macro_rules! range_set_fields {
    ($(($range_type:ty, $name:ident),)*) => {
        $(
        paste! {
            pub fn [< with_maximum_ $name >](mut self, $name: $range_type) -> Self {
                match &mut self.$name {
                    Some(r) => {
                        r.set_maximum(Some($name))
                    }
                    None => {
                        self.$name: Option<Range<$range_type>> = Some(Range {
                            maximum: Some($name),
                            minimum: None,
                            preferred: $range_type::default()
                        });
                    }
                }
                self
            }

            pub fn [< reset_maximum_ $name >](mut self) -> Self {
                if let Some(r) = self.$name {
                    self.$name.set_maximum(None)
                }

                self
            }


            pub fn [< set_maximum_ $name >](&mut self, $name: Option<$range_type>) {
                match &mut self.$name {
                    Some(r) => {
                        r.set_maximum($name)
                    }
                    None => {
                        self.$name: Option<Range<$range_type>> = Some(Range {
                            maximum: $name,
                            minimum: None,
                            preferred: $range_type::default()
                        });
                    }
                }
            }

            pub fn [< with_preferred_ $name >](mut self, $name: $range_type) -> Self {
                match self.$name {
                    Some(r) => {
                        r.set_preferred(Some($name))
                    }
                    None => {
                        self.$name: Option<Range<$range_type>> = Some(Range {
                            maximum: None,
                            minimum: None,
                            preferred: $range_type::default()
                        });
                    }
                }
                self
            }

            pub fn [< set_preferred_ $name >](&mut self, $name: $range_type) {
                match &mut self.$name {
                    Some(r) => {
                        r.set_preferred($name)
                    }
                    None => {
                        self.$name: Option<Range<$range_type>> = Some(Range {
                            maximum: None,
                            minimum: None,
                            preferred: $range_type
                        });
                    }
                }
            }

            pub fn [< with_minimum_ $name >](mut self, $name: $range_type) -> Self {
                match self.$name {
                    Some(r) => {
                        r.set_minimum(Some($name))
                    }
                    None => {
                        self.$name: Option<Range<$range_type>> = Some(Range {
                            maximum: None,
                            minimum: Some($name),
                            preferred: $range_type::default()
                        });
                    }
                }
                self
            }

            pub fn [< reset_minimum_ $name >](mut self) -> Self {
                if let Some(r) = self.$name {
                    self.$name.set_minimum(None)
                }

                self
            }

            pub fn [< set_minimum_ $name >](&mut self, $name: Option<$range_type>) {
                match &mut self.$name {
                    Some(r) => {
                        r.set_minimum($name)
                    }
                    None => {
                        self.$name: Option<Range<$range_type>> = Some(Range {
                            maximum: None,
                            minimum: $name,
                            preferred: $range_type::default()
                        });
                    }
                }
            }

            pub fn [< with_ $name _range >](mut self, $name: Option<Range<$range_type>>) -> Self {
                self.$name = $name
                Self
            }

            pub fn [< set_ $name _range >](&mut self, $name: Option<Range<$range_type>>) {
                self.$name = $name
            }
        }
        )*
    };
}

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

range_set_fields!((Resolution, resolution), (u32, frame_rate),);

// tomorrow wont come for those without FRAME FORMATS
pub fn resolve_format_request(
    request: FormatRequest,
    availible_formats: Vec<CameraFormat>,
) -> CameraFormat {
    // filter out by parts first
}
