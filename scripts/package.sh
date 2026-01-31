#!/bin/bash
# Build MuttonText packages for the current platform
set -e

echo "Building MuttonText packages..."
echo "================================"

# Verify prerequisites
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found. Install Rust first."; exit 1; }
command -v npm >/dev/null 2>&1 || { echo "Error: npm not found. Install Node.js first."; exit 1; }

# Build
npm run tauri build

echo ""
echo "Packages built successfully!"
echo "Output: src-tauri/target/release/bundle/"

# List built artifacts
if [ -d "src-tauri/target/release/bundle" ]; then
  echo ""
  echo "Built artifacts:"
  find src-tauri/target/release/bundle -type f \( -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" -o -name "*.dmg" -o -name "*.exe" -o -name "*.msi" \) 2>/dev/null | while read -r f; do
    echo "  $(basename "$f") ($(du -h "$f" | cut -f1))"
  done
fi
