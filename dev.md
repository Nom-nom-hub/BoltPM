# BoltPM Development Plan (Status: ✅ = Done, 🟧 = In Progress, 🕒 = Future)

## What's Done
- Deterministic, CWD-agnostic plugin loading and output paths (always workspace-root relative)
- Robust plugin lifecycle integration tests (success and failure cases)
- Serial test execution using `serial_test` to avoid race conditions
- Enhanced debug logging for plugin loading, exit codes, and errors
- All major CLI, registry, GUI, and plugin system features implemented and tested
- Unified, public, and serializable `PluginContext` for consistent plugin communication

## What's Left To Do
- WASM plugin support 🟧
- Remote plugin marketplace 🕒
- GUI auto-update packages 🕒
- Package audit logs & trust levels 🕒
- Further test isolation (unique plugin dirs per test) if needed

---

## Overview
BoltPM is a fast, modern, cross-platform NPM alternative written in Rust. It consists of a production-quality CLI, a real-time GUI frontend, a self-hosted private registry, and a dynamic plugin system for install hooks.

---

## 1. Project Structure

boltpm/
├── cli/ # CLI tool (boltpm)
├── gui/ # Desktop GUI (boltpm-gui)
├── registry/ # Self-hosted registry (boltpm-registry)
├── plugins/ # Plugin loader + sample plugin
│ └── sample_plugin/
├── plugin_api/ # Shared plugin API crate
├── Cargo.toml # Workspace root
└── README.md


---

## 2. Tech Stack

- **Language:** Rust (2021+) ✅
- **CLI:** `clap`, `serde`, `semver`, `reqwest`, `flate2`, `tar`, `zip`, `rayon`/`tokio` ✅
- **GUI:** `tauri` (webview, recommended) ✅
- **Registry:** `axum` ✅
- **Plugin System:**  
  - Dynamic library plugins: `.so`, `.dylib`, `.dll` ✅  
  - Loaded via `libloading` with workspace-root relative paths ✅  
  - Unified, public, and serde-serializable `PluginContext` ✅  
  - Robust plugin lifecycle tests with serial execution ✅  
  - Sample plugin updated for new API ✅  
  - WASM plugin support integration 🟧  
- **Testing:** Rust built-in test framework, mock backends for GUI 🚧

---

## 3. Component Breakdown

### 3.1 CLI (boltpm)
- [x] Subcommands: `install`, `remove`, `update`, `run`, `init`, `link`, etc.
- [x] Parse `package.json` with `serde`
- [x] Dependency resolution with `semver`
- [x] Download/extract packages with `reqwest`, `flate2`, `tar`, `zip`
- [x] Parallelism with `tokio` (ready for rayon)
- [x] Deterministic installs via `bolt.lock`
- [x] Cache in `.boltpm/`
- [x] Plugin hooks: preinstall, postinstall, onError
- [x] Migration scripts for npm, Yarn, pnpm

### 3.2 GUI (boltpm-gui)
- [x] Built with `tauri` (webview)
- [x] Live install logs
- [x] Dependency tree visualization
- [x] Search/install/uninstall packages
- [x] View/edit `package.json`
- [x] Show cache size
- [x] Config tab
- [x] Communicate with CLI/core via IPC
- [x] Dark mode, accessibility, error handling

### 3.3 Registry (boltpm-registry)
- [x] Built with `axum`
- [x] Endpoints: `PUT` for publishing `.tgz`, `GET` for metadata and tarballs, JSON index
- [x] Token-based authentication (stub, ready for prod)
- [x] Store metadata in `packages.json`
- [x] Store tarballs in `./packages/<name>/<version>/`
- [x] CORS enabled for GUI

### 3.4 Plugin System
- [x] Dynamic library plugins: `.so`, `.dylib`, `.dll`
- [x] Loaded via `libloading` with workspace-root relative paths
- [x] Unified, public, and serde-serializable `PluginContext`
- [x] Plugin lifecycle tests with serial execution to avoid race conditions
- [x] Robust loader and plugin lifecycle integration tests (success/failure)
- [x] Sample plugin updated for new PluginContext fields
- [🟧] WASM plugin support (integration test infrastructure working; final integration ongoing)
- [ ] Remote marketplace 🕒

---

## 4. Milestones

- [x] **Workspace & Scaffolding**
  - [x] Set up Rust workspace and all crates
  - [x] Add minimal code to each crate
- [x] **CLI Core**
  - [x] Implement `init`, `install`, `remove`, `update`, `run`, `link`
  - [x] Lockfile and cache management
  - [x] Plugin hook integration
- [x] **Registry**
  - [x] Implement endpoints for publish, fetch, metadata
  - [x] Local storage and authentication
- [x] **GUI**
  - [x] Tauri app with live logs, dependency tree, search, config
  - [x] IPC with CLI/core
- [x] **Plugin System**
  - [x] Plugin loader and API
  - [x] Sample plugin
- [x] **Testing & CI**
  - [x] Unit tests for CLI and registry
  - [x] GUI tests with mock backend (basic)
  - [x] Plugin loader tests
- [🟧] **WASM plugin support**
  - [🟧] Integration test harness and WASM plugin builds complete
  - [🟧] CLI integration and lifecycle behavior ongoing
- [x] **Docs & Polish**
  - [x] README, dev docs, usage examples
  - [x] Branding, UX polish

---

## 5. Testing Strategy

- [x] **CLI:** Unit and integration tests for all commands
- [x] **Registry:** Endpoint tests, authentication, error handling
- [x] **GUI:** Component tests, mock backend for registry/CLI
- [x] **Plugins:** Loader tests, sample plugin integration
- [x] **CI:** Enforce test coverage, linting, build checks
- [🟧] **WASM:** Lifecycle integration tests (success and failure cases)

---

## 6. Future Plans (Scaffold Only)
- [🟧] WASM plugin support
- [ ] Remote plugin marketplace
- [ ] GUI auto-update packages
- [ ] Package audit logs & trust levels

---

## 7. Contributing
- [x] Fork, branch, and PR workflow
- [x] Code style: rustfmt, clippy
- [x] All contributions must include tests

---

## 8. References
- [Rust Book](https://doc.rust-lang.org/book/)
- [Tauri Docs](https://tauri.app/)
- [Axum Docs](https://docs.rs/axum/)
- [Serde Docs](https://serde.rs/)
- [Clap Docs](https://docs.rs/clap/) 

## Lockfile (`bolt.lock`)

BoltPM uses a JSON lockfile (`bolt.lock`) to ensure deterministic installs. The lockfile records the exact version, resolved tarball URL, and dependency tree for every installed package. This guarantees that repeated installs produce the same dependency graph, even if newer versions are published.

Example format:
```json
{
  "packages": {
    "foo": {
      "version": "1.2.3",
      "resolved": "http://localhost:4000/v1/foo/1.2.3/",
      "dependencies": {
        "bar": "2.0.0"
      }
    },
    "bar": {
      "version": "2.0.0",
      "resolved": "http://localhost:4000/v1/bar/2.0.0/",
      "dependencies": {}
    }
  }
}
On install, BoltPM will use bolt.lock to pin versions if present.
On update, the lockfile is refreshed.
On remove, entries are deleted from the lockfile.
If bolt.lock and package.json are out of sync, BoltPM will warn or error (with --frozen-lockfile).