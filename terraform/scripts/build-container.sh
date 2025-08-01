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
PLANNING_POKER_IMAGE="${REGISTRY_ENDPOINT}/planning-poker:${IMAGE_TAG}"

log_info "Building container images:"
log_info "  Planning Poker: ${PLANNING_POKER_IMAGE}"

# Generate Planning Poker Dockerfile using clippier
log_info "Generating optimized Planning Poker Dockerfile with clippier..."
if ! ./generate-dockerfile.sh; then
    log_error "Failed to generate Planning Poker Dockerfile"
    exit 1
fi

# Verify Planning Poker Dockerfile was generated
PLANNING_POKER_DOCKERFILE="packages/app/PlanningPoker.Dockerfile"
if [ ! -f "$PLANNING_POKER_DOCKERFILE" ]; then
    log_error "Planning Poker Dockerfile not found at $PLANNING_POKER_DOCKERFILE"
    exit 1
fi

log_success "Planning Poker Dockerfile generated successfully"

# Build the Planning Poker container image
log_info "Building Planning Poker container image..."
if docker build -f "$PLANNING_POKER_DOCKERFILE" -t "$PLANNING_POKER_IMAGE" .; then
    log_success "Planning Poker container image built successfully: $PLANNING_POKER_IMAGE"
else
    log_error "Failed to build Planning Poker container image"
    exit 1
fi

# Show image details
log_info "Image details:"
docker images "$PLANNING_POKER_IMAGE" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"

# Optional: Push to registry if PUSH_IMAGE is set
if [ "${PUSH_IMAGE:-false}" = "true" ]; then
    log_info "Pushing images to registry..."

    # Login to DigitalOcean Container Registry
    if [ -n "${DIGITALOCEAN_TOKEN:-}" ]; then
        echo "$DIGITALOCEAN_TOKEN" | docker login "$REGISTRY_ENDPOINT" -u unused --password-stdin
    else
        log_warning "DIGITALOCEAN_TOKEN not set, skipping registry login"
        log_warning "Make sure you're logged in to the registry: doctl registry login"
    fi

    # Push Planning Poker image
    if docker push "$PLANNING_POKER_IMAGE"; then
        log_success "Planning Poker image pushed successfully: $PLANNING_POKER_IMAGE"
    else
        log_error "Failed to push Planning Poker image to registry"
        exit 1
    fi
fi

log_success "Container build complete!"
echo
echo "To deploy these images:"
echo "  1. Push to registry: PUSH_IMAGE=true $0"
echo "  2. Update Terraform: terraform apply -var=\"image_tag=${IMAGE_TAG}\""
echo "  3. Or use in Kubernetes:"
echo "     kubectl set image deployment/planning-poker planning-poker=${PLANNING_POKER_IMAGE}"
