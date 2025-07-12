[![Crates.io - boltpm](https://img.shields.io/crates/v/boltpm.svg)](https://crates.io/crates/boltpm)
[![Crates.io - plugin_api](https://img.shields.io/crates/v/plugin_api.svg)](https://crates.io/crates/plugin_api)

- [boltpm on crates.io](https://crates.io/crates/boltpm)
- [plugin_api on crates.io](https://crates.io/crates/plugin_api)

# BoltPM

BoltPM is a fast, modern, cross-platform NPM alternative written in Rust. It features a CLI, real-time GUI, private registry, and dynamic plugin system.

## Features
- CLI: install, remove, update, run, link, etc.
- GUI: Tauri desktop app with live logs, dependency tree, search, config, and dark mode
- Registry: self-hosted, token-auth, CORS, JSON index
- Plugins: dynamic hooks for install lifecycle

## Quick Start

### 1. Build All Components
```sh
cargo build --workspace
```

### 2. Run the Registry
```sh
cd registry
cargo run
# Registry runs at http://localhost:4000
```

### 3. Use the CLI
```sh
cd cli
cargo run -- init
cargo run -- install dep1
cargo run -- remove dep1
```

### 4. Run the GUI
```sh
cd gui
cargo tauri dev
```

## GUI Dark Mode
- Click the 🌙/☀️ button in the header to toggle dark mode.
- Your theme preference is remembered.
- ![Dark mode screenshot](docs/darkmode.png) <!-- Add screenshot here -->

## Usage Examples

- **Install a package:**
  ```sh
  boltpm install lodash
  ```
- **Remove a package:**
  ```sh
  boltpm remove lodash
  ```
- **Update all packages:**
  ```sh
  boltpm update
  ```
- **Run a script:**
  ```sh
  boltpm run build
  ```
- **Publish a package:**
  ```sh
  curl -X PUT -F "version=1.0.0" -F "description=My lib" -F "tarball=@package.tgz" http://localhost:4000/v1/mylib/
  ```
- **Use a plugin:**
  - Place your compiled `.so`/`.dylib`/`.dll` in `.boltpm/plugins/`.
  - Plugins are called automatically on install/remove/update.

## Troubleshooting & FAQ

- **Q: The GUI is blank or buttons don't work?**
  - A: Make sure you run `cargo tauri dev` in the `gui` directory and that the backend is built.
- **Q: Install fails with a network error?**
  - A: Ensure the registry is running at `http://localhost:4000`.
- **Q: Plugin not executing?**
  - A: Check that your plugin is in `.boltpm/plugins/` and built for your OS/arch.
- **Q: How do I clear the cache?**
  - A: Delete the `.boltpm/cache/` directory.

## Accessibility & UX Tips
- All navigation is keyboard accessible.
- Error messages are shown at the top of the GUI.
- Tooltips are available for all major actions.
- For best experience, keep your plugins stateless and your `package.json` valid.
- Dark mode is available for comfortable coding at night!

## Plugin System
- Plugins are dynamic libraries in `.boltpm/plugins/`.
- Hooks: `preinstall`, `postinstall`, `onError`.
- Example plugin logs the hook and package info.
- To develop a plugin:
  1. Implement the `run(ctx: PluginContext)` function.
  2. Build as `cdylib`.
  3. Place in `.boltpm/plugins/`.

## Error Handling & UX
- All errors print clear messages to the console and GUI.
- If a plugin fails, the `onError` hook is called.
- GUI shows logs, dependency tree, and config for troubleshooting.
- For best UX, keep plugins simple and stateless.

## Contributing
- PRs welcome! Run `cargo test --workspace` before submitting.
- See `dev.md` for architecture and roadmap.

## License
MIT
