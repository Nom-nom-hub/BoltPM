#!/bin/bash
set -euo pipefail

# Find all plugin_api artifacts except .d files
artifacts=( $(find target/debug/deps -type f -name '*plugin_api*' ! -name '*.d') )
count=${#artifacts[@]}

# Extract hashes from filenames (the long hex string after the last dash and before the extension)
hashes=()
for f in "${artifacts[@]}"; do
  # Extract hash using regex: match -<hash> before . or end
  if [[ "$f" =~ -([a-f0-9]{16,}) ]]; then
    hashes+=("${BASH_REMATCH[1]}")
  fi
# fallback: if no match, use the whole filename
# (shouldn't happen, but prevents empty array)

done

# Get unique hashes
unique_hashes=( $(printf "%s\n" "${hashes[@]}" | sort -u) )

if [ "${#unique_hashes[@]}" -eq 1 ]; then
  echo "✅ Single plugin_api build detected (hash: ${unique_hashes[0]}):"
  for f in "${artifacts[@]}"; do
    echo "  $f"
  done
  exit 0
else
  echo "❌ Multiple plugin_api builds detected! This will break FFI boundaries. Hashes: ${unique_hashes[*]}"
  for f in "${artifacts[@]}"; do
    echo "  $f"
  done
  exit 1
fi 