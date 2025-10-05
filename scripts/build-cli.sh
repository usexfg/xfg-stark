#!/bin/bash

# XFG STARK CLI Build Script
# This script builds the CLI tool and runs basic tests

set -e

echo "🔨 Building XFG STARK CLI Tool..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Cargo.toml not found. Please run this script from the xfgwin directory."
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build the CLI tool
echo "⚡ Building CLI tool..."
cargo build --release --bin xfg-stark-cli

# Check if build was successful
if [ -f "target/release/xfg-stark-cli" ]; then
    echo "✅ CLI tool built successfully!"
    echo "📁 Location: target/release/xfg-stark-cli"
    
    # Show help
    echo "📖 CLI Help:"
    ./target/release/xfg-stark-cli --help
    
    # Run tests
    echo "🧪 Running tests..."
    cargo test
    
    echo "🎉 Build and test completed successfully!"
    echo ""
    echo "💡 Next steps:"
    echo "   1. Install CLI: sudo cp target/release/xfg-stark-cli /usr/local/bin/"
    echo "   2. Create template: xfg-stark-cli create-template standard -o template.json"
    echo "   3. Create package: xfg-stark-cli create-package --template template.json --burn-amount 0.8 --txn-hash 0x123... --recipient 0x456... --secret my-secret --output package.json"
    echo "   4. Generate proof: xfg-stark-cli generate -i package.json -o proof.json"
else
    echo "❌ Build failed!"
    exit 1
fi
