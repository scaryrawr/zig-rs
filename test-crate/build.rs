fn main() {
    println!("cargo:rerun-if-changed=libhello/build.zig");
    println!("cargo:rerun-if-changed=libsl/hello.zig");

    let dst = zig::build("libhello");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=hello");
}
