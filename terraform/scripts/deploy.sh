#!/usr/bin/env bash
set -euo pipefail

# Deploy Planning Poker to Kubernetes
echo "Deploying Planning Poker to Kubernetes"

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
ENVIRONMENT="${1:-dev}"
IMAGE_TAG="${2:-latest}"

log_info "Deploying to environment: $ENVIRONMENT"
log_info "Using image tag: $IMAGE_TAG"

# Navigate to terraform directory
cd "$(dirname "$0")/.."

# Check if terraform workspace exists
if ! terraform workspace list | grep -q "$ENVIRONMENT"; then
    log_info "Creating terraform workspace: $ENVIRONMENT"
    terraform workspace new "$ENVIRONMENT"
else
    log_info "Switching to terraform workspace: $ENVIRONMENT"
    terraform workspace select "$ENVIRONMENT"
fi

# Initialize terraform
log_info "Initializing Terraform..."
if ! terraform init; then
    log_error "Failed to initialize Terraform"
    exit 1
fi

# Plan the deployment
log_info "Planning Terraform deployment..."
if ! terraform plan -var="image_tag=$IMAGE_TAG" -out="tfplan"; then
    log_error "Terraform plan failed"
    exit 1
fi

# Ask for confirmation
echo
log_warning "About to apply the following changes:"
terraform show tfplan
echo
read -p "Do you want to proceed with the deployment? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    log_info "Deployment cancelled"
    rm -f tfplan
    exit 0
fi

# Apply the changes
log_info "Applying Terraform changes..."
if terraform apply tfplan; then
    log_success "Deployment completed successfully!"
else
    log_error "Deployment failed"
    exit 1
fi

# Clean up
rm -f tfplan

# Show deployment status
log_info "Deployment status:"
terraform output

# Show kubectl commands for monitoring
echo
log_info "Useful kubectl commands for monitoring:"
echo "  kubectl get pods -n planning-poker-$ENVIRONMENT"
echo "  kubectl get services -n planning-poker-$ENVIRONMENT"
echo "  kubectl get certificates -n planning-poker-$ENVIRONMENT"
echo "  kubectl logs -f deployment/planning-poker -n planning-poker-$ENVIRONMENT"
echo "  kubectl logs -f deployment/moosicbox-lb -n planning-poker-$ENVIRONMENT"
echo
echo "Website URL: $(terraform output -raw website_url)"
