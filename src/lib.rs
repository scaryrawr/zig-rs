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

/// Configuration for a Zig build rooted at a project directory.
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
    /// Runs `zig build` and returns the directory containing installed artifacts.
    pub fn build(&mut self) -> PathBuf {
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
        let zig_target = zig_target_from_env().unwrap_or_else(|e| panic!("{}", e));
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
    pub fn define(&mut self, key: &str, value: &str) -> &mut Config {
        self.defines.push((key.to_string(), value.to_string()));
        self
    }

    /// Sets the optimization level for the build.
    pub fn optimize(&mut self, level: &str) -> &mut Config {
        self.optimize = Some(level.to_string());
        self
    }

    /// Creates a new Zig build configuration rooted at `path`.
    pub fn new<P: AsRef<Path>>(path: P) -> Config {
        Config {
            path: env::current_dir().unwrap().join(path),
            defines: Vec::new(),
            optimize: None,
        }
    }
}

fn zig_target_from_env() -> Result<String, String> {
    let target = env::var("TARGET").expect("TARGET must be set by Cargo");
    let arch =
        env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH must be set by Cargo");
    let os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS must be set by Cargo");
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let target_abi = env::var("CARGO_CFG_TARGET_ABI").unwrap_or_default();

    zig_target(&target, &arch, &os, &target_env, &target_abi)
}

fn zig_target(
    target: &str,
    rust_arch: &str,
    rust_os: &str,
    rust_env: &str,
    rust_abi: &str,
) -> Result<String, String> {
    let arch = zig_arch(target, rust_arch)?;
    let os = zig_os(rust_os, rust_env);

    match zig_abi(target, rust_os, rust_env, rust_abi)? {
        Some(abi) => Ok(format!("{}-{}-{}", arch, os, abi)),
        None => Ok(format!("{}-{}", arch, os)),
    }
}

fn zig_arch<'a>(target: &'a str, rust_arch: &'a str) -> Result<&'a str, String> {
    let triple_arch = target.split('-').next().unwrap_or(rust_arch);

    match triple_arch {
        "aarch64_be" | "amdgcn" | "armeb" | "bpfeb" | "bpfel" | "mips64el" | "mipsel"
        | "powerpc64le" => Ok(triple_arch),
        "arm64_32" => Err(unsupported_target(
            target,
            "Zig does not expose Rust's arm64_32 watchOS architecture",
        )),
        arch if arch.starts_with("mipsisa") => Err(unsupported_target(
            target,
            "Zig does not expose Rust's MIPS revision-specific architecture",
        )),
        arch if arch.starts_with("thumb") => Ok("thumb"),
        "i386" | "i586" | "i686" => Ok("x86"),
        _ => match rust_arch {
            "amdgpu" => Ok("amdgcn"),
            "i386" | "i586" | "i686" | "x86" => Ok("x86"),
            _ => Ok(rust_arch),
        },
    }
}

fn zig_os<'a>(rust_os: &'a str, rust_env: &str) -> &'a str {
    match (rust_os, rust_env) {
        ("android", _) => "linux",
        ("ios", "macabi") => "maccatalyst",
        ("none" | "unknown", _) => "freestanding",
        _ => rust_os,
    }
}

fn zig_abi(
    target: &str,
    rust_os: &str,
    rust_env: &str,
    rust_abi: &str,
) -> Result<Option<&'static str>, String> {
    if rust_os == "android" {
        return Ok(Some(
            if rust_abi == "eabi" || target.ends_with("androideabi") {
                "androideabi"
            } else {
                "android"
            },
        ));
    }

    if rust_env == "sim" || rust_abi == "sim" {
        return Ok(Some("simulator"));
    }

    if rust_env == "macabi" || rust_abi == "macabi" || rust_os == "wasi" {
        return Ok(None);
    }

    match rust_env {
        "gnu" => prefixed_abi(target, "gnu", rust_abi),
        "musl" => prefixed_abi(target, "musl", rust_abi),
        "msvc" => Ok(Some("msvc")),
        "ohos" => match rust_abi {
            "" => Ok(Some("ohos")),
            "eabi" => Ok(Some("ohoseabi")),
            _ => Err(unsupported_abi(target, rust_env, rust_abi)),
        },
        "uclibc" => Err(unsupported_target(
            target,
            "Zig does not expose uclibc as a target ABI",
        )),
        "sgx" => Err(unsupported_target(
            target,
            "Zig does not expose Fortanix SGX as a target ABI",
        )),
        "newlib" => match bare_abi(rust_abi) {
            Some(abi) => Ok(Some(abi)),
            None => abi_from_target_suffix(target),
        },
        "mlibc" | "p1" | "p2" | "p3" => Ok(None),
        "" => match bare_abi(rust_abi) {
            Some(abi) => Ok(Some(abi)),
            None => abi_from_target_suffix(target),
        },
        _ if rust_abi.is_empty() => abi_from_target_suffix(target),
        _ => Err(unsupported_abi(target, rust_env, rust_abi)),
    }
}

