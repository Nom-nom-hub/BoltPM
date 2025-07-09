#!/bin/bash
set -euo pipefail

echo "🧹 Deleting all .dylib files..."
find . -name '*.dylib' -delete

echo "🧹 Removing all target directories..."
rm -rf target test_project/temp_project/target

echo "🔨 Rebuilding the entire workspace..."
cargo build --workspace

if [ $? -eq 0 ]; then
  echo "✅ Rebuild complete."
else
  echo "❌ Rebuild failed."
  exit 1
fi 