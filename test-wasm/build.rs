fn main() {
    println!("cargo:rerun-if-changed=zig-math/build.zig");
    println!("cargo:rerun-if-changed=zig-math/math.zig");

    let dst = zig::build("zig-math");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=zig_math");
}
