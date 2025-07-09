# BoltPM Development Plan

## Overview
BoltPM is a fast, modern, cross-platform NPM alternative written in Rust. It consists of a production-quality CLI, a real-time GUI frontend, a self-hosted private registry, and a dynamic plugin system for install hooks.

---

## 1. Project Structure

```
boltpm/
├── cli/         # CLI tool (boltpm)
├── gui/         # Desktop GUI (boltpm-gui)
├── registry/    # Self-hosted registry (boltpm-registry)
├── plugins/     # Plugin loader + sample plugin
│   └── sample_plugin/
├── plugin_api/  # Shared plugin API crate
├── Cargo.toml   # Workspace root
└── README.md
```

---

## 2. Tech Stack

- **Language:** Rust (2021+)
- **CLI:** `clap`, `serde`, `semver`, `reqwest`, `flate2`, `tar`, `zip`, `rayon`/`tokio`
- **GUI:** `tauri` (webview, recommended) or `iced` (native)
- **Registry:** `axum` (recommended) or `actix-web`
- **Plugin System:** `libloading`, shared `plugin_api` crate
- **Testing:** Rust built-in test framework, mock backends for GUI

---

## 3. Component Breakdown

### 3.1 CLI (boltpm)
- Subcommands: `install`, `remove`, `update`, `run`, `init`, `link`, etc.
- Parse `package.json` with `serde`
- Dependency resolution with `semver`
- Download/extract packages with `reqwest`, `flate2`, `tar`, `zip`
- Parallelism with `rayon` or `tokio`
- Deterministic installs via `bolt.lock`
- Cache in `.boltpm/`
- Plugin hooks: preinstall, postinstall, onError

### 3.2 GUI (boltpm-gui)
- Built with `tauri` (webview) or `iced` (native)
- Features:
  - Live install logs
  - Dependency tree visualization
  - Search/install/uninstall packages
  - View/edit `package.json`
  - Show cache size
  - Config tab
- Communicate with CLI/core via IPC or shared async functions

### 3.3 Registry (boltpm-registry)
- Built with `axum` or `actix-web`
- Endpoints:
  - `PUT` for publishing `.tgz`
  - `GET` for metadata and tarballs
  - JSON index
  - Token-based authentication
- Store metadata in `packages.json`
- Store tarballs in `./packages/<name>/<version>/`
- CORS enabled for GUI

### 3.4 Plugin System
- Dynamic library plugins: `.so`, `.dylib`, `.dll`
- Loaded via `libloading`
- Plugin API:
  ```rust
  pub struct PluginContext {
      pub package_name: String,
      pub version: String,
      pub path: PathBuf,
      pub metadata: HashMap<String, String>,
  }
  // Plugin must export:
  fn run(ctx: PluginContext) -> Result<()>
  ```
- Plugin registry: `.boltpm/plugins/`
- Auto-discover plugins (local/global)
- Future: WASM plugin support, remote marketplace

---

## 4. Milestones

1. **Workspace & Scaffolding**
   - Set up Rust workspace and all crates
   - Add minimal code to each crate
2. **CLI Core**
   - Implement `init`, `install`, `remove`, `update`, `run`, `link`
   - Lockfile and cache management
   - Plugin hook integration
3. **Registry**
   - Implement endpoints for publish, fetch, metadata
   - Local storage and authentication
4. **GUI**
   - Tauri/iced app with live logs, dependency tree, search, config
   - IPC with CLI/core
5. **Plugin System**
   - Plugin loader and API
   - Sample plugin
6. **Testing & CI**
   - Unit tests for CLI and registry
   - GUI tests with mock backend
   - Plugin loader tests
7. **Docs & Polish**
   - README, dev docs, usage examples
   - Branding, UX polish

---

## 5. Testing Strategy

- **CLI:** Unit and integration tests for all commands, 100% coverage
- **Registry:** Endpoint tests, authentication, error handling
- **GUI:** Component tests, mock backend for registry/CLI
- **Plugins:** Loader tests, sample plugin integration
- **CI:** Enforce test coverage, linting, build checks

---

## 6. Future Plans (Scaffold Only)
- WASM plugin support
- Remote plugin marketplace
- GUI auto-update packages
- Package audit logs & trust levels

---

## 7. Contributing
- Fork, branch, and PR workflow
- Code style: rustfmt, clippy
- All contributions must include tests

---

## 8. References
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tauri Docs](https://tauri.app/)
- [Axum Docs](https://docs.rs/axum/)
- [Serde Docs](https://serde.rs/)
- [Clap Docs](https://docs.rs/clap/) 