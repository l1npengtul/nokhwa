use crate::{CameraInfo, CaptureAPIBackend, NokhwaError};

// TODO: Update as this goes
/// Query the system for a list of available devices. Please refer to the API Backends that support `Query`) <br>
/// Currently, these are V4L, UVC, and GST. <br>
/// Usually the order goes Native -> UVC -> Gstreamer.
/// # Errors
/// If you use an unsupported API (check the README or crate root for more info), incompatible backend for current platform, incompatible platform, or insufficient permissions, etc
/// this will error.
#[allow(clippy::module_name_repetitions)]
pub fn query_devices(api: CaptureAPIBackend) -> Result<Vec<CameraInfo>, NokhwaError> {
    match api {
        CaptureAPIBackend::Auto => {
            // determine platform
            match std::env::consts::OS {
                "linux" => {
                    if cfg!(feature = "input-v4l") {
                        query_devices(CaptureAPIBackend::Video4Linux)
                    } else if cfg!(feature = "input-uvc") {
                        query_devices(CaptureAPIBackend::UniversalVideoClass)
                    } else if cfg!(feature = "input-ffmpeg") {
                        query_devices(CaptureAPIBackend::Ffmpeg)
                    } else if cfg!(feature = "input-gstreamer") {
                        query_devices(CaptureAPIBackend::GStreamer)
                    } else {
                        Err(NokhwaError::UnsupportedOperation(CaptureAPIBackend::Auto))
                    }
                }
                "windows" => {
                    if cfg!(feature = "input-msmf") {
                        query_devices(CaptureAPIBackend::Windows)
                    } else if cfg!(feature = "input-uvc") {
                        query_devices(CaptureAPIBackend::UniversalVideoClass)
                    } else if cfg!(feature = "input-ffmpeg") {
                        query_devices(CaptureAPIBackend::Ffmpeg)
                    } else if cfg!(feature = "input-gstreamer") {
                        query_devices(CaptureAPIBackend::GStreamer)
                    } else {
                        Err(NokhwaError::UnsupportedOperation(CaptureAPIBackend::Auto))
                    }
                }
                "macos" => {
                    if cfg!(feature = "input-avfoundation") {
                        query_devices(CaptureAPIBackend::AVFoundation)
                    } else if cfg!(feature = "input-uvc") {
                        query_devices(CaptureAPIBackend::UniversalVideoClass)
                    } else if cfg!(feature = "input-ffmpeg") {
                        query_devices(CaptureAPIBackend::Ffmpeg)
                    } else if cfg!(feature = "input-gstreamer") {
                        query_devices(CaptureAPIBackend::GStreamer)
                    } else {
                        Err(NokhwaError::UnsupportedOperation(CaptureAPIBackend::Auto))
                    }
                }
                _ => Err(NokhwaError::UnsupportedOperation(CaptureAPIBackend::Auto)),
            }
        }
        CaptureAPIBackend::Video4Linux => query_v4l(),
        CaptureAPIBackend::UniversalVideoClass => query_uvc(),
        CaptureAPIBackend::Windows => Err(NokhwaError::UnsupportedOperation(
            CaptureAPIBackend::Windows,
        )),
        CaptureAPIBackend::Ffmpeg => {
            Err(NokhwaError::UnsupportedOperation(CaptureAPIBackend::Ffmpeg))
        }
        CaptureAPIBackend::GStreamer => query_gstreamer(),
        _ => Err(NokhwaError::UnsupportedOperation(api)),
    }
}

// TODO: More

#[cfg(feature = "input-v4l")]
#[allow(clippy::unnecessary_wraps)]
fn query_v4l() -> Result<Vec<CameraInfo>, NokhwaError> {
    Ok({
        let camera_info: Vec<CameraInfo> = v4l::context::enum_devices()
            .iter()
            .map(|node| {
                CameraInfo::new(
                    node.name()
                        .unwrap_or(format!("{}", node.path().to_string_lossy())),
                    format!("Video4Linux Device @ {}", node.path().to_string_lossy()),
                    "".to_string(),
                    node.index(),
                )
            })
            .collect();
        camera_info
    })
}

#[cfg(not(feature = "input-v4l"))]
fn query_v4l() -> Result<Vec<CameraInfo>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperation(
        CaptureAPIBackend::Video4Linux,
    ))
}

#[cfg(feature = "input-uvc")]
fn query_uvc() -> Result<Vec<CameraInfo>, NokhwaError> {
    use uvc::Device;
    let context = match uvc::Context::new() {
        Ok(ctx) => ctx,
        Err(why) => {
            return Err(NokhwaError::GeneralError(format!(
                "UVC Context failure: {}",
                why.to_string()
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
                why.to_string()
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
                        counter,
                    ));
                    counter += 1;
                }
            }
        }
    }
    Ok(camera_info_vec)
}

#[cfg(not(feature = "input-uvc"))]
fn query_uvc() -> Result<Vec<CameraInfo>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperation(
        CaptureAPIBackend::UniversalVideoClass,
    ))
}

