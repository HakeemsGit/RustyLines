#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Define the target triple
TARGET="x86_64-unknown-linux-gnu"

# Define the binary name based on the target OS
if [[ "$TARGET" == *"windows"* ]]; then
    BINARY_NAME="rustylines.exe"
else
    BINARY_NAME="rustylines"
fi

# Build the project
echo "Building the project for target: $TARGET"
cargo build --release --target "$TARGET"

# Define the path to the built binary
BUILT_BINARY="target/$TARGET/release/$BINARY_NAME"

# Check if the binary exists
if [ -f "$BUILT_BINARY" ]; then
    echo "Build successful. Copying binary to the main directory..."
    cp "$BUILT_BINARY" .
    echo "Binary copied to the main directory."
else
    echo "Error: Built binary not found at $BUILT_BINARY"
    exit 1
fi
