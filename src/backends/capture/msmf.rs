/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
    CameraControl, CameraFormat, CameraInfo, CaptureAPIBackend, CaptureBackendTrait, FrameFormat,
    KnownCameraControls, NokhwaError, Resolution,
};
use image::{ImageBuffer, Rgb};
use nokhwa_bindings_windows::wmf::MediaFoundationDevice;
use nokhwa_bindings_windows::MediaFoundationControls;
use std::{
    any::Any,
    borrow::Cow,
    cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut},
    collections::HashMap,
};

pub struct MediaFoundationCaptureDevice {
    inner: RefCell<MediaFoundationDevice>,
}

impl MediaFoundationCaptureDevice {
    pub fn new(index: usize, camera_fmt: Option<CameraFormat>) -> Result<Self, NokhwaError> {
        let mut mf_device = MediaFoundationDevice::new(index)?;
        if let Some(fmt) = camera_fmt {
            mf_device.set_format(fmt.into())?;
        }
        Ok(MediaFoundationCaptureDevice {
            inner: RefCell::new(mf_device),
        })
    }
}

impl CaptureBackendTrait for MediaFoundationCaptureDevice {
    fn backend(&self) -> CaptureAPIBackend {
        CaptureAPIBackend::MediaFoundation
    }

    fn camera_info(&self) -> CameraInfo {
        let inner_borrow = self.inner.borrow();
        CameraInfo::new(
            inner_borrow.name(),
            "".to_string(),
            inner_borrow.symlink(),
            inner_borrow.index(),
        )
    }

    fn camera_format(&self) -> CameraFormat {
        self.inner.format().into()
    }

    fn set_camera_format(&mut self, new_fmt: CameraFormat) -> Result<(), NokhwaError> {
        if let Err(why) = self.inner.borrow_mut().set_format(new_fmt.into()) {
            Err(why.into())
        }
        Ok(())
    }

    fn compatible_list_by_resolution(
        &self,
        fourcc: FrameFormat,
    ) -> Result<HashMap<Resolution, Vec<u32>>, NokhwaError> {
        let inner_borrow = match self.inner.try_borrow_mut() {
            Ok(mut brw) => (&mut brw),
            Err(why) => {
                return Err(NokhwaError::GetPropertyError {
                    property: "Device".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let mf_camera_format_list = inner_borrow.compatible_format_list()?;
        let mut resolution_map = HashMap::new();

        for mf_camera_format in mf_camera_format_list {
            let camera_format: CameraFormat = mf_camera_format.into();

            // check fcc
            if camera_format.format() != fourcc {
                continue;
            }

            match resolution_map.get_mut(&camera_format.resolution()) {
                Some(fps_list) => {
                    fps_list.append(camera_format.framerate());
                }
                None => {
                    if let Some(mut wtf_why_we_here_list) = resolution_map
                        .insert(camera_format.resolution(), vec![camera_format.framerate()])
                    {
                        wtf_why_we_here_list.push(camera_format.framerate());
                        resolution_map.insert(camera_format.resolution(), wtf_why_we_here_list);
                    }
                }
            }
        }
        Ok(resolution_map)
    }

    fn compatible_fourcc(&mut self) -> Result<Vec<FrameFormat>, NokhwaError> {
        let inner_borrow = match self.inner.try_borrow_mut() {
            Ok(mut brw) => (&mut brw),
            Err(why) => {
                return Err(NokhwaError::GetPropertyError {
                    property: "Device".to_string(),
                    error: why.to_string(),
                })
            }
        };

        let mf_camera_format_list = inner_borrow.compatible_format_list()?;
        let mut frame_format_list = vec![];

        for mf_camera_format in mf_camera_format_list {
            let camera_format: CameraFormat = mf_camera_format.into();

            if !frame_format_list.contains(&camera_format.format()) {
                frame_format_list.push(camera_format.format())
            }

            // TODO: Update as we get more frame formats!
            if frame_format_list.len() == 2 {
                break;
            }
        }
        Ok(frame_format_list)
    }

    fn resolution(&self) -> Resolution {
        self.camera_format().resolution()
    }

    fn set_resolution(&mut self, new_res: Resolution) -> Result<(), NokhwaError> {
        let mut new_format = self.camera_format();
        new_format.set_resolution(new_res);
        self.set_camera_format(new_format)
    }

    fn frame_rate(&self) -> u32 {
        self.camera_format().framerate()
    }

    fn set_frame_rate(&mut self, new_fps: u32) -> Result<(), NokhwaError> {
        let mut new_format = self.camera_format();
        new_format.set_framerate(new_fps);
        self.set_camera_format(new_format)
    }

    fn frame_format(&self) -> FrameFormat {
        self.camera_format().format()
    }

    fn set_frame_format(&mut self, fourcc: FrameFormat) -> Result<(), NokhwaError> {
        let mut new_format = self.camera_format();
        new_format.set_format(fourcc);
        self.set_camera_format(new_format)
    }

    fn supported_camera_controls(&self) -> Result<Vec<KnownCameraControls>, NokhwaError> {
        // let inner_borrow = match self.inner.try_borrow() {
        //     Ok(brw) => (&*brw),
        //     Err(why) => {
        //         return return Err(NokhwaError::GetPropertyError {
        //             property: "Device".to_string(),
        //             error: why.to_string(),
        //         })
        //     }
        // };
        // let a = KnownCameraControls
        todo!()
    }

    fn camera_control(&self, control: KnownCameraControls) -> Result<CameraControl, NokhwaError> {
        todo!()
    }

    fn set_camera_control(&mut self, control: CameraControl) -> Result<(), NokhwaError> {
        todo!()
    }

    fn raw_supported_camera_controls(&self) -> Result<Vec<Box<dyn Any>>, NokhwaError> {
        todo!()
    }

    fn raw_camera_control(&self, control: &dyn Any) -> Result<Box<dyn Any>, NokhwaError> {
        todo!()
    }

    fn set_raw_camera_control(
        &mut self,
        control: &dyn Any,
        value: &dyn Any,
    ) -> Result<(), NokhwaError> {
        todo!()
    }

    fn open_stream(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }

    fn is_stream_open(&self) -> bool {
        todo!()
    }

    fn frame(&mut self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, NokhwaError> {
        todo!()
    }

    fn frame_raw(&mut self) -> Result<Cow<[u8]>, NokhwaError> {
        todo!()
    }

    fn stop_stream(&mut self) -> Result<(), NokhwaError> {
        todo!()
    }
}
