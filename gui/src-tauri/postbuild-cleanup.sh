#!/bin/bash
# Delete all AppleDouble files (._*) in the build output
echo "Cleaning up AppleDouble files..."
find ../../target -name '._*' -type f -delete 