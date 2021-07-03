#[cfg(windows)]
fn main() {
    windows::build!(
        Windows::Win32::Media::MediaFoundation::*,
        Windows::Win32::System::Com::{CoTaskMemFree},
    )
}

#[cfg(not(windows))]
fn main() {}
