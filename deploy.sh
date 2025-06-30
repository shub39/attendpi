#!/bin/bash

set -e

### CONFIGURATION ###
PI_USER=raspberry
PI_HOST=192.168.31.205         # Replace with Pi's hostname or IP
PI_PATH=/home/raspberry       # Deployment directory on Pi
BIN_NAME=attendpi            # Your binary name (from Cargo.toml)
TARGET=aarch64-unknown-linux-gnu   # 64-bit Pi target

##########################

echo "[*] Adding target (if needed)..."
rustup target add $TARGET

echo "[*] Building project for $TARGET..."
cargo build --release --target=$TARGET

echo "[*] Creating remote directory on Pi..."
ssh ${PI_USER}@${PI_HOST} "mkdir -p $PI_PATH"

echo "[*] Copying binary to Raspberry Pi..."
scp target/$TARGET/release/$BIN_NAME ${PI_USER}@${PI_HOST}:$PI_PATH/

echo "[*] Running."
ssh ${PI_USER}@${PI_HOST} $PI_PATH/$BIN_NAME
