#!/usr/bin/env bash
set -euo pipefail

# Build and Deploy Planning Poker with MoosicBox Load Balancer
echo "Building and deploying Planning Poker with MoosicBox Load Balancer"

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

# Step 2: Deploy to Kubernetes
log_info "Step 2: Deploying to Kubernetes..."
if ! ./terraform/scripts/deploy.sh "$ENVIRONMENT" "$IMAGE_TAG"; then
    log_error "Failed to deploy to Kubernetes"
    exit 1
fi

log_success "Deployment completed successfully!"

# Step 3: Wait for services to be ready
log_info "Step 3: Waiting for services to be ready..."
NAMESPACE="planning-poker-$ENVIRONMENT"

log_info "Waiting for Planning Poker deployment to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/planning-poker -n "$NAMESPACE" || {
    log_error "Planning Poker deployment failed to become ready"
    kubectl describe deployment/planning-poker -n "$NAMESPACE"
    exit 1
}

log_info "Waiting for MoosicBox Load Balancer deployment to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/moosicbox-lb -n "$NAMESPACE" || {
    log_error "MoosicBox Load Balancer deployment failed to become ready"
    kubectl describe deployment/moosicbox-lb -n "$NAMESPACE"
    exit 1
}

log_info "Waiting for LoadBalancer IP to be assigned..."
EXTERNAL_IP=""
for i in {1..30}; do
    EXTERNAL_IP=$(kubectl get service moosicbox-lb-service -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "")
    if [ -n "$EXTERNAL_IP" ] && [ "$EXTERNAL_IP" != "null" ]; then
        break
    fi
    log_info "Waiting for LoadBalancer IP... (attempt $i/30)"
    sleep 10
done

if [ -z "$EXTERNAL_IP" ] || [ "$EXTERNAL_IP" = "null" ]; then
    log_warning "LoadBalancer IP not assigned yet. Check status manually:"
    echo "  kubectl get service moosicbox-lb-service -n $NAMESPACE"
else
    log_success "LoadBalancer IP assigned: $EXTERNAL_IP"
fi

# Step 4: Check certificate status
log_info "Step 4: Checking certificate status..."
kubectl get certificate planning-poker-tls -n "$NAMESPACE" -o wide || {
    log_warning "Certificate not found or not ready yet"
}

# Step 5: Show final status
echo
log_success "=== Deployment Summary ==="
echo "Environment: $ENVIRONMENT"
echo "Image Tag: $IMAGE_TAG"
echo "Namespace: $NAMESPACE"
if [ -n "$EXTERNAL_IP" ] && [ "$EXTERNAL_IP" != "null" ]; then
    echo "LoadBalancer IP: $EXTERNAL_IP"
fi
echo "Website URL: https://$([ "$ENVIRONMENT" = "prod" ] && echo "planning-poker.hyperchad.dev" || echo "$ENVIRONMENT.planning-poker.hyperchad.dev")"
echo

log_info "Monitoring commands:"
echo "  kubectl get pods -n $NAMESPACE"
echo "  kubectl get services -n $NAMESPACE"
echo "  kubectl get certificates -n $NAMESPACE"
echo "  kubectl logs -f deployment/planning-poker -n $NAMESPACE"
echo "  kubectl logs -f deployment/moosicbox-lb -n $NAMESPACE"
echo

log_success "Build and deployment completed successfully!"
