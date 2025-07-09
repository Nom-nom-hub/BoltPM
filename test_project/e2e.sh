#!/bin/bash
set -euo pipefail

REG_PID=""
cleanup() {
  if [[ -n "$REG_PID" ]]; then
    kill $REG_PID || true
  fi
}
trap cleanup EXIT

cd "$(dirname "$0")"

# Start registry in background
echo "Starting registry..."
cargo run --bin boltpm-registry > registry.log 2>&1 &
REG_PID=$!
sleep 2

echo "Publishing package..."
curl -s -X PUT -F version=1.0.0 -F description="E2E test package" -F tarball=@<(echo 'test tarball contents') http://localhost:4000/v1/e2eshpkg/ || { echo 'Publish failed'; exit 1; }

echo "Yanking package..."
curl -s -X POST http://localhost:4000/v1/e2eshpkg/1.0.0/yank || { echo 'Yank failed'; exit 1; }

echo "Unyanking package..."
curl -s -X POST http://localhost:4000/v1/e2eshpkg/1.0.0/unyank || { echo 'Unyank failed'; exit 1; }

echo "Deprecating package..."
curl -s -X POST -H 'Content-Type: application/json' -d '{"message":"Deprecated for test"}' http://localhost:4000/v1/e2eshpkg/1.0.0/deprecate || { echo 'Deprecate failed'; exit 1; }

echo "Searching for package..."
SEARCH=$(curl -s "http://localhost:4000/v1/search?q=e2eshpkg")
echo "$SEARCH" | grep e2eshpkg || { echo 'Search did not return package'; exit 1; }

echo "Installing package using CLI..."
cargo run --manifest-path ../cli/Cargo.toml --bin boltpm -- install e2eshpkg || { echo 'CLI install failed'; exit 1; }

echo "Checking plugin output..."
if [[ -f .boltpm/plugins_output/PLUGIN_TEST ]]; then
  cat .boltpm/plugins_output/PLUGIN_TEST | grep 'plugin ran' || { echo 'Plugin did not run as expected'; exit 1; }
else
  echo 'Plugin output file not found'; exit 1;
fi

echo "Testing migration scripts..."
for script in npm_to_boltpm.js yarn_to_boltpm.js pnpm_to_boltpm.js; do
  echo "Running $script..."
  node ../scripts/$script || { echo "$script failed"; exit 1; }
  if [[ -f bolt.lock ]]; then
    head -5 bolt.lock | grep 'bolt' || { echo "bolt.lock missing expected content after $script"; exit 1; }
  else
    echo "bolt.lock not created by $script"; exit 1;
  fi
done

echo "E2E test completed successfully!" 