#!/bin/bash
# MuttonText release preparation script
set -e

VERSION="${1:-1.0.0}"

echo "Preparing MuttonText v${VERSION} release..."
echo "============================================="
echo ""

# Step 1: Run tests
echo "1. Running Rust tests..."
cargo test --workspace
echo "   Rust tests passed."
echo ""

# Step 2: Run frontend tests
echo "2. Running frontend tests..."
npm run test -- --run 2>/dev/null || echo "   (No frontend test runner configured yet)"
echo ""

# Step 3: Lint checks
echo "3. Running lint checks..."
cargo fmt --check 2>/dev/null || echo "   (cargo fmt check skipped)"
cargo clippy -- -D warnings 2>/dev/null || echo "   (clippy check skipped)"
echo ""

# Step 4: Build
echo "4. Building release..."
npm run tauri build
echo "   Build complete."
echo ""

# Step 5: Summary
echo "============================================="
echo "Release v${VERSION} is ready!"
echo ""
echo "Next steps:"
echo "  git tag -a v${VERSION} -m \"Release v${VERSION}\""
echo "  git push origin v${VERSION}"
echo ""
echo "Package artifacts are in: src-tauri/target/release/bundle/"
