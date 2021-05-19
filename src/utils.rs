use std::fmt::Display;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum FrameFormat {
    MJPEG,
    YUYV,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Resolution {
    pub width_x: u32,
    pub height_y: u32,
}

impl Resolution {
    pub fn new(x: u32, y: u32) -> Self {
        Resolution {
            width_x: x,
            height_y: y,
        }
    }

    pub fn width(&self) -> u32 {
        self.width_x
    }

    pub fn height(&self) -> u32 {
        self.height_y
    }

    pub fn x(&self) -> u32 {
        self.width_x
    }

    pub fn y(&self) -> u32 {
        self.height_y
    }
}

impl Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.x(), self.y())
    }
}

impl PartialOrd for Resolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Resolution {
    // Flip around the order to make it seem the way the user would expect.
    // The user would expect a descending list of resolutions (aka highest -> lowest)
    fn cmp(&self, other: &Self) -> Ordering {
        match self.x().cmp(&other.x()) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.y().cmp(&other.y()),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub struct CameraFormat {
    resolution: Resolution,
    format: FrameFormat,
    framerate: u32,
}

impl CameraFormat {
    pub fn new(resolution: Resolution, format: FrameFormat, framerate: u32) -> Self {
        CameraFormat {
            resolution,
            format,
            framerate,
        }
    }

    pub fn new_from(res_x: u32, res_y: u32, format: FrameFormat, fps: u32) -> Self {
        CameraFormat {
            resolution: Resolution {
                width_x: res_x,
                height_y: res_y,
            },
            format,
            framerate: fps,
        }
    }

    pub fn res(&self) -> Resolution {
        self.resolution
    }

    pub fn width(&self) -> u32 {
        self.resolution.width()
    }

    pub fn height(&self) -> u32 {
        self.resolution.height()
    }

    pub fn frame_format(&self) -> FrameFormat {
        self.format
    }

    pub fn framerate(&self) -> u32 {
        self.framerate
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaptureAPIBackend {
    AUTO,
    V4L2,
    UVC,
    MSMF,
    OPENCV,
    FFMPEG,
}
