#!/bin/bash
set -euo pipefail

# 1. Find all Cargo.toml files
CARGO_TOMLS=$(find . -name Cargo.toml)

# 2. Check plugin_api dependency in each Cargo.toml
echo "Checking plugin_api path dependencies in all Cargo.toml files..."
MISSING_PATH=0
for toml in $CARGO_TOMLS; do
    if grep -q '\[dependencies\]' "$toml"; then
        if grep -A 5 '\[dependencies\]' "$toml" | grep -q 'plugin_api'; then
            if ! grep -A 5 '\[dependencies\]' "$toml" | grep 'plugin_api' | grep -q 'path'; then
                echo "[WARN] $toml: plugin_api dependency does NOT use a path!"
                MISSING_PATH=1
            else
                echo "[OK]   $toml: plugin_api uses path dependency."
            fi
        fi
    fi
done
if [[ $MISSING_PATH -eq 0 ]]; then
    echo "All plugin_api dependencies use a path."
else
    echo "Some plugin_api dependencies do NOT use a path. Fix these!"
fi

# 3. Print rustc version
echo
RUSTC_VERSION=$(rustc --version)
echo "rustc version: $RUSTC_VERSION"

# 4. Check architecture of CLI binary and plugin dylibs
CLI_BIN=$(find ./target/debug -maxdepth 1 -type f -perm +111 -name 'boltpm' | head -n1)
PLUGIN_DYLIBS=$(find . -name 'lib*.dylib')

echo
if [[ -n "$CLI_BIN" ]]; then
    echo "CLI binary architecture:"
    file "$CLI_BIN"
else
    echo "[WARN] Could not find CLI binary (boltpm) in ./target/debug."
fi

echo
if [[ -n "$PLUGIN_DYLIBS" ]]; then
    echo "Plugin dylib architectures:"
    for dylib in $PLUGIN_DYLIBS; do
        file "$dylib"
    done
else
    echo "[WARN] No plugin dylibs found."
fi

echo
# 5. Summary
if [[ $MISSING_PATH -eq 0 ]]; then
    echo "[SUMMARY] All plugin_api dependencies use a path."
else
    echo "[SUMMARY] Some plugin_api dependencies do NOT use a path. Fix these!"
fi

echo "[SUMMARY] rustc version: $RUSTC_VERSION" 