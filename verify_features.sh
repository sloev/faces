#!/bin/bash
set -e

echo "--- 🔍 EMPIRICAL FEATURE VERIFICATION ---"

# 1. Test Generator
echo "Running generator..."
cargo run --release -p generator

# 2. Assert Asset Existence
ASSETS=("assets/timeline.json" "assets/face.jpg")
for file in "${ASSETS[@]}"; do
    if [ -s "$file" ]; then
        echo "✅ $file exists and is not empty ($(stat -c%s "$file") bytes)"
    else
        echo "❌ $file is missing or empty!"
        exit 1
    fi
done

# 3. Test Wasm Compilation
echo "Testing Wasm compilation..."
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown -p player
if [ -s "target/wasm32-unknown-unknown/release/player.wasm" ]; then
    echo "✅ Wasm binary generated ($(stat -c%s "target/wasm32-unknown-unknown/release/player.wasm") bytes)"
else
    echo "❌ Wasm binary missing!"
    exit 1
fi

echo "--- ✅ LOCAL VERIFICATION PASSED ---"
