use crate::{CameraFormat, CameraInfo, NokhwaError};
use flume::{Receiver, Sender};
use ouroboros::self_referencing;
use std::sync::{atomic::AtomicUsize, Arc};
use uvc::{ActiveStream, Context, Device, DeviceHandle, StreamHandle, Error, DeviceList};

// ignore the IDE.
/// The backend struct that interfaces with libuvc.
/// To see what this does, please see [`CaptureBackendTrait`]
/// # Quirks
/// The indexing for this backend is based off of `libuvc`'s device ordering, not the OS.
/// # Safety
/// This backend requires use of `unsafe` due to the self-referencing structs involved.
#[self_referencing(chain_hack, pub_extras)]
pub struct UVCCaptureDevice<'a> {
    camera_format: Option<CameraFormat>,
    camera_info: CameraInfo,
    device_receiver: Box<Receiver<Vec<u8>>>,
    device_sender: Box<Sender<Vec<u8>>>,
    context: Box<Context<'a>>,
    #[borrows(context)]
    #[not_covariant]
    device: Box<Option<Device<'this>>>,
    #[borrows(device)]
    #[not_covariant]
    device_handle: Box<Option<DeviceHandle<'this>>>,
    #[borrows(device_handle)]
    #[not_covariant]
    stream_handle: Box<Option<StreamHandle<'this>>>,
    #[borrows(stream_handle)]
    #[not_covariant]
    active_stream: Box<Option<ActiveStream<'this, Arc<AtomicUsize>>>>,
}

impl<'a> UVCCaptureDevice<'a> {
    pub fn new(index: usize) -> Result<Self, NokhwaError> {
        let context = match Context::new() {
            Ok(ctx) => Box::new(ctx),
            Err(why) => return Err(NokhwaError::CouldntOpenDevice(why.to_string())),
        };
        
        let device = match context.devices() {
            Ok(device_list) => {
                for (idx, dev) in device_list.enumerate() {
                    if idx == index {
                    }
                }
            }
            Err(why) => return Err(NokhwaError::CouldntOpenDevice(why.to_string())),
        };
    }
}
