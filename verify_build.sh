#!/bin/bash
# Build verification script for Jožin
# Run this to verify the project is in a clean, buildable state

set -e  # Exit on error

echo "🔧 Jožin Build Verification"
echo "=========================="
echo ""

# Clean previous build
echo "1️⃣  Cleaning previous build..."
cargo clean --quiet
echo "   ✅ Clean complete"
echo ""

# Build workspace
echo "2️⃣  Building workspace (debug)..."
cargo build --workspace --quiet
echo "   ✅ Debug build successful"
echo ""

# Build release
echo "3️⃣  Building workspace (release)..."
cargo build --workspace --release --quiet
echo "   ✅ Release build successful"
echo ""

# Run tests
echo "4️⃣  Running tests..."
cargo test --workspace --quiet
echo "   ✅ All tests passed"
echo ""

# Check for warnings
echo "5️⃣  Checking for warnings..."
WARNINGS=$(cargo build --workspace 2>&1 | grep -c "warning:" || true)
if [ "$WARNINGS" -eq 0 ]; then
    echo "   ✅ No warnings"
else
    echo "   ❌ Found $WARNINGS warnings"
    exit 1
fi
echo ""

# Verify binary works
echo "6️⃣  Testing CLI binary..."
./target/release/jozin --version > /dev/null
echo "   ✅ Binary executes"
echo ""

echo "=========================="
echo "✅ All checks passed!"
echo ""
echo "Binary location: ./target/release/jozin"
echo "Run: ./target/release/jozin --help"
echo ""
