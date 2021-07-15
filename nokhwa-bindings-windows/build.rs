#[cfg(all(windows, not(feature = "docs-only")))]
extern crate bindgen;
extern crate cc;

use std::{env, path::PathBuf};

#[cfg(all(windows, not(feature = "docs-only")))]
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=NokhwaBindingsWindowsCXX\\NokhwaBindingsWindowsCXX\\*");
    cc::Build::new()
        .cpp(true)
        .include("NokhwaBindingsWindowsCXX\\NokhwaBindingsWindowsCXX\\NokhwaBindingsWindowsCXX.h")
        .file("NokhwaBindingsWindowsCXX\\NokhwaBindingsWindowsCXX\\NokhwaBindingsWindowsCXX.cpp")
        .object("uuid.lib")
        .object("mf.lib")
        .object("mfplat.lib")
        .object("mfreadwrite.lib")
        .object("mfuuid.lib")
        .object("shlwapi.lib")
        .object("Windows.lib")
        .compile("nokhwacxx");

    println!("cargo:rustc-link-lib=nokhwacxx.o");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_function("Nokhwa*")
        .allowlist_type("Nokhwa*")
        .allowlist_type("NOKHWA*")
        .generate()
        .expect("Failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("nokhwa_bindings.rs"))
        .expect("Failed to write bindings");
}

#[cfg(any(not(windows), feature = "docs-only"))]
fn main() {}
