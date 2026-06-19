# zig-rs

A Rust build dependency that wraps the `zig` CLI to build native Zig libraries from Cargo `build.rs` scripts.

## Project Structure

- `src/lib.rs` — the entire public API. Exposes `zig::build(path)` and `zig::Config` with `.define()` and `.optimize()`.
- `test-crate/` — self-contained example crate showing real usage: a `build.rs` calls `zig::build("libhello")` and links the resulting static library.
- `test-targets/` — no-link smoke crate whose build script calls `zig::build("smoke-zig")`; use it with `cargo check --target ...` to validate Rust→Zig target-string mappings without platform linkers.
- `test-wasm/` — wasm integration crate that builds a Zig static library, links it into a Rust `cdylib` for `wasm32-unknown-unknown`, and uses Node to call the exported Rust function that calls into Zig.
- `.github/workflows/CI.yml` — CI: builds test-crate on macOS/Windows/Ubuntu, cross-compiles via `cargo zigbuild` on Ubuntu, checks common target mappings with `test-targets/`, and runs the wasm integration path.
- `.devcontainer/` — VS Code dev container (Ubuntu + zig + rust-analyzer).

Generated/directories to ignore: `target/`, `.zig-cache/`, `zig-out/`.

## Build, Test, and Development Commands

```bash
# Build the root crate (no deps, fast)
cargo build

# Build and test via the test-crate (integrates zig build)
cd test-crate && cargo build && cargo test

# Check representative target mapping without linking
cargo check --manifest-path test-targets/Cargo.toml --target aarch64-apple-ios-sim

# Build and execute the wasm Rust→Zig integration path
cd test-wasm && cargo build --target wasm32-unknown-unknown && node run.js

# Cross-compile (requires `cargo-zigbuild` installed)
cargo zigbuild --target aarch64-unknown-linux-gnu
```

CI installs zig via `brew` (macOS/Linux) or `choco` (Windows), or `setup-zig@v2` for cross-compilation jobs.

## Coding Style

- Rust 2021 edition, standard clippy/rustfmt defaults.
- Public functions and types in `src/lib.rs` are documented with doc comments including `no_run` examples.
- The crate is intentionally minimal — no external dependencies beyond `std`.

## Testing

- Root unit tests cover target-string mapping helpers. `test-crate/src/main.rs` calls the C FFI exported by `test-crate/libhello/hello.zig`.
- Run with `cargo test` (debug and release) from `test-crate/`.
- Run `cargo check --manifest-path test-targets/Cargo.toml --target <triple>` for no-link target mapping smoke checks.
- Run `cd test-wasm && cargo build --target wasm32-unknown-unknown && node run.js` to verify the wasm artifact executes through Rust into Zig.
- CI validates native builds, selected cross-compilation targets, common target-smoke checks for Linux, macOS, iOS simulator, Android, Windows, and wasm, plus the executable wasm integration fixture.

## Commit & PR Guidelines

- CI must pass on all three platforms (macOS, Windows, Ubuntu) for native builds, and cross-compilation must pass for all listed targets.
- Use conventional commit messages.

## Agent Notes

- This crate is a **build dependency** — it runs during `cargo build` via `build.rs`, not at runtime.
- The `zig` binary must be on `PATH` for the `build()` function to work. CI handles this; locally agents may need to install it.
- Zig target strings are constructed from Cargo's `TARGET`, `CARGO_CFG_TARGET_ARCH`, `CARGO_CFG_TARGET_OS`, `CARGO_CFG_TARGET_ENV`, and `CARGO_CFG_TARGET_ABI` env vars. Keep mappings explicit and tested for Rust/Zig spelling differences such as macOS, Mac Catalyst, iOS simulators, Android, WASI/freestanding, Linux ABI suffixes, and architecture aliases.
- Optimize level maps Rust's `OPT_LEVEL` (0–3, s, z) to Zig's `Debug/Safe/Release{Safe,Fast,Small}`.
- The sample `test-crate` intentionally keeps Zig 0.15/0.16 build API compatibility and reparses macOS static archives with `/usr/bin/libtool`; Apple `ld` rejects some Zig 0.16 archives when Mach-O members are not 8-byte aligned.
