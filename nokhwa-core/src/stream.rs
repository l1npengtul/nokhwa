use crate::error::NokhwaError;
use crate::frame_buffer::FrameBuffer;
use crate::types::CameraFormat;
use flume::{Receiver, Sender, TryRecvError};
use std::cell::Cell;
use std::sync::Arc;
use std::time::Duration;
use typed_builder::TypedBuilder;

/// What receiving behaviour the stream should observe.
///
/// Note that [`StreamHandleTrait::poll_frame`] does not respect [`StreamReceiverBehaviour::Timeout`] -
/// it will either immediately return (try once) or block until the next frame or error.
///
/// The default behaviour is to block until a new event is sent.
#[derive(Clone, Debug, Default, PartialOrd, PartialEq)]
pub enum StreamReceiverBehaviour {
    /// Blocks until a new event is sent to the Stream.
    #[default]
    Blocking,
    /// Only waits [duration] amount of time for a new event, returning [`Event::NotReady`] otherwise.
    Timeout(Duration),
    /// Immediately return. If there is no event waiting for the stream, it will return an [`Event::NotReady`] instead.
    Try,
}

/// How many events a stream can hold. By default, it is **one**.
///
/// This means that streams will be blocked until the stream handle is emptied.
#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum StreamBounds {
    Bounded(u32),
    Unbounded,
}

impl Default for StreamBounds {
    fn default() -> Self {
        StreamBounds::Bounded(1)
    }
}

#[derive(Clone, Default, Debug, PartialOrd, PartialEq)]
pub enum ControlFlowOnOther {
    Continue,
    #[default]
    Break,
}

/// Configuration for a [`StreamHandle`].
#[derive(Clone, Debug, Default, PartialOrd, PartialEq, TypedBuilder)]
pub struct StreamConfiguration {
    #[builder(default)]
    pub receiver: StreamReceiverBehaviour,
    #[builder(default)]
    pub bound: StreamBounds,
    #[builder(default)]
    pub on_other: ControlFlowOnOther,
}

/// Possible events to receive from an active stream.
#[derive(Clone, Debug, PartialEq)]
pub enum Event<'a> {
    /// A new frame.
    NewFrame(FrameBuffer<'a>),
    /// Camera Format Changed.
    ///
    /// This will usually require the reset of a buffer, or be followed by a [`Event::Terminated`],
    /// depending on the backend used.
    FormatChange(CameraFormat),
    /// This stream is not ready for another event. This is **never** sent by the stream itself, but
    /// instead a [`StreamHandle`] construct for when the user sets [`StreamReceiverBehaviour`] to either
    /// [`StreamReceiverBehaviour::Timeout`] or [`StreamReceiverBehaviour::Try`] but the stream does not
    /// have the data ready.
    ///
    /// (This can be ignored when iterating, or using the [`StreamReceiverBehaviour::Blocking`] approach.)
    NotReady,
    /// The stream will be ended shortly. Users should call [`StreamHandleTrait::close_stream`] afterwards.
    Terminating,
    /// The stream is closed.
    Closed,
    /// An error from the driver
    Error(String),
    /// Some other message sent by the driver. This can be ignored, although logging this is preferable.
    Other(String),
}

/// Represents a handle to a currently open stream.
///
/// Streams are only valid as long as the camera is live. Any Stream that is living past a camera
/// is invalid to use. (This doesn't cause UB, it will just kindly tell you that the stream has
/// already closed.)
///
/// Streams may unexpectedly close due to unforeseen consequences e.g. webcam undergoes spontaneous
/// deconstruction.
///
/// The async methods [`StreamHandle::poll_event`] and [`StreamHandle::poll_frame`] **do not** respect the [`StreamReceiverBehaviour`] setting.
///
/// You may also close the stream from the handle side using
#[derive(Debug)]
pub struct StreamHandle<'a> {
    frame: Receiver<Event<'a>>,
    control: Arc<Sender<()>>,
    configuration: StreamConfiguration,
    format: Cell<CameraFormat>,
}

impl<'a> StreamHandle<'a> {
    /// You shouldn't be here.
    #[must_use]
    pub fn new(
        recv: Receiver<Event<'a>>,
        control: Arc<Sender<()>>,
        configuration: StreamConfiguration,
        format: CameraFormat,
    ) -> Self {
        Self {
            frame: recv,
            control,
            configuration,
            format: Cell::new(format),
        }
    }

    pub fn configuration(&self) -> &StreamConfiguration {
        &self.configuration
    }

    pub fn format(&self) -> CameraFormat {
        self.format.get()
    }

    pub fn next_event(&self) -> Result<Event<'_>, NokhwaError> {
        let event = match self.configuration.receiver {
            StreamReceiverBehaviour::Blocking => {
                self.frame.recv().unwrap_or_else(|_| Event::Closed)
            }
            StreamReceiverBehaviour::Timeout(time) => self
                .frame
                .recv_timeout(time)
                .unwrap_or_else(|_| Event::NotReady),
            StreamReceiverBehaviour::Try => self.frame.try_recv().unwrap_or_else(|why| match why {
                TryRecvError::Empty => Event::NotReady,
                TryRecvError::Disconnected => Event::Closed,
            }),
        };

        if let Event::FormatChange(fmt) = event {
            self.format.set(fmt);
        }

        Ok(event)
    }

    pub fn next_frame(&self) -> Result<FrameBuffer<'_>, NokhwaError> {
        loop {
            let event = self.next_event()?;
            match event {
                Event::NewFrame(f) => return Ok(f),
                Event::Terminating | Event::Closed => {
                    let _ = self.control.try_send(());
                    return Err(NokhwaError::ReadFrameError("Stream Closed.".to_string()));
                }
                Event::Other(why) => {
                    if self.configuration.on_other == ControlFlowOnOther::Break {
                        return Err(NokhwaError::ReadFrameError(why));
                    }
                }
                Event::Error(e) => return Err(NokhwaError::ReadFrameError(e.clone())),
                _ => {}
            }
        }
    }

    #[cfg(feature = "async")]
    pub async fn poll_event(&self) -> Result<Event<'_>, NokhwaError> {
        Ok(self.frame.recv_async().await.map_or_else(
            |_| Event::Closed,
            |e| {
                if let Event::FormatChange(fmt) = e {
                    self.format.set(fmt);
                }
                e
            },
        ))
    }

    // TODO: a smarter implementation? maybe?
    #[cfg(feature = "async")]
    pub async fn poll_next_frame(&self) -> Result<FrameBuffer<'_>, NokhwaError> {
        loop {
            let event = self.poll_event().await?;
            match event {
                Event::NewFrame(f) => return Ok(f),
                Event::Terminating | Event::Closed => {
                    let _ = self.control.try_send(());
                    return Err(NokhwaError::ReadFrameError("Stream Closed.".to_string()));
                }
                Event::Other(why) => {
                    if let ControlFlowOnOther::Break = self.configuration.on_other {
                        return Err(NokhwaError::ReadFrameError(why));
                    }
                }
                _ => {}
            }
        }
    }
}

impl Drop for StreamHandle<'_> {
    fn drop(&mut self) {
        let _ = self.control.try_send(());
    }
}
