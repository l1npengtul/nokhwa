[![cargo version](https://img.shields.io/crates/v/nokhwa.svg)](https://crates.io/crates/nokhwa) ![docs.rs version](https://img.shields.io/docsrs/nokhwa)
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

```.ignore
// set up the Camera
let mut camera = Camera::new(
    0, // index
    Some(CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30)), // format
)
.unwrap();
// open stream
camera.open_stream().unwrap();
loop {
    let frame = camera.get_frame().unwrap();
    println!("{}, {}", frame.width(), frame.height());
}
```
A command line app made with `nokhwa` can be found in the `examples` folder.

## API Support
The table below lists current Nokhwa API support.
- The `Backend` column signifies the backend.
- The `Input` column signifies reading frames from the camera
- The `Query` column signifies system device list support
- The `Query-Device` column signifies reading device capabilities
- The `Platform` column signifies what Platform this is availible on.

 | Backend                             | Input              | Query             | Query-Device       | Platform            |
 |-------------------------------------|--------------------|-------------------|--------------------|---------------------|
 | Video4Linux(`input-v4l`)            | ✅                 | ✅                 | ✅                 | Linux               |
 | MSMF(`input-msmf`)                  | ✅                 | ✅                 | ✅                 | Windows             |
 | AVFoundation(`input-avfoundatuin`)^^| ✅                 | ✅                 | ✅                 | Mac                 |
 | libuvc(`input-uvc`)^^^              | ❌                 | ✅                 | ❌                 | Linux, Windows, Mac |
 | OpenCV(`input-opencv`)^             | ✅                 | ❌                 | ❌                 | Linux, Windows, Mac |
 | IPCamera(`input-ipcam`/OpenCV)^     | ✅                 | ❌                 | ❌                 | Linux, Windows, Mac |
 | GStreamer(`input-gst`)              | ✅                 | ✅                 | ✅                 | Linux, Windows, Mac |
 | JS/WASM(`input-wasm`)               | ✅                 | ✅                 | ✅                 | Browser(Web)        |

 ✅: Working, 🔮 : Experimental, ❌ : Not Supported, 🚧: Planned/WIP

  ^ = No CameraFormat setting support.

  ^^ = No FPS setting support.

  ^^^ = `input-uvc` is disabled for now as there are lifetime/soundness holes. You can still query, however.
## Feature
The default feature includes nothing. Anything starting with `input-*` is a feature that enables the specific backend. 
As a general rule of thumb, you would want to keep at least `input-uvc` or other backend that has querying enabled so you can get device information from `nokhwa`.

`input-*` features:
 - `input-v4l`: Enables the `Video4Linux` backend. (linux)
 - `input-msmf`: Enables the `MediaFoundation` backennd. (Windows 7 or newer)
 - `input-avfoundation`: Enables the `AVFoundation` backend. (MacOSX 10.7)
 - `input-uvc`: Enables the `libuvc` backend. (cross-platform, libuvc statically-linked)
 - `input-opencv`: Enables the `opencv` backend. (cross-platform) 
 - `input-ipcam`: Enables the use of IP Cameras, please see the `NetworkCamera` struct. Note that this relies on `opencv`, so it will automatically enable the `input-opencv` feature.
 - `input-gst`: Enables the `gstreamer` backend. (cross-platform)
 - `input-jscam`: Enables the use of the `JSCamera` struct, which uses browser APIs. (Web)

Conversely, anything that starts with `output-*` controls a feature that controls the output of something (usually a frame from the camera)

`output-*` features:
 - `output-wgpu`: Enables the API to copy a frame directly into a `wgpu` texture.
 - `output-wasm`: Generate WASM API binding specific functions.
 - `output-threaded`: Enable the threaded/callback based camera. 

Other features:
 - `decoding`: Enables `mozjpeg` decoding. Enabled by default.  
 - `small-wasm`: Makes use of `wee-alloc`. Only enable this if you are building a standalone WASM binary!

 Please use the following command for `wasm-pack` in order to get a functional WASM binary:
 ```.ignore
 wasm-pack build --release -- --features "input-jscam, output-wasm, test-fail-warning" --no-default-features 
 ```
 - `docs-only`: Documentation feature. Enabled for docs.rs builds.
 - `docs-nolink`: Build documentation **without** linking to any libraries. Enabled for docs.rs builds.
 - `test-fail-warning`: Fails on warning. Enabled in CI.
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
 - All contributions are under the Apache 2.0 license unless otherwise specified

## Minimum Service Rust Version
`nokhwa` may build on older versions of `rustc`, but there is no guarantee except for the latest stable rust. 

## 0.10
0.10 is currently stalled due to upstream not having the necessary features (wasm-bindgen).
