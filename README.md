# ⚡️ BoltPM

A blazing fast, extensible, and workspace-friendly package manager for JavaScript/TypeScript monorepos.

---

## Features
- 🚀 **Fast, reproducible installs**
- 🧩 **Plugin system**: Native & WASM, per-package, lifecycle hooks
- 🏢 **Workspace support**: Multi-package monorepos, single lockfile
- 🔒 **Lockfile management**: Deterministic, reproducible builds
- 🎨 **Modern CLI/UX**: Color, emoji, summaries, beautiful help
- 🧪 **Integration tests**: Robust, automated CLI validation

---

## Install

```
cargo install boltpm
```
Or build locally:
```
git clone https://github.com/yourusername/BoltPM.git
cd BoltPM
cargo build --release -p boltpm
```

---

## Example Usage

```sh
boltpm install
boltpm plugin list
boltpm --help
```

---

## Writing a Plugin (Rust Example)

```rust
use plugin_api::PluginContext;
use std::fs;
use std::path::PathBuf;

#[no_mangle]
pub extern "C" fn run(ctx: PluginContext) -> i32 {
    let output_file = PathBuf::from(&ctx.output_path).join("PLUGIN_TEST");
    fs::create_dir_all(&ctx.output_path).unwrap();
    fs::write(&output_file, "Plugin executed!").unwrap();
    0
}
```

---

## Plugin Lifecycle Hooks
- `preinstall`: Before install
- `postinstall`: After install
- `onError`: On error

Plugins can register for specific hooks by filename (e.g. `my_plugin__postinstall.dylib`).

---

## WASM Plugin Support ⚠️
- WASM plugins are **experimental**. Use with caution.
- WASM plugins receive `PLUGIN_CONTEXT` as a JSON env var.

---

## License
MIT