fn prefixed_abi(
    target: &str,
    prefix: &'static str,
    rust_abi: &str,
) -> Result<Option<&'static str>, String> {
    match (prefix, rust_abi) {
        ("gnu", "") => Ok(Some("gnu")),
        ("musl", "") => Ok(Some("musl")),
        ("gnu", "llvm") => Ok(Some("gnu")),
        ("gnu", "eabi") => Ok(Some("gnueabi")),
        ("gnu", "eabihf") => Ok(Some("gnueabihf")),
        ("gnu", "abi64") => Ok(Some("gnuabi64")),
        ("gnu", "abin32") => Ok(Some("gnuabin32")),
        ("gnu", "x32") => Ok(Some("gnux32")),
        ("gnu", "f32") => Ok(Some("gnuf32")),
        ("gnu", "sf") => Ok(Some("gnusf")),
        ("musl", "eabi") => Ok(Some("musleabi")),
        ("musl", "eabihf") => Ok(Some("musleabihf")),
        ("musl", "abi64") => Ok(Some("muslabi64")),
        ("musl", "abin32") => Ok(Some("muslabin32")),
        ("musl", "x32") => Ok(Some("muslx32")),
        ("musl", "f32") => Ok(Some("muslf32")),
        ("musl", "sf") => Ok(Some("muslsf")),
        (_, "elfv1" | "elfv2") => Ok(Some(prefix)),
        (_, "ilp32") => Err(unsupported_target(
            target,
            "Zig does not expose Rust's Linux GNU ILP32 ABI",
        )),
        (_, "spe") => Err(unsupported_target(
            target,
            "Zig does not expose Rust's PowerPC SPE ABI",
        )),
        _ => Err(unsupported_abi(target, prefix, rust_abi)),
    }
}

fn bare_abi(rust_abi: &str) -> Option<&'static str> {
    match rust_abi {
        "eabi" => Some("eabi"),
        "eabihf" => Some("eabihf"),
        _ => None,
    }
}

fn abi_from_target_suffix(target: &str) -> Result<Option<&'static str>, String> {
    for (suffix, abi) in [
        ("-uclibceabihf", None),
        ("-uclibceabi", None),
        ("-uclibc", None),
        ("-gnu_ilp32", None),
        ("-gnuspe", None),
        ("-muslspe", None),
        ("-gnullvm", Some("gnu")),
        ("-musleabihf", Some("musleabihf")),
        ("-musleabi", Some("musleabi")),
        ("-muslabi64", Some("muslabi64")),
        ("-muslabin32", Some("muslabin32")),
        ("-muslx32", Some("muslx32")),
        ("-musl", Some("musl")),
        ("-gnueabihf", Some("gnueabihf")),
        ("-gnueabi", Some("gnueabi")),
        ("-gnuabi64", Some("gnuabi64")),
        ("-gnuabin32", Some("gnuabin32")),
        ("-gnux32", Some("gnux32")),
        ("-gnu", Some("gnu")),
        ("-msvc", Some("msvc")),
        ("-ohoseabi", Some("ohoseabi")),
        ("-ohos", Some("ohos")),
        ("-eabihf", Some("eabihf")),
        ("-eabi", Some("eabi")),
    ] {
        if target.ends_with(suffix) {
            return match abi {
                Some(abi) => Ok(Some(abi)),
                None => Err(unsupported_target(
                    target,
                    "Zig does not expose this Rust target ABI",
                )),
            };
        }
    }

    Ok(None)
}

fn unsupported_abi(target: &str, rust_env: &str, rust_abi: &str) -> String {
    unsupported_target(
        target,
        &format!(
            "unsupported target_env `{}` with target_abi `{}`",
            rust_env, rust_abi
        ),
    )
}

fn unsupported_target(target: &str, reason: &str) -> String {
    format!("unsupported Rust target `{}`: {}", target, reason)
}

#[cfg(test)]
mod tests {
    use super::zig_target;

    fn map(
        target: &str,
        arch: &str,
        os: &str,
        target_env: &str,
        abi: &str,
    ) -> Result<String, String> {
        zig_target(target, arch, os, target_env, abi)
    }

    #[test]
    fn maps_apple_targets() {
        assert_eq!(
            map("aarch64-apple-darwin", "aarch64", "macos", "", "").unwrap(),
            "aarch64-macos"
        );
        assert_eq!(
            map(
                "x86_64-apple-ios-macabi",
                "x86_64",
                "ios",
                "macabi",
                "macabi",
            )
            .unwrap(),
            "x86_64-maccatalyst"
        );
        assert_eq!(
            map("aarch64-apple-ios-sim", "aarch64", "ios", "sim", "sim").unwrap(),
            "aarch64-ios-simulator"
        );
    }

