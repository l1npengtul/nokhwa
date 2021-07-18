#[cfg(all(windows, not(feature = "docs-only")))]
fn main() {
    windows::build!(
        Windows::Win32::Media::MediaFoundation::*,
        Windows::Win32::System::Com::{CoTaskMemFree, CoInitializeEx, COINIT, CoUninitialize},
        Windows::Win32::Foundation::{S_OK},
        Windows::Win32::Graphics::DirectShow::*,
    )
}

#[cfg(any(not(windows), feature = "docs-only"))]
fn main() {}
