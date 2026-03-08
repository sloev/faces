#!/bin/bash
set -e

echo "Building Avatar Player for Wasm..."

# 1. Add wasm target if missing
rustup target add wasm32-unknown-unknown

# 2. Build player in release mode
cargo build --release --target wasm32-unknown-unknown -p player

# 3. Copy artifacts to static folder
cp target/wasm32-unknown-unknown/release/player.wasm static/
cp avatar_data.json static/

# 4. Check for texture
if [ -f face.png ]; then
    cp face.png static/
fi

echo "Wasm build complete! Files are in 'static/'."
echo "You can serve them with: 'miniserve static' or 'python3 -m http.server -d static'"