#[cfg(feature = "input-gst")]
fn query_gstreamer() -> Result<Vec<CameraInfo>, NokhwaError> {
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
                why.to_string()
            )))
        }
    };
    let _video_filter_id = match device_monitor.add_filter(Some("Video/Source"), Some(&video_caps))
    {
        Some(id) => id,
        None => {
            return Err(NokhwaError::CouldntOpenDevice(
                "Failed to generate Device Monitor Filter ID with video/x-raw and Video/Source"
                    .to_string(),
            ))
        }
    };
    if let Err(why) = device_monitor.start() {
        return Err(NokhwaError::GeneralError(format!(
            "Failed to start device monitor: {}",
            why.to_string()
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
                counter - 1,
            )
        })
        .collect();
    device_monitor.stop();
    Ok(devices)
}

#[cfg(not(feature = "input-gst"))]
fn query_gstreamer() -> Result<Vec<CameraInfo>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperation(
        CaptureAPIBackend::GStreamer,
    ))
}

// please refer to https://docs.microsoft.com/en-us/windows/win32/medfound/enumerating-video-capture-devices
#[cfg(feature = "input-msmf")]
fn query_msmf() -> Result<Vec<CameraInfo>, NokhwaError> {
    use nokhwa_bindings_windows::Windows::Win32::{
        Foundation::PWSTR,
        Media::MediaFoundation::{
            IMFActivate, IMFAttributes, MFCreateAttributes, MFEnumDeviceSources, MFStartup,
            MFSTARTUP_NOSOCKET, MF_API_VERSION, MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
            MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_CATEGORY,
            MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
            MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
        },
        System::Com::CoTaskMemFree,
    };
    use std::ffi::{c_void, OsStr, OsString};

    unsafe {
        MFStartup(MF_API_VERSION, MFSTARTUP_NOSOCKET);
    }

    let mut attributes_opt: Option<IMFAttributes> = None;
    unsafe { MFCreateAttributes(&mut attributes_opt, 1) };

    let attributes = match attributes_opt {
        Some(attr) => {
            if let Err(why) = unsafe {
                attr.SetGUID(
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                )
            } {
                return Err(NokhwaError::GeneralError(format!(
                    "Failed to set GUID for MSMF: {}",
                    why.to_string()
                )));
            }
            attr
        }
        None => {
            return Err(NokhwaError::GeneralError(
                "Failed to set Attributes for MSMF: {}"(),
            ))
        }
    };

    let mut devices_opt: Option<IMFActivate> = None;
    let mut devices_ptr: *mut Option<IMFActivate> = &mut devices_opt;
    let devices_ptr_d: *mut *mut Option<IMFActivate> = &mut devices_ptr; // WHYYYYYYYYYYYYYYY

    let mut camera_info_vec = vec![];
    let mut count = 0;

    if let Err(why) = unsafe { MFEnumDeviceSources(attributes, devices_ptr_d, &mut count) } {
        return Err(NokhwaError::GeneralError(format!(
            "Failed to query devices: {}",
            why.to_string()
        )));
    }

    match &mut devices_opt {
        Some(devices) => {
            for device_index in 0..count {
                // name
                let mut raw_name = PWSTR::default();
                let mut size_name = 0_u32;
                unsafe {
                    devices.GetAllocatedString(
                        &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
                        &mut raw_name,
                        &mut size_name,
                    )
                }

                let device_name = {
                    let os_str = unsafe {
                        OsString::from_wide_ptr(&raw_name as *const u16, size_name as usize)
                    };
                    os_str.to_string_lossy().to_string()
                };

                // desc
                let mut raw_catagory = PWSTR::default();
                let mut size_catagory = 0_u32;
                unsafe {
                    devices.GetAllocatedString(
                        &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
                        &mut raw_catagory,
                        &mut size_catagory,
                    )
                }

                let device_catagory = {
                    let os_str = unsafe {
                        OsString::from_wide_ptr(&raw_catagory as *const u16, size_catagory as usize)
                    };
                    os_str.to_string_lossy().to_string()
                };

                // symbolic link
                let mut raw_lnk = PWSTR::default();
                let mut size_lnk = 0_u32;
                unsafe {
                    devices.GetAllocatedString(
                        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                        &mut raw_lnk,
                        &mut size_lnk,
                    )
                }

                let device_lnk = {
                    let os_str = unsafe {
                        OsString::from_wide_ptr(&raw_lnk as *const u16, size_lnk as usize)
                    };
                    os_str.to_string_lossy().to_string()
                };

                camera_info_vec.push(CameraInfo::new(
                    device_name,
                    device_catagory,
                    device_lnk,
                    device_index as usize,
                ));

                unsafe { CoTaskMemFree(&mut (raw_name as c_void)) }
            }
        }
        None => {
            return Ok(camera_info_vec);
        }
    }

    if let Some(mut devices) = devices_opt {
        if let Err(why) = unsafe { devices.DeleteAllItems() } {
            return Err(NokhwaError::GeneralError(format!(
                "Failed to free IMFActivate device list: {}",
                why.to_string()
            )));
        }
    }

    Ok(camera_info_vec)
}

#[cfg(not(feature = "input-msmf"))]
fn query_msmf() -> Result<Vec<CameraInfo>, NokhwaError> {
    Err(NokhwaError::UnsupportedOperation(
        CaptureAPIBackend::Windows,
    ))
}
