#!/bin/bash
set -euo pipefail

# Upload static assets to S3
echo "Uploading static assets to S3 bucket: ${BUCKET_NAME}"

# Check if AWS CLI is available
if ! command -v aws &> /dev/null; then
    echo "Error: AWS CLI not found. Please install AWS CLI."
    exit 1
fi

# Check if source directory exists
if [ ! -d "${SOURCE_DIR}" ]; then
    echo "Error: Source directory ${SOURCE_DIR} not found"
    exit 1
fi

# Upload files to S3 with appropriate content types
echo "Syncing ${SOURCE_DIR} to s3://${BUCKET_NAME}/"

aws s3 sync "${SOURCE_DIR}/" "s3://${BUCKET_NAME}/" \
    --delete \
    --cache-control "max-age=31536000,public,immutable" \
    --metadata-directive REPLACE

# Set specific cache control for HTML files (shorter cache)
aws s3 cp "${SOURCE_DIR}/" "s3://${BUCKET_NAME}/" \
    --recursive \
    --exclude "*" \
    --include "*.html" \
    --cache-control "max-age=300,public" \
    --metadata-directive REPLACE

echo "Assets uploaded successfully to S3"
