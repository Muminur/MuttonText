#!/bin/bash
set -e

echo "🧹 Cleaning MuttonText build artifacts..."

# Change to project root
cd "$(dirname "$0")/.."

# Remove node_modules
if [ -d "node_modules" ]; then
    echo "Removing node_modules..."
    rm -rf node_modules
    echo "✅ Removed node_modules"
else
    echo "✓ node_modules not found"
fi

# Remove Rust target directory
if [ -d "src-tauri/target" ]; then
    echo "Removing src-tauri/target..."
    rm -rf src-tauri/target
    echo "✅ Removed src-tauri/target"
else
    echo "✓ src-tauri/target not found"
fi

# Remove dist directory
if [ -d "dist" ]; then
    echo "Removing dist..."
    rm -rf dist
    echo "✅ Removed dist"
else
    echo "✓ dist not found"
fi

# Remove Vite cache
if [ -d ".vite" ]; then
    echo "Removing .vite cache..."
    rm -rf .vite
    echo "✅ Removed .vite"
else
    echo "✓ .vite not found"
fi

# Remove Vitest cache
if [ -d ".vitest" ]; then
    echo "Removing .vitest cache..."
    rm -rf .vitest
    echo "✅ Removed .vitest"
else
    echo "✓ .vitest not found"
fi

# Remove test coverage
if [ -d "coverage" ]; then
    echo "Removing coverage..."
    rm -rf coverage
    echo "✅ Removed coverage"
else
    echo "✓ coverage not found"
fi

# Remove Playwright test results
if [ -d "test-results" ]; then
    echo "Removing test-results..."
    rm -rf test-results
    echo "✅ Removed test-results"
else
    echo "✓ test-results not found"
fi

# Remove Playwright cache
if [ -d "playwright-report" ]; then
    echo "Removing playwright-report..."
    rm -rf playwright-report
    echo "✅ Removed playwright-report"
else
    echo "✓ playwright-report not found"
fi

# Remove package-lock.json (optional - uncomment if needed)
# if [ -f "package-lock.json" ]; then
#     echo "Removing package-lock.json..."
#     rm package-lock.json
#     echo "✅ Removed package-lock.json"
# fi

# Remove Cargo.lock from src-tauri (optional - uncomment if needed)
# if [ -f "src-tauri/Cargo.lock" ]; then
#     echo "Removing src-tauri/Cargo.lock..."
#     rm src-tauri/Cargo.lock
#     echo "✅ Removed src-tauri/Cargo.lock"
# fi

echo ""
echo "✅ Cleanup complete!"
echo ""
echo "📝 To rebuild:"
echo "   - Run './scripts/setup-dev.sh' to reinstall dependencies"
echo "   - Run 'npm run tauri dev' to start development"
echo ""
