//! A build dependency for running `zig` to build a native library
//!
//! This crate provides some necessary boilerplate and shim support for running
//! the system `zig` command to build a native library.
//!
//! ## Examples
//!
//! ```no_run
//! use zig;
//!
//! // Builds the project in the directory located in `libfoo`, installing it
//! // into $OUT_DIR
//! let dst = zig::build("libfoo");
//!
//! println!("cargo:rustc-link-search=native={}", dst.display());
//! println!("cargo:rustc-link-lib=static=foo");
//! ```

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Config {
    path: PathBuf,
    defines: Vec<(String, String)>,
    optimize: Option<String>,
}

/// Builds the native library rooted at `path` with the default zig options.
/// This will return the directory in which the library was installed.
///
/// # Examples
///
/// ```no_run
/// use zig;
///
/// // Builds the project in the directory located in `libfoo`, installing it
/// // into $OUT_DIR
/// let dst = zig::build("libfoo");
///
/// println!("cargo:rustc-link-search=native={}", dst.display());
/// println!("cargo:rustc-link-lib=static=foo");
/// ```
///
pub fn build<P: AsRef<Path>>(path: P) -> PathBuf {
    Config::new(path.as_ref()).build()
}

impl Config {
    pub fn build(&mut self) -> PathBuf {
        let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "x86" | "i686" | "i586" | "i386" => "x86".to_string(),
            s => s.to_string(),
        };

        // Map Rust OS to Zig OS, handling special cases like wasm
        let os = match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
            // wasm32-unknown-unknown has OS "unknown", which should map to zig's "freestanding"
            "unknown" => "freestanding".to_string(),
            s => s.to_string(),
        };

        let target = env::var("TARGET").unwrap();
        let abi = if target.contains("msvc") {
            Some("msvc")
        } else if target.contains("musleabihf") {
            Some("musleabihf")
        } else if target.contains("musleabi") {
            Some("musleabi")
        } else if target.contains("musl") {
            Some("musl")
        } else if target.contains("gnueabihf") {
            Some("gnueabihf")
        } else if target.contains("gnueabi") {
            Some("gnueabi")
        } else if target.contains("gnu") {
            Some("gnu")
        } else {
            // macOS, iOS, FreeBSD, etc. don't need an explicit ABI
            None
        };

        let optimize = match self.optimize {
            Some(ref s) => s.clone(),
            None => {
                let profile = match std::env::var("PROFILE").unwrap().as_str() {
                    "debug" => "Debug",
                    _ => "Release",
                };

                let opt_level = match std::env::var("OPT_LEVEL").unwrap().as_str() {
                    "0" | "1" => "Safe",
                    "2" | "3" => "Fast",
                    "s" | "z" => "Small",
                    _ => "Safe",
                };

                match profile {
                    "Release" => format!("Release{}", opt_level),
                    s => s.to_string(),
                }
            }
        };

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        let dst = out_path.join("lib");
        let cache = out_path.join("cache");
        let mut cmd = Command::new("zig");
        cmd.env("ZIG_GLOBAL_CACHE_DIR", cache.to_str().unwrap());
        cmd.env("ZIG_LOCAL_CACHE_DIR", cache.to_str().unwrap());
        cmd.current_dir(self.path.clone());
        cmd.arg("build");
        cmd.arg("--prefix");
        cmd.arg(out_path.display().to_string());
        let zig_target = match abi {
            Some(abi) => format!("{}-{}-{}", arch, os, abi),
            None => format!("{}-{}", arch, os),
        };
        cmd.arg(format!("-Dtarget={}", zig_target));
        cmd.arg(format!("-Doptimize={}", optimize));
        self.defines.iter().for_each(|(k, v)| {
            cmd.arg(format!("-D{}={}", k, v));
        });

        match cmd.status() {
            Ok(status) => {
                if !status.success() {
                    panic!("zig build failed");
                }
            }
            Err(e) => {
                panic!("failed to execute zig build: {}", e);
            }
        }

        println!("cargo:root={}", dst.display());
        dst
    }

    /// Adds a new `-D` flag to pass to zig.
    pub fn define<K, V>(&mut self, key: &str, value: &str) -> &mut Config {
        self.defines.push((key.to_string(), value.to_string()));
        self
    }

    /// Sets the optimization level for the build.
    pub fn optimize(&mut self, level: &str) -> &mut Config {
        self.optimize = Some(level.to_string());
        self
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Config {
        Config {
            path: env::current_dir().unwrap().join(path),
            defines: Vec::new(),
            optimize: None,
        }
    }
}
