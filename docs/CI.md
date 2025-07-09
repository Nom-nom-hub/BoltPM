# CI/CD Pipeline Documentation

## Overview

Our CI pipeline ensures the BoltPM plugin system remains stable and production-ready by running comprehensive checks on every push and pull request.

## Workflow Steps

### 1. 🔍 Plugin API Duplication Check
**Script:** `ci/check_plugin_api.sh`
**Purpose:** Prevents FFI boundary issues by detecting duplicate `plugin_api` artifacts
**What it does:**
- Scans `target/debug/deps/` for multiple `plugin_api` libraries
- Fails the build if duplicates are found (indicates linkage issues)
- Ensures all crates use the same `plugin_api` build

**Debugging:** If this fails, run `./scripts/rebuild_all.sh` locally to clean and rebuild

### 2. 🔨 Build Workspace
**Command:** `cargo build --workspace --verbose`
**Purpose:** Ensures all crates compile successfully
**What it does:**
- Builds CLI, plugin API, plugins, and all dependencies
- Uses verbose output for detailed error reporting
- Validates workspace-wide compilation

### 3. 🧪 Run Tests
**Command:** `cargo test --workspace --verbose`
**Purpose:** Validates functionality across all components
**What it does:**
- Runs unit tests for all crates
- Executes integration tests
- Runs E2E tests in `test_project/`
- Ensures plugin system works end-to-end

### 4. 🔌 Plugin FFI Verification
**Script:** `scripts/debug_plugin.sh`
**Purpose:** Tests FFI boundaries and plugin loading
**What it does:**
- Builds test plugin with canary file functionality
- Runs CLI with test plugin
- Verifies canary file creation at `/tmp/plugin_canary.txt`
- Confirms FFI entry and context passing

### 5. 🔍 Code Quality Checks
**Clippy:** `cargo clippy --workspace -- -D warnings`
**Formatting:** `cargo fmt --all -- --check`
**Purpose:** Maintains code quality and consistency

## Debugging Failures

### Plugin API Duplication Failures
```bash
# Local fix
./scripts/rebuild_all.sh
```

### FFI Verification Failures
1. **Check canary file:**
   ```bash
   cat /tmp/plugin_canary.txt
   ```
2. **Expected content:**
   ```
   run() entered at 2024-01-15T10:30:00Z
   ctx_len: 123
   ```
3. **If missing:** Plugin `run()` function not being called

### Test Failures
1. **Check artifacts:** Download `test-results` from GitHub Actions
2. **Look for canary files:** Verify plugin execution
3. **Check logs:** Look for `[PLUGIN LOADER DEBUG]` and `[SamplePlugin]` output

### Build Failures
1. **Check verbose output:** Look for specific compilation errors
2. **Verify dependencies:** Ensure all crates are compatible
3. **Check workspace:** Ensure `Cargo.toml` workspace configuration is correct

## Local Development

### Quick Checks
```bash
# Run CI checks locally
./ci/check_plugin_api.sh
cargo build --workspace
cargo test --workspace
./scripts/debug_plugin.sh
```

### Clean Rebuild
```bash
# When experiencing strange issues
./scripts/rebuild_all.sh
```

### Plugin Development
```bash
# Test plugin FFI boundaries
./scripts/debug_plugin.sh

# Check canary file
cat /tmp/plugin_canary.txt
```

## Artifacts

The CI pipeline uploads these artifacts for debugging:
- `target/debug/` - Build outputs
- `/tmp/plugin_canary.txt` - FFI verification file

Download these from the GitHub Actions run page to investigate failures.

## Common Issues

### "Plugin API duplication detected"
- **Cause:** Multiple `plugin_api` libraries in build artifacts
- **Fix:** Run `./scripts/rebuild_all.sh`

### "Canary file was NOT written"
- **Cause:** Plugin `run()` function not being called
- **Debug:** Check plugin loading logs and FFI boundaries

### "Plugins directory does not exist"
- **Cause:** Missing `.boltpm/plugins/` directory
- **Fix:** Ensure plugin discovery path is correct

## Best Practices

1. **Always run local checks** before pushing
2. **Use canary files** for FFI debugging
3. **Check artifacts** when CI fails
4. **Keep plugin_api minimal** and ABI-stable
5. **Use rebuild script** when experiencing strange issues 