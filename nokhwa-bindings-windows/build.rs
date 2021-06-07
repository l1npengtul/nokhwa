#[cfg(windows)]
fn main() {
    windows::build!(
        Windows::Win32::Media::MediaFoundation::*
    )
}

#[cfg(not(windows))]
fn main() {}
