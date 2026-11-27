#!/bin/sh
set -e
echo "demo-check: cargo fmt --check"
cargo fmt --check
echo "demo-check: cargo test"
cargo test --quiet
echo "demo-check: all good"
