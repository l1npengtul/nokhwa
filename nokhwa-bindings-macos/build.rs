use bindgen::Builder;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "macos")]
fn main() {
    // link our frameworks
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
    println!("cargo:rustc-link-lib=framework=AVFoundation");
    println!("cargo:rustc-link-lib=framework=CoreMedia");

    let sdk_directory = {
        let output = Command::new("xcrun")
            .args(&["--show-sdk-path"])
            .output()
            .expect("Failed to get SDK path command")
            .stdout;
        let prefix_str = std::str::from_utf8(&output).expect("invalid output from `xcrun`");
        prefix_str.trim_end().to_string()
    };

    let clang_args = vec!["-x", "objective-c", "-fblocks", "-isysroot", &sdk_directory];

    let bindings = Builder::default()
        .clang_args(&clang_args)
        .objc_extern_crate(true)
        .block_extern_crate(true)
        .generate_block(true)
        .rustfmt_bindings(true)
        // time.h as has a variable called timezone that conflicts with some of the objective-c
        // calls from NSCalendar.h in the Foundation framework. This removes that one variable.
        .blocklist_item("timezone")
        // https://github.com/rust-lang/rust-bindgen/issues/1705
        .blocklist_item("objc_object")
        .header_contents("CoreMedia.h", "#include <CoreMedia/CoreMedia.h>")
        .header_contents("AVFoundation.h", "#include <AVFoundation/AVFoundation.h>")
        .generate()
        .expect("Failed to generate bindings");

    let output_directory = PathBuf::from(std::env::var("OUT_DIR").expect("No envvar OUT_DIR"));

    // Write them to the crate root.
    bindings
        .write_to_file(output_directory.join("bindings.rs"))
        .expect("could not write bindings");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
