# zig-rs

A build dependency for building [`zig`](https://ziglang.org/) libraries.

## Example

Please look at [test-crate](test-crate) for a full example.

```rust
// build.rs

fn main() {
    println!("cargo:rerun-if-changed=libhello/build.zig");
    println!("cargo:rerun-if-changed=libsl/hello.zig");

    let dst = zig::build("libhello");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=hello");
}
```

## Acknowledgements

- Based on [cmake-rs](https://github.com/rust-lang/cmake-rs) for zig, but way more minimal at the moment (I don't know zig...).
- Skimmed [zig build system](https://ziglang.org/learn/build-system/) for how to build a library.
