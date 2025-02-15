use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;

fn main() {
    let index: CameraIndex = CameraIndex::Index(50);
    let requested: RequestedFormat<'_> =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = Camera::new(index, requested).unwrap();
    println!("{}", camera.camera_format());
    camera.open_stream().unwrap();
    let frame = camera.frame().unwrap();
    camera.stop_stream().unwrap();
    let decoded = frame.decode_image::<RgbFormat>().unwrap();
    decoded
        .save_with_format("turtle.jpeg", image::ImageFormat::Jpeg)
        .unwrap()
}
