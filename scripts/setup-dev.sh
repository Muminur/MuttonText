#!/bin/bash
set -e

echo "🚀 Setting up MuttonText development environment..."

# Change to project root
cd "$(dirname "$0")/.."

# Check for required tools
echo "🔍 Checking for required tools..."

# Check for Rust
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install from https://rustup.rs/"
    exit 1
fi
echo "✅ Rust $(rustc --version)"

# Check for Cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi
echo "✅ Cargo $(cargo --version)"

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install from https://nodejs.org/"
    exit 1
fi
echo "✅ Node.js $(node --version)"

# Check for npm
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed. Please install Node.js from https://nodejs.org/"
    exit 1
fi
echo "✅ npm $(npm --version)"

# Install frontend dependencies
echo ""
echo "📦 Installing frontend dependencies..."
npm install

# Check Rust compilation
echo ""
echo "🔧 Checking Rust compilation..."
cd src-tauri
cargo check
cd ..

# Success message
echo ""
echo "✅ Development environment setup complete!"
echo ""
echo "📝 Next steps:"
echo "   - Run 'npm run tauri dev' to start development server"
echo "   - Run './scripts/pre-push.sh' before pushing changes"
echo "   - Check CONTRIBUTING.md for development guidelines"
echo ""
