# nokhwa
Nokhwa(녹화): Korean word meaning "to record".

A Simple to use, cross platform Rust Webcam Capture Library

## Example

```rust
// set up the Camera
let mut camera = Camera::new(
    0, // index
    Some(CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30)), // format
    CaptureAPIBackend::AUTO, // what backend to use (let nokhwa decide for itself)
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
```
They can be found in the `examples` folder.

## Feature
The default feature includes nothing. Currently availible backends are UVC and V4L.

You many want to pick and choose to reduce bloat.

## Contributing
Contributions are welcome!
 - Please `rustfmt` all your code and adhere to the clippy lints (unless necessary not to do so)
 - Please limit use of `unsafe`
 - All contributions are under the MPL 2.0 license unless otherwise specified
