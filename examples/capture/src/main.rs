use nokhwa::{Camera, CameraFormat, CaptureAPIBackend, FrameFormat};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};

fn main() {
    // set up the Camera
    let mut camera = Camera::new(
        0,
        Some(CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30)),
        CaptureAPIBackend::AUTO,
    )
    .unwrap();
    // open stream
    camera.open_stream().unwrap();
    loop {
        println!(
            "{:?}, {:?}",
            camera.get_frame().unwrap().width(),
            camera.get_frame().unwrap().height()
        );
    }
}
