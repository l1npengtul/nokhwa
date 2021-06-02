# nokhwa
Nokhwa(녹화): Korean word meaning "to record".

A Simple-to-use, cross-platform Rust Webcam Capture Library

## Using nokhwa
Nokhwa can be added to your crate by adding it to your `Cargo.toml`:
```.ignore
[dependencies.nokhwa]
// TODO: replace the "*" with the latest version of `nokhwa`
version = "*"
// TODO: add some features
features = [""]
```

Most likely, you will only use functionality provided by the `Camera` struct. If you need lower-level access, you may instead opt to use the raw capture backends found at `nokhwa::backends::capture::*`.

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
    let frame = camera.get_frame().unwrap();
    println!("{}, {}", frame.width(), frame.height());
}
```
They can be found in the `examples` folder.

## Feature
The default feature includes nothing. Anything starting with `input-*` is a feature that enables the specific backend. 

`input-*` features:
 - `input-v4l`: Enables the `Video4Linux` backend (linux)
 - `input-uvc`: Enables the `libuvc` backend (cross-platform)

Conversely, anything that starts with `output-*` controls a feature that controls the output of something (usually a frame from the camera)

`output-*` features:
 - `output-wgpu`: Enables the API to copy a frame directly into a `wgpu` texture.

You many want to pick and choose to reduce bloat.

## Issues
If you are making an issue, please make sure that
 - It has not been made yet
 - Attach what you were doing, your environment, steps to reproduce, and backtrace.
Thank you!

## Contributing
Contributions are welcome!
 - Please `rustfmt` all your code and adhere to the clippy lints (unless necessary not to do so)
 - Please limit use of `unsafe`
 - All contributions are under the MPL 2.0 license unless otherwise specified

## Minimum Service Rust Version
`nokhwa` may build on older versions of `rustc`, but there is no guarantee except for the latest stable rust. 
