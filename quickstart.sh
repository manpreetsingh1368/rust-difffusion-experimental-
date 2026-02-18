#!/bin/bash
set -e

echo "🎨 Diffusion Server - Quick Start"
echo "=================================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    echo "Install from: https://rustup.rs/"
    exit 1
fi

echo "✓ Rust is installed"

# Check if protoc is installed
if ! command -v protoc &> /dev/null; then
    echo "⚠ Protocol Buffers compiler not found"
    echo "Installing protobuf-compiler..."
    
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y protobuf-compiler
    elif command -v brew &> /dev/null; then
        brew install protobuf
    else
        echo "Please install protobuf-compiler manually"
        exit 1
    fi
fi

echo "✓ Protocol Buffers compiler is installed"

# Create necessary directories
echo ""
echo "Creating directories..."
mkdir -p models cache output

# Build the project
echo ""
echo "Building project (this may take a few minutes)..."
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Build successful!"
    echo ""
    echo "🚀 To run the server:"
    echo "   ./target/release/diffusion-server"
    echo ""
    echo "📡 The server will be available at:"
    echo "   - gRPC: localhost:50051"
    echo "   - REST: http://localhost:8080"
    echo ""
    echo "🔍 Test the health endpoint:"
    echo "   curl http://localhost:8080/health"
    echo ""
    echo "📖 See README.md for more information"
else
    echo "❌ Build failed!"
    exit 1
fi
