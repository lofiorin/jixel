//! Builds libjxl (and its bundled dependencies: brotli, highway, skcms) from
//! the `libjxl` git submodule as static libraries and links them.
//!
//! Run `scripts/bootstrap.sh` from the repository root first, so the submodule
//! and the three third-party submodules it needs are checked out.
//!
//! Every decision here is based on the *target* platform (read from the
//! `CARGO_CFG_*` variables), never on the host, so cross-compiling for iOS and
//! Android from macOS works.

use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest_dir.join("libjxl");
    assert!(
        source.join("third_party/highway/CMakeLists.txt").exists(),
        "libjxl sources are missing. Run `scripts/bootstrap.sh` from the repository root."
    );

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or_default();

    let mut config = cmake::Config::new(&source);
    config
        // Release build even for `cargo build` (debug libjxl is ~20x slower).
        .profile("Release")
        // CMake >= 4 refuses projects that declare very old minimum versions;
        // some bundled third-party projects still do.
        .define("CMAKE_POLICY_VERSION_MINIMUM", "3.5")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_TESTING", "OFF")
        .define("JPEGXL_STATIC", "OFF")
        .define("JPEGXL_ENABLE_TOOLS", "OFF")
        .define("JPEGXL_ENABLE_DEVTOOLS", "OFF")
        .define("JPEGXL_ENABLE_BENCHMARK", "OFF")
        .define("JPEGXL_ENABLE_EXAMPLES", "OFF")
        .define("JPEGXL_ENABLE_FUZZERS", "OFF")
        .define("JPEGXL_ENABLE_VIEWERS", "OFF")
        .define("JPEGXL_ENABLE_PLUGINS", "OFF")
        .define("JPEGXL_ENABLE_DOXYGEN", "OFF")
        .define("JPEGXL_ENABLE_MANPAGES", "OFF")
        .define("JPEGXL_ENABLE_JNI", "OFF")
        .define("JPEGXL_ENABLE_SJPEG", "OFF")
        .define("JPEGXL_ENABLE_OPENEXR", "OFF")
        .define("JPEGXL_ENABLE_JPEGLI", "OFF")
        .define("JPEGXL_ENABLE_SKCMS", "ON")
        .define("JPEGXL_BUNDLE_LIBPNG", "OFF")
        .define("JPEGXL_FORCE_SYSTEM_BROTLI", "OFF")
        .define("JPEGXL_FORCE_SYSTEM_HWY", "OFF");

    if let Ok(jobs) = std::thread::available_parallelism() {
        config.env("CMAKE_BUILD_PARALLEL_LEVEL", jobs.to_string());
    }

    if target_os == "android" {
        // The NDK ships an official CMake toolchain file; point CMake at it.
        let ndk = env::var("ANDROID_NDK_HOME")
            .or_else(|_| env::var("ANDROID_NDK_ROOT"))
            .or_else(|_| env::var("ANDROID_NDK"))
            .expect("Set ANDROID_NDK_HOME to build for Android");
        let abi = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "aarch64" => "arm64-v8a",
            "arm" => "armeabi-v7a",
            "x86_64" => "x86_64",
            "x86" => "x86",
            other => panic!("unsupported Android architecture: {other}"),
        };
        config
            .define(
                "CMAKE_TOOLCHAIN_FILE",
                PathBuf::from(ndk).join("build/cmake/android.toolchain.cmake"),
            )
            .define("ANDROID_ABI", abi)
            .define("ANDROID_PLATFORM", "android-24")
            .define("ANDROID_STL", "c++_static");
    }

    if target_env == "msvc" {
        // highway and libjxl are best supported by clang-cl on Windows, and we
        // link the static CRT to match Rust's default for static libraries.
        config
            .generator_toolset("ClangCL")
            .define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded")
            .cflag("/Zl");
    }

    let prefix = config.build();
    for dir in ["lib", "lib64"] {
        let lib_dir = prefix.join(dir);
        if lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", lib_dir.display());
        }
    }

    for lib in [
        "jxl",
        "jxl_cms",
        "jxl_threads",
        "hwy",
        "brotlienc",
        "brotlidec",
        "brotlicommon",
    ] {
        println!("cargo:rustc-link-lib=static={lib}");
    }

    // libjxl is C++: link the C++ standard library of the *target*.
    if target_vendor == "apple" || target_os == "freebsd" {
        println!("cargo:rustc-link-lib=c++");
    } else if target_os == "android" {
        println!("cargo:rustc-link-lib=c++_static");
        println!("cargo:rustc-link-lib=c++abi");
    } else if target_os == "linux" || target_env == "gnu" {
        println!("cargo:rustc-link-lib=stdc++");
    }

    println!("cargo:include={}", source.join("lib/include").display());
    println!("cargo:rerun-if-changed=build.rs");
}
