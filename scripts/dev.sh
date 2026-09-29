#!/usr/bin/env bash
set -e

# Check prerequisites
command -v cargo >/dev/null 2>&1 || { echo >&2 "cargo required but not installed. Aborting."; exit 1; }
command -v node >/dev/null 2>&1 || { echo >&2 "node required but not installed. Aborting."; exit 1; }
command -v pnpm >/dev/null 2>&1 || { echo >&2 "pnpm required but not installed. Aborting."; exit 1; }

if [ -z "$OCCT_ROOT" ]; then
    echo "Warning: OCCT_ROOT is not set. Ensure OCCT is installed locally."
fi

# Install web deps if needed
(cd web && pnpm install)

# Trap SIGINT
trap 'kill $(jobs -p)' SIGINT SIGTERM

echo "Starting Rust API server..."
cargo run &

echo "Starting Vite dev server..."
(cd web && pnpm dev) &

wait
