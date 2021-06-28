use crate::{Camera, NokhwaError};
use flume::{Receiver, Sender};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub struct ThreadedCamera {
    camera: Arc<Mutex<Camera>>,
    thread_handle: JoinHandle<_>,
    receiver: Receiver<Result<Vec<u8>, NokhwaError>>,
    sender: Sender<Result<Vec<u8>, NokhwaError>>,
}

impl ThreadedCamera {}

fn capture(camera: Arc<Mutex<Camera>>) {}
