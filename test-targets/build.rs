fn main() {
    println!("cargo:rerun-if-changed=smoke-zig/build.zig");

    let _ = zig::build("smoke-zig");
}
