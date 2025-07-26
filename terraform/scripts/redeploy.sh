#!/usr/bin/env bash
set -euo pipefail

echo "🚀 Force redeploying Lambda function and static assets..."

echo "Tainting build resources..."
tofu taint terraform_data.build_lambda
tofu taint terraform_data.build_static
tofu taint terraform_data.upload_assets

echo "Applying changes..."
tofu apply "$@"

echo "✅ Redeploy complete!"
