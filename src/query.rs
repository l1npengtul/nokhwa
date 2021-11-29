/*
 * Copyright 2021 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::{CameraIndex, CameraInfo, CaptureAPIBackend, NokhwaError};

/// Query the system for a list of available devices.
/// Usually the order goes Native -> UVC -> Gstreamer.
/// # Quirks
/// - Media Foundation: The symbolic link for the device is listed in the `misc` attribute of the [`CameraInfo`].
/// - Media Foundation: The names may contain invalid characters since they were converted from UTF16.
/// - AVFoundation: The ID of the device is stored in the `misc` attribute of the [`CameraInfo`].
/// - AVFoundation: There is lots of miscellaneous info in the `desc` attribute.
/// # Errors
/// If you use an unsupported API (check the README or crate root for more info), incompatible backend for current platform, incompatible platform, or insufficient permissions, etc
/// this will error.
pub fn query<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    query_devices(CaptureAPIBackend::Auto)
}

// TODO: Update as this goes
/// Query the system for a list of available devices. Please refer to the API Backends that support `Query`) <br>
/// Currently, these are `V4L`, `MediaFoundation`, `AVFoundation`, `UVC`, and `GST`. <br>
/// Usually the order goes Native -> UVC -> Gstreamer.
/// # Quirks
/// - Media Foundation: The symbolic link for the device is listed in the `misc` attribute of the [`CameraInfo`].
/// - Media Foundation: The names may contain invalid characters since they were converted from UTF16.
/// - AVFoundation: The ID of the device is stored in the `misc` attribute of the [`CameraInfo`].
/// - AVFoundation: There is lots of miscellaneous info in the `desc` attribute.
/// # Errors
/// If you use an unsupported API (check the README or crate root for more info), incompatible backend for current platform, incompatible platform, or insufficient permissions, etc
/// this will error.
#[allow(clippy::module_name_repetitions)]
pub fn query_devices<'a>(api: CaptureAPIBackend) -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    match api {
        CaptureAPIBackend::Auto => {
            // determine platform
            match std::env::consts::OS {
                "linux" => {
                    if cfg!(feature = "input-v4l") && cfg!(target_os = "linux") {
                        query_devices(CaptureAPIBackend::Video4Linux)
                    } else if cfg!(feature = "input-uvc") {
                        query_devices(CaptureAPIBackend::UniversalVideoClass)
                    } else if cfg!(feature = "input-gstreamer") {
                        query_devices(CaptureAPIBackend::GStreamer)
                    } else if cfg!(feature = "input-opencv") {
                        query_devices(CaptureAPIBackend::OpenCv)
                    } else {
                        Err(NokhwaError::UnsupportedOperationError(
                            CaptureAPIBackend::Auto,
                        ))
                    }
                }
                "windows" => {
                    if cfg!(feature = "input-msmf") && cfg!(target_os = "windows") {
                        query_devices(CaptureAPIBackend::MediaFoundation)
                    } else if cfg!(feature = "input-uvc") {
                        query_devices(CaptureAPIBackend::UniversalVideoClass)
                    } else if cfg!(feature = "input-gstreamer") {
                        query_devices(CaptureAPIBackend::GStreamer)
                    } else if cfg!(feature = "input-opencv") {
                        query_devices(CaptureAPIBackend::OpenCv)
                    } else {
                        Err(NokhwaError::UnsupportedOperationError(
                            CaptureAPIBackend::Auto,
                        ))
                    }
                }
                "macos" => {
                    if cfg!(feature = "input-avfoundation") {
                        query_devices(CaptureAPIBackend::AVFoundation)
                    } else if cfg!(feature = "input-uvc") {
                        query_devices(CaptureAPIBackend::UniversalVideoClass)
                    } else if cfg!(feature = "input-gstreamer") {
                        query_devices(CaptureAPIBackend::GStreamer)
                    } else if cfg!(feature = "input-opencv") {
                        query_devices(CaptureAPIBackend::OpenCv)
                    } else {
                        Err(NokhwaError::UnsupportedOperationError(
                            CaptureAPIBackend::Auto,
                        ))
                    }
                }
                "ios" => {
                    if cfg!(feature = "input-avfoundation") {
                        query_devices(CaptureAPIBackend::AVFoundation)
                    } else {
                        Err(NokhwaError::UnsupportedOperationError(
                            CaptureAPIBackend::Auto,
                        ))
                    }
                }
                _ => Err(NokhwaError::UnsupportedOperationError(
                    CaptureAPIBackend::Auto,
                )),
            }
        }
        CaptureAPIBackend::AVFoundation => query_avfoundation(),
        CaptureAPIBackend::Video4Linux => query_v4l(),
        CaptureAPIBackend::UniversalVideoClass => query_uvc(),
        CaptureAPIBackend::MediaFoundation => query_msmf(),
        CaptureAPIBackend::GStreamer => query_gstreamer(),
        CaptureAPIBackend::OpenCv => Err(NokhwaError::UnsupportedOperationError(api)),
        CaptureAPIBackend::Network => Err(NokhwaError::UnsupportedOperationError(api)),
        CaptureAPIBackend::Browser => query_wasm(),
    }
}

// TODO: More

#[cfg(all(feature = "input-v4l", target_os = "linux"))]
#[allow(clippy::unnecessary_wraps)]
fn query_v4l<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    Ok({
        let camera_info: Vec<CameraInfo> = v4l::context::enum_devices()
            .iter()
            .map(|node| {
                CameraInfo::new(
                    node.name()
                        .unwrap_or(format!("{}", node.path().to_string_lossy())),
                    format!("Video4Linux Device @ {}", node.path().to_string_lossy()),
                    "".to_string(),
                    CameraIndex::Index(node.index() as u32),
                )
            })
            .collect();
        camera_info
    })
}

#[cfg(any(not(feature = "input-v4l"), not(target_os = "linux")))]
fn query_v4l<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperationError(
        CaptureAPIBackend::Video4Linux,
    ))
}

#[cfg(feature = "input-uvc")]
fn query_uvc<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    use uvc::Device;
    let context = match uvc::Context::new() {
        Ok(ctx) => ctx,
        Err(why) => {
            return Err(NokhwaError::GeneralError(format!(
                "UVC Context failure: {}",
                why
            )))
        }
    };

    let usb_devices = usb_enumeration::enumerate(None, None);
    let uvc_devices = match context.devices() {
        Ok(devs) => {
            let device_vec: Vec<Device> = devs.collect();
            device_vec
        }
        Err(why) => {
            return Err(NokhwaError::GeneralError(format!(
                "UVC Context Devicelist failure: {}",
                why
            )))
        }
    };

    let mut camera_info_vec = vec![];
    let mut counter = 0_usize;

    // Optimize this O(n*m) algorithm
    for usb_dev in &usb_devices {
        for uvc_dev in &uvc_devices {
            if let Ok(desc) = uvc_dev.description() {
                if desc.product_id == usb_dev.product_id && desc.vendor_id == usb_dev.vendor_id {
                    let name = usb_dev
                        .description
                        .as_ref()
                        .unwrap_or(&format!(
                            "{}:{} {} {}",
                            desc.vendor_id,
                            desc.product_id,
                            desc.manufacturer.unwrap_or_else(|| "Generic".to_string()),
                            desc.product.unwrap_or_else(|| "Camera".to_string())
                        ))
                        .clone();

                    camera_info_vec.push(CameraInfo::new(
                        name.clone(),
                        usb_dev
                            .description
                            .as_ref()
                            .unwrap_or(&"".to_string())
                            .clone(),
                        format!(
                            "{}:{} {}",
                            desc.vendor_id,
                            desc.product_id,
                            desc.serial_number.unwrap_or_else(|| "".to_string())
                        ),
                        CameraIndex::Index(counter as u32),
                    ));
                    counter += 1;
                }
            }
        }
    }
    Ok(camera_info_vec)
}

#[cfg(not(feature = "input-uvc"))]
fn query_uvc<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperationError(
        CaptureAPIBackend::UniversalVideoClass,
    ))
}

#[cfg(feature = "input-gst")]
fn query_gstreamer<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    use gstreamer::{
        prelude::{DeviceExt, DeviceMonitorExt, DeviceMonitorExtManual},
        Caps, DeviceMonitor,
    };
    use std::str::FromStr;
    if let Err(why) = gstreamer::init() {
        return Err(NokhwaError::GeneralError(format!(
            "Failed to init gstreamer: {}",
            why
        )));
    }
    let device_monitor = DeviceMonitor::new();
    let video_caps = match Caps::from_str("video/x-raw") {
        Ok(cap) => cap,
        Err(why) => {
            return Err(NokhwaError::GeneralError(format!(
                "Failed to generate caps: {}",
                why
            )))
        }
    };
    let _video_filter_id = match device_monitor.add_filter(Some("Video/Source"), Some(&video_caps))
    {
        Some(id) => id,
        None => {
            return Err(NokhwaError::StructureError {
                structure: "Video Filter ID Video/Source".to_string(),
                error: "Null".to_string(),
            })
        }
    };
    if let Err(why) = device_monitor.start() {
        return Err(NokhwaError::GeneralError(format!(
            "Failed to start device monitor: {}",
            why
        )));
    }
    let mut counter = 0;
    let devices: Vec<CameraInfo> = device_monitor
        .devices()
        .iter_mut()
        .map(|gst_dev| {
            let name = DeviceExt::display_name(gst_dev);
            let class = DeviceExt::device_class(gst_dev);
            counter += 1;
            CameraInfo::new(
                name.to_string(),
                class.to_string(),
                "".to_string(),
                CameraIndex::Index(counter - 1),
            )
        })
        .collect();
    device_monitor.stop();
    Ok(devices)
}

#[cfg(not(feature = "input-gst"))]
fn query_gstreamer<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperationError(
        CaptureAPIBackend::GStreamer,
    ))
}

// please refer to https://docs.microsoft.com/en-us/windows/win32/medfound/enumerating-video-capture-devices
#[cfg(all(feature = "input-msmf", target_os = "windows"))]
fn query_msmf<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    let list: Vec<CameraInfo> =
        match nokhwa_bindings_windows::wmf::query_media_foundation_descriptors() {
            Ok(l) => l
                .into_iter()
                .map(|mf_desc| {
                    let camera_info: CameraInfo = mf_desc.into();
                    camera_info
                })
                .collect(),
            Err(why) => return Err(why.into()),
        };
    Ok(list)
}

#[cfg(any(not(feature = "input-msmf"), not(target_os = "windows")))]
fn query_msmf<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperationError(
        CaptureAPIBackend::MediaFoundation,
    ))
}

#[cfg(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
))]
fn query_avfoundation<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    use nokhwa_bindings_macos::avfoundation::query_avfoundation as q_avf;

    Ok(q_avf()?
        .into_iter()
        .map(CameraInfo::from)
        .collect::<Vec<CameraInfo>>())
}

#[cfg(not(all(
    feature = "input-avfoundation",
    any(target_os = "macos", target_os = "ios")
)))]
fn query_avfoundation<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperationError(
        CaptureAPIBackend::AVFoundation,
    ))
}

#[cfg(feature = "input-jscam")]
fn query_wasm<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    use crate::js_camera::query_js_cameras;
    use wasm_rs_async_executor::single_threaded::block_on;

    block_on(query_js_cameras())
}

#[cfg(not(feature = "input-jscam"))]
fn query_wasm<'a>() -> Result<Vec<CameraInfo<'a>>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperationError(
        CaptureAPIBackend::Browser,
    ))
}
