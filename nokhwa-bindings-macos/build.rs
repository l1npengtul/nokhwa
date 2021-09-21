#[cfg(any(target_os = "macos", target_os = "ios"))]
fn main() {
    println!("cargo:rustc-link-lib=framework=CoreMedia");
    println!("cargo:rustc-link-lib=framework=AVFoundation");
}

#[cfg(not(any(target_os = "macos", target_os = "ios")))]
fn main() {}
