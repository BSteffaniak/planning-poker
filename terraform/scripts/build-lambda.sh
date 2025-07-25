#!/usr/bin/env bash
set -euo pipefail

# Build Lambda function for HyperChad
echo "Building Lambda function for environment: ${ENVIRONMENT}"

# Navigate to project root
cd "$(dirname "$0")/../.."

# Clean previous build
rm -rf target/lambda/

# Build Lambda function
if [ "${DEBUG_MODE:-false}" = "true" ]; then
    echo "Building Lambda in DEBUG mode..."
    cargo lambda build -p planning_poker_app --bin planning-poker-app-lambda --no-default-features --features lambda,vanilla-js,postgres
else
    echo "Building Lambda in RELEASE mode..."
    cargo lambda build -p planning_poker_app --release --bin planning-poker-app-lambda --no-default-features --features lambda,vanilla-js,postgres
fi

# Verify build output
if [ ! -f "target/lambda/planning-poker-app-lambda/bootstrap" ]; then
    echo "Error: Lambda build failed - bootstrap binary not found"
    exit 1
fi

# Create zip file for Lambda deployment
echo "Creating bootstrap.zip for Lambda deployment..."
cd target/lambda/planning-poker-app-lambda
zip bootstrap.zip bootstrap
cd ../../..

# Verify zip file was created
if [ ! -f "target/lambda/planning-poker-app-lambda/bootstrap.zip" ]; then
    echo "Error: Failed to create bootstrap.zip"
    exit 1
fi

echo "Lambda function built successfully:"
ls -la target/lambda/planning-poker-app-lambda/
