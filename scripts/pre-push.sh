#!/bin/bash
set -e

echo "🔍 Running pre-push validation..."

# Change to project root
cd "$(dirname "$0")/.."

echo "📦 Checking Rust formatting..."
cd src-tauri
cargo fmt --check
cd ..

echo "🔎 Running Clippy..."
cd src-tauri
cargo clippy --all-targets --all-features -- -D warnings
cd ..

echo "🧪 Running Rust tests..."
cd src-tauri
cargo test --workspace
cd ..

echo "📦 Checking TypeScript..."
npm run typecheck

echo "🔎 Running ESLint..."
npm run lint

echo "🧪 Running frontend tests..."
npm run test

echo "🏗️ Building application..."
npm run build

echo "✅ All checks passed! Safe to push."
