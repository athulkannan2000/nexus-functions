#!/bin/bash

# Build script for hello-world example

echo "Building hello-world function..."

cargo build --target wasm32-wasi --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful"
    
    # Create build directory if it doesn't exist
    mkdir -p ../../build
    
    # Copy WASM binary
    cp target/wasm32-wasi/release/hello_world.wasm ../../build/handler.wasm
    
    echo "✓ WASM binary copied to build/handler.wasm"
    
    # Show file size
    SIZE=$(du -h ../../build/handler.wasm | cut -f1)
    echo "  Size: $SIZE"
else
    echo "✗ Build failed"
    exit 1
fi
