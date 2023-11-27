#!/bin/sh

# This program utilizes cargo-cross to automate the cross-compilation of
# SRI releases targeting the Rasperry Pi OS.

cross build -p pool_sv2 --target aarch64-unknown-linux-gnu --release
cross build -p translator_sv2 --target aarch64-unknown-linux-gnu --release
cross build -p mining_proxy_sv2 --target aarch64-unknown-linux-gnu --release

TARGET_DIR=target/aarch64-unknown-linux-gnu/release
