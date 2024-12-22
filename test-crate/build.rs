use zig;

fn main() {
    let mut config = zig::Config::new("libhello");
    let dst = config.build();
    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=hello");
}