    #[test]
    fn maps_android_as_linux_with_android_abi() {
        assert_eq!(
            map("aarch64-linux-android", "aarch64", "android", "", "").unwrap(),
            "aarch64-linux-android"
        );
        assert_eq!(
            map("arm-linux-androideabi", "arm", "android", "", "eabi").unwrap(),
            "arm-linux-androideabi"
        );
    }

    #[test]
    fn maps_linux_abi_variants() {
        assert_eq!(
            map("aarch64-unknown-linux-gnu", "aarch64", "linux", "gnu", "",).unwrap(),
            "aarch64-linux-gnu"
        );
        assert_eq!(
            map(
                "arm-unknown-linux-musleabihf",
                "arm",
                "linux",
                "musl",
                "eabihf",
            )
            .unwrap(),
            "arm-linux-musleabihf"
        );
        assert_eq!(
            map(
                "mips64el-unknown-linux-gnuabi64",
                "mips64",
                "linux",
                "gnu",
                "abi64",
            )
            .unwrap(),
            "mips64el-linux-gnuabi64"
        );
        assert_eq!(
            map(
                "x86_64-unknown-linux-gnux32",
                "x86_64",
                "linux",
                "gnu",
                "x32",
            )
            .unwrap(),
            "x86_64-linux-gnux32"
        );
        assert_eq!(
            map(
                "powerpc64le-unknown-linux-gnu",
                "powerpc64",
                "linux",
                "gnu",
                "elfv2",
            )
            .unwrap(),
            "powerpc64le-linux-gnu"
        );
        assert_eq!(
            map("aarch64-unknown-linux-ohos", "aarch64", "linux", "ohos", "",).unwrap(),
            "aarch64-linux-ohos"
        );
        assert_eq!(
            map("armv7-unknown-linux-ohos", "arm", "linux", "ohos", "eabi").unwrap(),
            "arm-linux-ohoseabi"
        );
    }

    #[test]
    fn maps_non_linux_unix_variants() {
        assert_eq!(
            map(
                "aarch64-unknown-managarm-mlibc",
                "aarch64",
                "managarm",
                "mlibc",
                "",
            )
            .unwrap(),
            "aarch64-managarm"
        );
        assert_eq!(
            map(
                "armv7-sony-vita-newlibeabihf",
                "arm",
                "vita",
                "newlib",
                "eabihf",
            )
            .unwrap(),
            "arm-vita-eabihf"
        );
    }

    #[test]
    fn maps_windows_targets() {
        assert_eq!(
            map("i686-pc-windows-gnullvm", "x86", "windows", "gnu", "llvm").unwrap(),
            "x86-windows-gnu"
        );
        assert_eq!(
            map("x86_64-pc-windows-msvc", "x86_64", "windows", "msvc", "",).unwrap(),
            "x86_64-windows-msvc"
        );
    }

    #[test]
    fn maps_freestanding_wasi_and_arch_aliases() {
        assert_eq!(
            map("wasm32-unknown-unknown", "wasm32", "unknown", "", "").unwrap(),
            "wasm32-freestanding"
        );
        assert_eq!(
            map("wasm32-wasip1", "wasm32", "wasi", "p1", "").unwrap(),
            "wasm32-wasi"
        );
        assert_eq!(
            map("thumbv7em-none-eabihf", "arm", "none", "", "eabihf").unwrap(),
            "thumb-freestanding-eabihf"
        );
        assert_eq!(
            map("bpfel-unknown-none", "bpf", "none", "", "").unwrap(),
            "bpfel-freestanding"
        );
        assert_eq!(
            map(
                "aarch64_be-unknown-linux-gnu",
                "aarch64",
                "linux",
                "gnu",
                "",
            )
            .unwrap(),
            "aarch64_be-linux-gnu"
        );
    }

    #[test]
    fn rejects_rust_targets_without_zig_abi_support() {
        for (target, arch, os, target_env, abi) in [
            (
                "aarch64-unknown-linux-gnu_ilp32",
                "aarch64",
                "linux",
                "gnu",
                "ilp32",
            ),
            (
                "powerpc-unknown-linux-gnuspe",
                "powerpc",
                "linux",
                "gnu",
                "spe",
            ),
            (
                "armv7-unknown-linux-uclibceabihf",
                "arm",
                "linux",
                "uclibc",
                "eabihf",
            ),
            ("arm64_32-apple-watchos", "aarch64", "watchos", "", ""),
        ] {
            assert!(map(target, arch, os, target_env, abi).is_err(), "{target}");
        }
    }
}
