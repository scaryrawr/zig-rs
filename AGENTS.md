# zig-rs

A Rust build dependency that wraps the `zig` CLI to build native Zig libraries from Cargo `build.rs` scripts.

## Project Structure

- `src/lib.rs` — the entire public API. Exposes `zig::build(path)` and `zig::Config` with `.define()` and `.optimize()`.
- `test-crate/` — self-contained example crate showing real usage: a `build.rs` calls `zig::build("libhello")` and links the resulting static library.
- `.github/workflows/CI.yml` — CI: builds test-crate on macOS/Windows/Ubuntu (zig via brew/choco), and cross-compiles via `cargo zigbuild` on Ubuntu.
- `.devcontainer/` — VS Code dev container (Ubuntu + zig + rust-analyzer).

Generated/directories to ignore: `target/`, `.zig-cache/`, `zig-out/`.

## Build, Test, and Development Commands

```bash
# Build the root crate (no deps, fast)
cargo build

# Build and test via the test-crate (integrates zig build)
cd test-crate && cargo build && cargo test

# Cross-compile (requires `cargo-zigbuild` installed)
cargo zigbuild --target aarch64-unknown-linux-gnu
```

CI installs zig via `brew` (macOS/Linux) or `choco` (Windows), or `setup-zig@v2` for cross-compilation jobs.

## Coding Style

- Rust 2021 edition, standard clippy/rustfmt defaults.
- Public functions and types in `src/lib.rs` are documented with doc comments including `no_run` examples.
- The crate is intentionally minimal — no external dependencies beyond `std`.

## Testing

- The only test suite lives in `test-crate/src/main.rs` and calls the C FFI exported by `test-crate/libhello/hello.zig`.
- Run with `cargo test` (debug and release) from `test-crate/`.
- CI validates both native and cross-compilation targets.

## Commit & PR Guidelines

- CI must pass on all three platforms (macOS, Windows, Ubuntu) for native builds, and cross-compilation must pass for all listed targets.
- Use conventional commit messages.

## Agent Notes

- This crate is a **build dependency** — it runs during `cargo build` via `build.rs`, not at runtime.
- The `zig` binary must be on `PATH` for the `build()` function to work. CI handles this; locally agents may need to install it.
- Zig target strings are constructed from `CARGO_CFG_TARGET_ARCH`, `CARGO_CFG_TARGET_OS`, and `TARGET` env vars. The OS mapping special-cases `"unknown"` → `"freestanding"` for wasm32.
- Optimize level maps Rust's `OPT_LEVEL` (0–3, s, z) to Zig's `Debug/Safe/Release{Safe,Fast,Small}`.
