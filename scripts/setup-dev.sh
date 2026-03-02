#!/bin/bash
set -e

echo "🚀 Setting up MuttonText development environment..."

# Change to project root
cd "$(dirname "$0")/.."

# Detect OS
OS="$(uname -s)"

# Check for required tools
echo "🔍 Checking for required tools..."

# Check for Xcode Command Line Tools (macOS only)
if [ "$OS" = "Darwin" ]; then
    if ! xcode-select -p &> /dev/null; then
        echo "📦 Installing Xcode Command Line Tools..."
        xcode-select --install
        echo "⏳ Please complete the Xcode CLT installation, then re-run this script."
        exit 1
    fi
    echo "✅ Xcode Command Line Tools"
fi

# Check for Rust - auto-install if missing
if ! command -v rustc &> /dev/null; then
    # Try sourcing cargo env in case it's installed but not in PATH
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
fi

if ! command -v rustc &> /dev/null; then
    echo "📦 Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo "✅ Rust $(rustc --version)"

# Check for Cargo (should always be present with Rust)
if ! command -v cargo &> /dev/null; then
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
fi
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi
echo "✅ Cargo $(cargo --version)"

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install from https://nodejs.org/"
    if [ "$OS" = "Darwin" ] && command -v brew &> /dev/null; then
        echo "   Or run: brew install node"
    fi
    exit 1
fi
echo "✅ Node.js $(node --version)"

# Check for npm
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed. Please install Node.js from https://nodejs.org/"
    exit 1
fi
echo "✅ npm $(npm --version)"

# macOS: remind user about Accessibility permissions
if [ "$OS" = "Darwin" ]; then
    echo ""
    echo "ℹ️  macOS Note: MuttonText requires Accessibility permissions to monitor keyboard input."
    echo "   When you first run the app, go to:"
    echo "   System Settings → Privacy & Security → Accessibility → enable MuttonText"
fi

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
