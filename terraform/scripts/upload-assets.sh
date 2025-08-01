#!/usr/bin/env bash
set -euo pipefail

# Upload static assets to Cloudflare R2
echo "Uploading static assets to Cloudflare R2 bucket: ${BUCKET_NAME}"

# Check if AWS CLI is available (R2 uses S3-compatible API)
if ! command -v aws &> /dev/null; then
    echo "Error: AWS CLI not found. Please install AWS CLI."
    exit 1
fi

# Check if source directory exists
if [ ! -d "${SOURCE_DIR}" ]; then
    echo "Error: Source directory ${SOURCE_DIR} not found"
    exit 1
fi

# Check if R2 credentials are set
if [ -z "${AWS_ACCESS_KEY_ID:-}" ] || [ -z "${AWS_SECRET_ACCESS_KEY:-}" ] || [ -z "${R2_ACCOUNT_ID:-}" ]; then
    echo "Error: R2 credentials not set. Please set AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, and R2_ACCOUNT_ID environment variables."
    echo "Get R2 credentials from Cloudflare Dashboard > R2 Object Storage > Manage R2 API tokens"
    echo "Get R2_ACCOUNT_ID from the right sidebar of your Cloudflare dashboard"
    exit 1
fi

# Upload files to R2 with appropriate content types
echo "Syncing ${SOURCE_DIR} to R2 bucket s3://${BUCKET_NAME}/"

aws s3 sync "${SOURCE_DIR}/" "s3://${BUCKET_NAME}/" \
    --endpoint-url "https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com" \
    --region auto \
    --delete \
    --cache-control "max-age=31536000,public,immutable" \
    --metadata-directive REPLACE

# Set specific cache control for HTML files (shorter cache)
aws s3 cp "${SOURCE_DIR}/" "s3://${BUCKET_NAME}/" \
    --endpoint-url "https://${R2_ACCOUNT_ID}.r2.cloudflarestorage.com" \
    --region auto \
    --recursive \
    --exclude "*" \
    --include "*.html" \
    --cache-control "max-age=300,public" \
    --metadata-directive REPLACE

echo "Assets uploaded successfully to Cloudflare R2"
