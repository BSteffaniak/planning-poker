#!/usr/bin/env bash
set -euo pipefail

# Build container image for Planning Poker
echo "Building container image for environment: ${ENVIRONMENT:-dev}"

# Navigate to project root
cd "$(dirname "$0")/../.."

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
ENVIRONMENT="${ENVIRONMENT:-dev}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
REGISTRY_ENDPOINT="${REGISTRY_ENDPOINT:-registry.digitalocean.com/planning-poker}"
IMAGE_NAME="${REGISTRY_ENDPOINT}/planning-poker:${IMAGE_TAG}"

log_info "Building container image: ${IMAGE_NAME}"

# Generate Dockerfile using clippier
log_info "Generating optimized Dockerfile with clippier..."
if ! ./generate-dockerfile.sh; then
    log_error "Failed to generate Dockerfile"
    exit 1
fi

# Verify Dockerfile was generated
DOCKERFILE_PATH="packages/app/PlanningPoker.Dockerfile"
if [ ! -f "$DOCKERFILE_PATH" ]; then
    log_error "Dockerfile not found at $DOCKERFILE_PATH"
    exit 1
fi

log_success "Dockerfile generated successfully"

# Copy clippier-generated dockerignore for optimal build context
DOCKERIGNORE_PATH="${DOCKERFILE_PATH%.*}.dockerignore"
if [ -f "$DOCKERIGNORE_PATH" ]; then
    log_info "Using clippier-generated .dockerignore for minimal build context"
    cp "$DOCKERIGNORE_PATH" .dockerignore
fi

# Build the container image using standard Docker build
log_info "Building container image using standard Docker build..."
if docker build -f "$DOCKERFILE_PATH" -t "$IMAGE_NAME" .; then
    log_success "Container image built successfully: $IMAGE_NAME"
else
    log_error "Failed to build container image"
    exit 1
fi

# Show image details
log_info "Image details:"
docker images "$IMAGE_NAME" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"

# Optional: Push to registry if PUSH_IMAGE is set
if [ "${PUSH_IMAGE:-false}" = "true" ]; then
    log_info "Pushing image to registry..."

    # Login to DigitalOcean Container Registry
    if [ -n "${DIGITALOCEAN_TOKEN:-}" ]; then
        echo "$DIGITALOCEAN_TOKEN" | docker login "$REGISTRY_ENDPOINT" -u unused --password-stdin
    else
        log_warning "DIGITALOCEAN_TOKEN not set, skipping registry login"
        log_warning "Make sure you're logged in to the registry: doctl registry login"
    fi

    if docker push "$IMAGE_NAME"; then
        log_success "Image pushed successfully: $IMAGE_NAME"
    else
        log_error "Failed to push image to registry"
        exit 1
    fi
fi

log_success "Container build complete!"
echo
echo "To deploy this image:"
echo "  1. Push to registry: PUSH_IMAGE=true $0"
echo "  2. Update Terraform: terraform apply -var=\"image_tag=${IMAGE_TAG}\""
echo "  3. Or use in Kubernetes: kubectl set image deployment/planning-poker planning-poker=${IMAGE_NAME}"
