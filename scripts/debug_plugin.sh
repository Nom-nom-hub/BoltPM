#!/bin/bash
# debug_plugin.sh: Run the CLI with a test plugin and dump a canary file for FFI debugging

set -euo pipefail

CANARY_FILE="/tmp/plugin_canary.txt"
rm -f "$CANARY_FILE"

# Set up environment (edit as needed for your workspace)
PROJECT_DIR="$(pwd)"
PLUGIN_DIR="$PROJECT_DIR/target/debug/deps"
PLUGIN_DYLIB="$PLUGIN_DIR/libtest_plugin.dylib"

if [[ ! -f "$PLUGIN_DYLIB" ]]; then
  echo "No plugin dylib found at $PLUGIN_DYLIB. Build the plugin first with: cargo build --package test_plugin"
  exit 1
fi

# Run the CLI with the test plugin, passing a dummy context
# (You may need to adjust the CLI invocation and arguments)
BOLTPM_CLI="$PROJECT_DIR/target/debug/boltpm"
if [[ ! -x "$BOLTPM_CLI" ]]; then
  echo "CLI not found or not executable: $BOLTPM_CLI. Build it first with: cargo build --package boltpm"
  exit 1
fi

# Example: run the CLI with a test command that triggers plugin loading
# (Replace 'install' and args as needed for your CLI)
BOLTPM_TEST_PROJECT="$PROJECT_DIR/test_project/temp_project"
cd "$BOLTPM_TEST_PROJECT"

# Remove old canary file if present
rm -f "$CANARY_FILE"

# Run the CLI (edit command as needed)
"$BOLTPM_CLI" install || true

if [[ -f "$CANARY_FILE" ]]; then
  echo "Canary file written by plugin:"
  cat "$CANARY_FILE"
else
  echo "Canary file was NOT written. Plugin may not have been loaded or run() not entered."
  exit 2
fi 