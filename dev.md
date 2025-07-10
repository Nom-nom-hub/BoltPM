# BoltPM Development Guide

## ✅ Current State

- **README.md**: Complete, clear, and user-friendly
- **Cargo.toml**: Version, description, license, repository, workspace members all set
- **CLI**: Modern, colored, beautiful help and version output; user-friendly errors
- **Plugin System**: Per-package, native/WASM, lifecycle hooks, output isolation
- **CI/CD**: GitHub Actions runs `cargo test` and E2E CLI checks
- **Examples**: Minimal Rust plugin in `examples/sample_plugin/`
- **Integration Tests**: Automated, robust CLI validation

---

## 🚀 Release Checklist

- [ ] Publish to crates.io (`cargo publish --dry-run` must succeed)
- [ ] Tag GitHub release (v0.1.0)
- [ ] Upload prebuilt plugin binaries (.wasm, .dylib, .so) to releases
- [ ] Add more plugin examples (WASM/native)
- [ ] Expand documentation (advanced plugin authoring, lifecycle, troubleshooting)

---

## 🧠 Optional Nice-to-Haves (v0.2+)

- [ ] Plugin manifest (`plugin.toml`) for metadata
- [ ] Plugin result reporting (status, log, warnings)
- [ ] Uninstall command
- [ ] Plugin registry/marketplace
- [ ] GUI (Tauri or web)
- [ ] Performance/scalability improvements

---

## 🛠 Contributor Guide

- **Build CLI:**
  ```sh
  cargo build -p boltpm
  ```
- **Run CLI:**
  ```sh
  ./target/debug/boltpm --help
  ./target/debug/boltpm install
  ```
- **Test:**
  ```sh
  cargo test --all
  ```
- **Develop Plugins:**
  - See `examples/sample_plugin/` for a minimal template
  - Place built plugins in `.boltpm/plugins/` or per-package `.boltpm/plugins/`
- **CI:**
  - All PRs must pass GitHub Actions (build, test, E2E CLI)

---

## ⚠️ Gotchas & Tips

- Workspace globs are relative to project root; run CLI from the correct directory
- WASM plugin support is experimental
- For best UX, keep plugins stateless and simple
- See README.md for full feature list and usage