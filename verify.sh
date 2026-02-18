#!/bin/bash
# Build Verification Script

echo "🔍 Checking project structure..."

# Check all required files exist
REQUIRED_FILES=(
    "Cargo.toml"
    "build.rs"
    "proto/diffusion.proto"
    "config/default.toml"
    "src/main.rs"
    "src/config.rs"
    "src/errors.rs"
    "src/inference/mod.rs"
    "src/inference/pipeline.rs"
    "src/queue/mod.rs"
    "src/queue/memory.rs"
    "src/server/mod.rs"
    "src/server/grpc.rs"
    "src/server/rest.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✓ $file"
    else
        echo "✗ Missing: $file"
        exit 1
    fi
done

echo ""
echo "✓ All files present!"
echo ""
echo "📦 Project structure:"
tree -L 3 -I 'target|cache|models|output' .

echo ""
echo "To build the project:"
echo "  cargo build --release"
echo ""
echo "To run:"
echo "  cargo run --release"
