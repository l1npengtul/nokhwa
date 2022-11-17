/*
 * Copyright 2022 l1npengtul <l1npengtul@protonmail.com> / The Nokhwa Contributors
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

// Some assembly required. For developers 7 and up.

use clap::{Parser, Subcommand};
use flume::{Receiver, Sender};
use ggez::{
    event::{run, EventHandler},
    graphics::{Canvas, Image},
    Context, ContextBuilder, GameError,
};
use nokhwa::{
    native_api_backend,
    pixel_format::RgbFormat,
    query,
    utils::{
        frame_formats, CameraFormat, CameraIndex, FrameFormat, RequestedFormat,
        RequestedFormatType, Resolution,
    },
    Buffer, CallbackCamera, Camera,
};
use std::sync::Arc;

struct CaptureState {
    sender: Arc<Sender<Buffer>>,
    receiver: Arc<Receiver<Buffer>>,
    buffer: Vec<u8>,
    camera: CallbackCamera,
}

impl EventHandler<GameError> for CaptureState {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        self.receiver
            .recv()
            .map_err(|why| GameError::RenderError(why.to_string()))?
            .decode_image_to_buffer(&mut self.buffer)?;
        let image = Image::from_bytes(ctx, &self.buffer)?;
        let canvas = Canvas::from_image(ctx, image, None);
        canvas.finish(ctx)?;
        Ok(())
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

enum IndexKind {
    String(String),
    Index(u32),
}

#[derive(Subcommand)]
enum Commands {
    ListDevices,
    ListProperties {
        device: Option<IndexKind>,
        kind: PropertyKind,
    },
    Stream {
        device: Option<IndexKind>,
        display: bool,
        requested: Option<RequestedCliFormat>,
    },
    Single {
        device: Option<IndexKind>,
        save: Option<String>,
        requested: Option<RequestedCliFormat>,
    },
}

struct RequestedCliFormat {
    format_type: String,
    format_option: Option<String>,
}

enum PropertyKind {
    All,
    Controls,
    CompatibleFormats,
}

fn main() {
    nokhwa::nokhwa_initialize(|_| {
        println!("Nokhwa Initalized.");
        nokhwa_main()
    })
}

fn nokhwa_main() {
    let cli = Cli::parse();

    let cmd = match &cli.command {
        Some(cmd) => cmd,
        None => {
            println!("Unknown command \"\". Do --help for info.")
        }
    };

    match cmd {
        Commands::ListDevices => {
            let backend = native_api_backend().unwrap();
            let devices = query(backend).unwrap();
            println!("There are {} available cameras.", devices.len());
            for device in devices {
                println!("{device}");
            }
        }
        Commands::ListProperties { device, kind } => {
            let index = match device.unwrap_or(IndexKind::Index(0)) {
                IndexKind::String(s) => CameraIndex::String(s),
                IndexKind::Index(i) => CameraIndex::Index(i),
            };
            let mut camera = Camera::new(
                index,
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
            )
            .unwrap();
            match kind {
                PropertyKind::All => {
                    camera_print_controls(&camera);
                    camera_compatible_formats(&mut camera);
                }
                PropertyKind::Controls => {
                    camera_print_controls(&camera);
                }
                PropertyKind::CompatibleFormats => {
                    camera_compatible_formats(&mut camera);
                }
            }
        }
        Commands::Stream {
            device,
            display,
            requested,
        } => {
            let requested = match requested {
                Some(req) => match req.format_type.as_str() {
                    "HighestResolutionAbs" => RequestedFormat::new::<RgbFormat>(
                        RequestedFormatType::AbsoluteHighestResolution,
                    ),
                    "HighestFrameRateAbs" => RequestedFormat::new::<RgbFormat>(
                        RequestedFormatType::AbsoluteHighestFrameRate,
                    ),
                    "HighestResolution" => {
                        let values = req.format_option.unwrap().split(",").collect::<Vec<&str>>();
                        let x = values[0].parse::<u32>().unwrap();
                        let y = values[1].parse::<u32>().unwrap();
                        let resolution = Resolution::new(x, y);

                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(
                            resolution,
                        ))
                    }
                    "HighestFrameRate" => {
                        let fps = req.format_option.unwrap().parse::<u32>().unwrap();

                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestFrameRate(
                            fps,
                        ))
                    }
                    "Exact" => {
                        let values = req.format_option.unwrap().split(",").collect::<Vec<&str>>();
                        let x = values[0].parse::<u32>().unwrap();
                        let y = values[1].parse::<u32>().unwrap();
                        let fps = values[2].parse::<u32>().unwrap();
                        let fourcc = values[3].parse::<FrameFormat>().unwrap();

                        let resolution = Resolution::new(x, y);
                        let camera_format = CameraFormat::new(resolution, fourcc, fps);
                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(camera_format))
                    }
                    "Closest" => {
                        let values = req.format_option.unwrap().split(",").collect::<Vec<&str>>();
                        let x = values[0].parse::<u32>().unwrap();
                        let y = values[1].parse::<u32>().unwrap();
                        let fps = values[2].parse::<u32>().unwrap();
                        let fourcc = values[3].parse::<FrameFormat>().unwrap();

                        let resolution = Resolution::new(x, y);
                        let camera_format = CameraFormat::new(resolution, fourcc, fps);
                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(
                            camera_format,
                        ))
                    }
                    "None" => RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
                    _ => {
                        println!("Expected HighestResolutionAbs, HighestFrameRateAbs, HighestResolution, HighestFrameRate, Exact, Closest, or None");
                        return;
                    }
                },
                None => RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
            };

            let index = match device.unwrap_or(IndexKind::Index(0)) {
                IndexKind::String(s) => CameraIndex::String(s),
                IndexKind::Index(i) => CameraIndex::Index(i),
            };

            if display {
                let (sender, receiver) = flume::unbounded();
                let (sender, receiver) = (Arc::new(sender), Arc::new(receiver));
                let sender_clone = sender.clone();

                let mut camera = CallbackCamera::new(index, requested, move |buf| {
                    sender_clone.send(buf).expect("Error sending frame!!!!");
                })
                .unwrap();

                let camera_info = camera.info().unwrap().clone();

                camera.open_stream().unwrap();

                let state = CaptureState {
                    sender,
                    receiver,
                    buffer: Vec::with_capacity(3840 * 2160 * 3),
                    camera,
                };

                let cb = ContextBuilder::new(&camera_info.human_name(), "Nokhwa");
                let (ctx, el) = cb.build().unwrap();
                run(ctx, el, state)
            } else {
                let mut cb = CallbackCamera::new(index, requested, |buf| {
                    println!("Captured frame of size {}", buf.buffer().len());
                })
                .unwrap();

                cb.open_stream().unwrap();
            }
        }
        Commands::Single {
            device,
            save,
            requested,
        } => {
            let index = match device.unwrap_or(IndexKind::Index(0)) {
                IndexKind::String(s) => CameraIndex::String(s),
                IndexKind::Index(i) => CameraIndex::Index(i),
            };
            let requested = match requested {
                Some(req) => match req.format_type.as_str() {
                    "HighestResolutionAbs" => RequestedFormat::new::<RgbFormat>(
                        RequestedFormatType::AbsoluteHighestResolution,
                    ),
                    "HighestFrameRateAbs" => RequestedFormat::new::<RgbFormat>(
                        RequestedFormatType::AbsoluteHighestFrameRate,
                    ),
                    "HighestResolution" => {
                        let values = req.format_option.unwrap().split(",").collect::<Vec<&str>>();
                        let x = values[0].parse::<u32>().unwrap();
                        let y = values[1].parse::<u32>().unwrap();
                        let resolution = Resolution::new(x, y);

                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(
                            resolution,
                        ))
                    }
                    "HighestFrameRate" => {
                        let fps = req.format_option.unwrap().parse::<u32>().unwrap();

                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestFrameRate(
                            fps,
                        ))
                    }
                    "Exact" => {
                        let values = req.format_option.unwrap().split(",").collect::<Vec<&str>>();
                        let x = values[0].parse::<u32>().unwrap();
                        let y = values[1].parse::<u32>().unwrap();
                        let fps = values[2].parse::<u32>().unwrap();
                        let fourcc = values[3].parse::<FrameFormat>().unwrap();

                        let resolution = Resolution::new(x, y);
                        let camera_format = CameraFormat::new(resolution, fourcc, fps);
                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(camera_format))
                    }
                    "Closest" => {
                        let values = req.format_option.unwrap().split(",").collect::<Vec<&str>>();
                        let x = values[0].parse::<u32>().unwrap();
                        let y = values[1].parse::<u32>().unwrap();
                        let fps = values[2].parse::<u32>().unwrap();
                        let fourcc = values[3].parse::<FrameFormat>().unwrap();

                        let resolution = Resolution::new(x, y);
                        let camera_format = CameraFormat::new(resolution, fourcc, fps);
                        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(
                            camera_format,
                        ))
                    }
                    "None" => RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
                    _ => {
                        println!("Expected HighestResolutionAbs, HighestFrameRateAbs, HighestResolution, HighestFrameRate, Exact, Closest, or None");
                        return;
                    }
                },
                None => RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
            };

            let mut camera = Camera::new(index, requested).unwrap();

            let frame = camera.frame().unwrap();
            println!("Captured Single Frame of {}", frame.buffer().len());
            let decoded = frame.decode_image::<RgbFormat>().unwrap();
            println!("Decoded Frame of {}", decoded.len());

            if let Some(path) = save {
                println!("Saving to {path}");
                decoded.save(path).unwrap();
            }
        }
    }
}

fn camera_print_controls(cam: &Camera) {
    let ctrls = cam.camera_controls().unwrap();
    let index = cam.index();
    println!("Controls for camera {index}");
    for ctrl in ctrls {
        println!("{ctrl}")
    }
}

fn camera_compatible_formats(cam: &mut Camera) {
    for ffmt in frame_formats() {
        if let Ok(compatible) = cam.compatible_list_by_resolution(*ffmt) {
            for (resolution, fps) in compatible {
                println!("{ffmt}:");
                println!("    {resolution}: {fps:?}");
            }
        }
    }
}
