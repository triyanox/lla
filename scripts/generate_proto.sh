#!/bin/bash
set -e

cd "$(dirname "$0")/.."

# Build with the regenerate-protobuf feature
echo "Building with regenerate-protobuf feature..."
cargo build --features regenerate-protobuf

echo "Successfully generated protobuf bindings"
