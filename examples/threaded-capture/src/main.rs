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

use image::{ImageBuffer, Rgb};
use nokhwa::{query_devices, CallbackCamera, CaptureAPIBackend};
use std::time::Duration;

fn main() {
    let cameras = query_devices(CaptureAPIBackend::Auto).unwrap();
    cameras.iter().for_each(|cam| println!("{:?}", cam));

    let mut threaded = CallbackCamera::new(0, None).unwrap();
    threaded.open_stream(callback).unwrap();
    #[allow(clippy::empty_loop)] // keep it running
    loop {
        let frame = threaded.poll_frame().unwrap();
        println!(
            "{}x{} {} naripoggers",
            frame.width(),
            frame.height(),
            frame.len()
        );
    }
}

fn callback(image: ImageBuffer<Rgb<u8>, Vec<u8>>) {
    println!("{}x{} {}", image.width(), image.height(), image.len());
}
