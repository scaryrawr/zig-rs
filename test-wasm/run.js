const { readFile } = require("node:fs/promises");
const { join } = require("node:path");

async function main() {
  const mode = process.argv[2] || "debug";
  const wasmPath = join(__dirname, "target", "wasm32-unknown-unknown", mode, "test_wasm.wasm");
  const bytes = await readFile(wasmPath);
  const { instance } = await WebAssembly.instantiate(bytes, {});
  const actual = instance.exports.rust_calls_zig(20, 22);
  const expected = 84;

  if (actual !== expected) {
    throw new Error(`expected rust_calls_zig(20, 22) to return ${expected}, got ${actual}`);
  }

  console.log(`rust_calls_zig(20, 22) = ${actual}`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
