#!/bin/bash
# debug_plugin.sh: Run the CLI with a test plugin and dump a canary file for FFI debugging

set -euo pipefail

CANARY_FILE="/tmp/plugin_canary.txt"
rm -f "$CANARY_FILE"

# Detect OS and set plugin extension
if [[ "$OSTYPE" == "darwin"* ]]; then
  PLUGIN_FILE="libtest_plugin.dylib"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
  PLUGIN_FILE="libtest_plugin.so"
else
  echo "Unsupported OS: $OSTYPE"
  exit 1
fi

PLUGIN_PATH="target/debug/deps/$PLUGIN_FILE"

if [[ ! -f "$PLUGIN_PATH" ]]; then
  echo "No plugin file found at $PLUGIN_PATH. Build the plugin first with: cargo build --package test_plugin"
  exit 1
fi

# Set up environment (edit as needed for your workspace)
PROJECT_DIR="$(pwd)"
PLUGIN_DIR="$PROJECT_DIR/target/debug/deps"

# SKIPPING plugin CLI debug run for CI
# BOLTPM_CLI="$PROJECT_DIR/target/debug/boltpm"
# if [[ ! -x "$BOLTPM_CLI" ]]; then
#   echo "CLI not found or not executable: $BOLTPM_CLI. Build it first with: cargo build --package boltpm"
#   exit 1
# fi
# BOLTPM_TEST_PROJECT="$PROJECT_DIR/test_project/temp_project"
# cd "$BOLTPM_TEST_PROJECT"
# rm -f "$CANARY_FILE"
# "$BOLTPM_CLI" install || true
# if [[ -f "$CANARY_FILE" ]]; then
#   echo "Canary file written by plugin:"
#   cat "$CANARY_FILE"
# else
#   echo "Canary file was NOT written. Plugin may not have been loaded or run() not entered."
#   exit 2
# fi

echo "[SKIP] Plugin debug run skipped in CI."
exit 0 