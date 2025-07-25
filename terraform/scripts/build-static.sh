#!/usr/bin/env bash
set -euo pipefail

# Build static assets for HyperChad
echo "Building static assets for environment: ${ENVIRONMENT}"

# Navigate to project root
cd "$(dirname "$0")/../.."

# Clean previous build
rm -rf packages/app/gen/

# Build static assets using HyperChad
echo "Running cargo build for static assets..."
cargo run -p planning_poker_app --release --bin planning-poker-app --no-default-features --features vanilla-js,static-routes,assets,lambda gen

# Verify build output
if [ ! -d "packages/app/gen" ]; then
    echo "Error: Static assets build failed - packages/app/gen/ directory not found"
    exit 1
fi

echo "Static assets built successfully in packages/app/gen/"
ls -la packages/app/gen/
