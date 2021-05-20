use std::fmt::Display;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum FrameFormat {
    MJPEG,
    YUYV,
}

/// Describes a Resolution. 
/// This struct consists of a Width and a Height value (x,y).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Resolution {
    pub width_x: u32,
    pub height_y: u32,
}

impl Resolution {
    /// Create a new resolution from 2 image size coordinates.
    pub fn new(x: u32, y: u32) -> Self {
        Resolution {
            width_x: x,
            height_y: y,
        }
    }

    /// Get the width of Resolution
    pub fn width(self) -> u32 {
        self.width_x
    }

    /// Get the height of Resolution
    pub fn height(self) -> u32 {
        self.height_y
    }

    /// Get the x (width) of Resolution
    pub fn x(self) -> u32 {
        self.width_x
    }

    /// Get the y (height) of Resolution
    pub fn y(self) -> u32 {
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

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct CameraInfo {
    human_name: String,
    description: String,
    misc: String,
    index: usize,
}

impl CameraInfo {
    pub fn new(human_name: String, description: String, misc: String, index: usize) -> Self {
        CameraInfo {
            human_name,
            description,
            misc,
            index,

        }
    }

    /// Get a reference to the device info's human name.
    pub fn human_name(&self) -> &String {
        &self.human_name
    }

    /// Set the device info's human name.
    pub fn set_human_name(&mut self, human_name: String) {
        self.human_name = human_name;
    }

    /// Get a reference to the device info's description.
    pub fn description(&self) -> &String {
        &self.description
    }

    /// Set the device info's description.
    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    /// Get a reference to the device info's misc.
    pub fn misc(&self) -> &String {
        &self.misc
    }

    /// Set the device info's misc.
    pub fn set_misc(&mut self, misc: String) {
        self.misc = misc;
    }

    /// Get a reference to the device info's index.
    pub fn index(&self) -> &usize {
        &self.index
    }

    /// Set the device info's index.
    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
}

impl PartialOrd for CameraInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CameraInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
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
