#!/bin/bash
set -euo pipefail

# Find all plugin_api artifacts except .d files
artifacts=( $(find target/debug/deps -type f -name '*plugin_api*' ! -name '*.d') )
count=${#artifacts[@]}

if [ "$count" -eq 1 ]; then
  echo "✅ Single plugin_api build detected: ${artifacts[0]}"
  exit 0
else
  echo "❌ Multiple plugin_api builds detected! This will break FFI boundaries."
  for f in "${artifacts[@]}"; do
    echo "  $f"
  done
  exit 1
fi 