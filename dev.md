# BoltPM Development Guide

## What's Left To Do (Checklist)

- [ ] **Finalize WASM plugin support**
    - [ ] Ensure WASM plugins receive correct PluginContext and output path
    - [ ] Add integration tests for WASM plugins (success/failure)
    - [ ] Document WASM plugin authoring and usage
- [ ] **Advanced lifecycle hooks**
    - [ ] Allow plugins to register for custom hooks (preuninstall, onError, etc.)
    - [ ] Document all available hooks and plugin subscription
- [ ] **Plugin marketplace (optional/future)**
    - [ ] Implement remote plugin registry/marketplace
    - [ ] Add CLI commands for searching/installing remote plugins
- [ ] **GUI enhancements**
    - [ ] Add more features to Tauri GUI (auto-update, audit logs, etc.)
    - [ ] Improve real-time feedback and error reporting
- [ ] **Error handling & reporting**
    - [ ] Summarize all plugin errors at end of install/update
    - [ ] Add more granular exit codes for CI/automation
    - [ ] (Optional) Add error telemetry
- [ ] **Test coverage & CI**
    - [ ] Expand integration tests for edge cases (plugin failures, workspace edge cases)
    - [ ] Add tests for new plugin types and hooks
    - [ ] Ensure all new features are covered by CI
- [ ] **Documentation & DX**
    - [ ] Expand docs for plugin authors (native & WASM)
    - [ ] Add usage examples and troubleshooting tips
    - [ ] Polish website/docs for onboarding/marketing
- [ ] **Performance & scalability (optional)**
    - [ ] Profile install and plugin execution for large monorepos
    - [ ] Optimize plugin loading and workspace traversal if needed

---

## Features

### Core Package Management
- Install, update, remove, and search for packages (npm-style commands)
- Workspace support: multi-package monorepos with a single lockfile
- Lockfile management: generates and updates `bolt.lock` for reproducible installs
- Dependency resolution for all workspace packages

### Plugin System
- Per-package plugin discovery: `.boltpm/plugins/` in each package, fallback to workspace root
- Native (dylib) plugin support (Rust)
- WASM plugin support (if enabled)
- Plugin context isolation: each plugin gets `output_path`, `install_path`, and env vars
- Per-package plugin output: `.boltpm/plugins_output/<package>/`
- Lifecycle hooks: `preinstall`, `postinstall`, etc.
- Easy plugin authoring (Rust or WASM)

### CLI & UX
- Modern, colored CLI output (banner, emoji, color-coded statuses)
- Beautiful, user-friendly help menu
- Concise plugin execution summary after install
- Minimal/no debug noise in production output

### Testing & Reliability
- Automated integration tests for per-package plugin execution and output
- Error handling: CLI continues on plugin errors, summarizes failures, uses proper exit codes

### Extensibility
- Plugins can subscribe to specific lifecycle hooks
- Pluggable architecture for future plugin types/hooks

---

## Development Workflow

1. **Build the CLI:**
   ```sh
   cargo build -p boltpm
   ```
2. **Run the CLI:**
   ```sh
   ./target/debug/boltpm install
   # or from a workspace:
   cd test_project && ../target/debug/boltpm install
   ```
3. **Develop Plugins:**
   - Write plugins in Rust (see `plugins/sample_plugin/`).
   - Build with `cargo build -p sample_plugin`.
   - Copy `.dylib` to the target package's `.boltpm/plugins/` directory.
   - Plugins receive a `PluginContext` with all relevant paths and env.
   - Output files should be written to `ctx.output_path` for isolation.

4. **Integration Testing:**
   - See `tests/integration/per_package_plugin.rs` for examples.
   - Use `assert_cmd`, `tempfile`, and `predicates` crates.
   - Fixtures are under `tests/fixtures/`.

5. **Debugging:**
   - Use the `--verbose` flag for extra output (if enabled).
   - For workspace issues, ensure you run from the correct directory with the right `bolt.lock`.

6. **Contributing:**
   - Keep CLI output clean (no debug prints in production).
   - Document new features in this file and in the help menu.
   - Add tests for new plugin or workspace features.

---

## Gotchas & Tips
- Workspace globs are relative to the project root. Always run from the correct directory.
- Plugins must write output to `ctx.output_path` for per-package isolation.
- If a plugin fails, the CLI will summarize errors but continue (unless critical).
- For WASM plugins, ensure wasmtime is enabled and the plugin exposes a `_run` function.

---

For more, see the docs/ folder and the CLI help menu.