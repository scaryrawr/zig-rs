fn main() {
    println!("cargo:rerun-if-changed=libhello/build.zig");
    println!("cargo:rerun-if-changed=libhello/hello.zig");

    let dst = zig::build("libhello");
    fix_macos_static_archive(&dst.join("libhello.a"));

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=hello");
}

#[cfg(target_os = "macos")]
fn fix_macos_static_archive(library: &std::path::Path) {
    use std::{fs, os::unix::fs::PermissionsExt, process::Command};

    // Apple ld requires 64-bit Mach-O archive members to be 8-byte aligned.
    // Repacking with libtool keeps the sample crate linkable with Zig 0.16.
    let work_dir = library
        .parent()
        .unwrap()
        .join(format!(".{}-rearchive", std::process::id()));
    let fixed_library = work_dir.join("libhello.a");

    if work_dir.exists() {
        fs::remove_dir_all(&work_dir).unwrap();
    }
    fs::create_dir(&work_dir).unwrap();

    let status = Command::new("ar")
        .arg("-x")
        .arg(library)
        .current_dir(&work_dir)
        .status()
        .unwrap();
    assert!(status.success(), "failed to extract {}", library.display());

    let mut objects = Vec::new();
    for entry in fs::read_dir(&work_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension() == Some(std::ffi::OsStr::new("o")) {
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(permissions.mode() | 0o600);
            fs::set_permissions(&path, permissions).unwrap();
            objects.push(path);
        }
    }
    objects.sort();
    assert!(
        !objects.is_empty(),
        "no object files found in {}",
        library.display()
    );

    let status = Command::new("/usr/bin/libtool")
        .arg("-static")
        .arg("-o")
        .arg(&fixed_library)
        .args(&objects)
        .status()
        .unwrap();
    assert!(status.success(), "failed to rearchive {}", library.display());

    fs::rename(&fixed_library, library).unwrap();
    fs::remove_dir_all(&work_dir).unwrap();
}

#[cfg(not(target_os = "macos"))]
fn fix_macos_static_archive(_: &std::path::Path) {}
