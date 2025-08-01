#!/usr/bin/env bash
set -euo pipefail

# Build and Deploy Planning Poker
echo "Building and deploying Planning Poker"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Configuration
ENVIRONMENT="${1:-test}"
IMAGE_TAG="${2:-latest}"

log_info "Building and deploying to environment: $ENVIRONMENT"
log_info "Using image tag: $IMAGE_TAG"

# Navigate to project root
cd "$(dirname "$0")/../.."

# Step 1: Build containers
log_info "Step 1: Building container images..."
export ENVIRONMENT="$ENVIRONMENT"
export IMAGE_TAG="$IMAGE_TAG"
export PUSH_IMAGE="true"

if ! ./terraform/scripts/build-container.sh; then
    log_error "Failed to build container images"
    exit 1
fi

log_success "Container images built and pushed successfully"

# Step 2: Show final status
echo
log_success "=== Deployment Summary ==="
echo "Environment: $ENVIRONMENT"
echo "Image Tag: $IMAGE_TAG"
echo "Website URL: https://$([ "$ENVIRONMENT" = "prod" ] && echo "planning-poker.hyperchad.dev" || echo "$ENVIRONMENT.planning-poker.hyperchad.dev")"
echo

log_success "Build and deployment completed successfully!"
