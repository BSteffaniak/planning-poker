#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Show help message
show_help() {
    echo "Usage: $0"
    echo
    echo "Generate Dockerfile for Planning Poker app using clippier."
    echo
    echo "This script generates an optimized Dockerfile and dockerignore"
    echo "for the planning_poker_app package using clippier."
    echo
    echo "Options:"
    echo "  -h, --help    Show this help message"
    echo
    echo "Example:"
    echo "  $0    # Generate PlanningPoker.Dockerfile"
    echo
}

# Utility functions
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

# Build clippier if needed
build_clippier() {
    log_info "Checking if clippier is available..."

    # Check if clippier binary exists (fix tilde expansion)
    if [[ ! -f ~/.cargo/bin/clippier ]]; then
        log_info "Installing clippier tool..."
        if ! cargo install --git https://github.com/MoosicBox/MoosicBox clippier --features git-diff; then
            log_error "Failed to install clippier"
            exit 1
        fi
        log_success "Clippier installed successfully"
    else
        log_info "Clippier already installed at ~/.cargo/bin/clippier"
    fi
}

# Generate dockerfile for planning poker app
generate_dockerfile() {
    local package_name="planning_poker_app"
    local features="postgres,vanilla-js,actix"
    local dockerfile_name="PlanningPoker.Dockerfile"
    local dockerfile_path="packages/app/${dockerfile_name}"

    log_info "Generating Dockerfile and dockerignore for Planning Poker app..."

    # Create backup of existing Dockerfile if it exists
    if [[ -f "$dockerfile_path" ]]; then
        cp "$dockerfile_path" "${dockerfile_path}.backup.$(date +%Y%m%d_%H%M%S)"
        log_warning "Backed up existing Dockerfile to ${dockerfile_path}.backup.*"
    fi

    # Build the command with environment variables and serve argument
    local cmd="~/.cargo/bin/clippier generate-dockerfile . ${package_name} --output $dockerfile_path --build-env PORT=80 --env MAX_THREADS=64 --env ACTIX_WORKERS=32 --arg serve --no-default-features"
    if [[ -n "$features" ]]; then
        cmd="$cmd --features=$features"
    fi

    # Generate the Dockerfile and dockerignore
    if eval "$cmd"; then
        log_success "Generated Dockerfile: $dockerfile_path"
        log_success "Generated dockerignore: ${dockerfile_path%.*}.dockerignore"

        # Post-process Dockerfile for MoosicBox dependencies
        log_info "Post-processing Dockerfile for MoosicBox dependencies..."

        # Fix package directory paths in generated Dockerfile, dockerignore, and workspace members
        log_info "Fixing package paths in generated Dockerfile, dockerignore, and workspace members"
        sed -i 's|packages/planning_poker_app/|packages/app/|g' "$dockerfile_path"
        sed -i 's|/packages/planning_poker_app|/packages/app|g' "${dockerfile_path%.*}.dockerignore"
        sed -i 's|"packages/planning_poker_app"|"packages/app"|g' "$dockerfile_path"

        # Show package count reduction
        local cmd_normal="~/.cargo/bin/clippier workspace-deps . ${package_name}"
        local cmd_all="~/.cargo/bin/clippier workspace-deps . ${package_name} --all-potential-deps"

        if [[ -n "$features" ]]; then
            cmd_normal="$cmd_normal --features=$features"
            cmd_all="$cmd_all --features=$features"
        fi

        local dep_count_normal dep_count_all
        dep_count_normal=$(eval "$cmd_normal" | wc -l)
        dep_count_all=$(eval "$cmd_all" | wc -l)
        log_info "Dependencies: $dep_count_normal actual, $dep_count_all potential (+$((dep_count_all - dep_count_normal)) for Docker compatibility)"
        return 0
    else
        log_error "Failed to generate Dockerfile for $package_name"
        return 1
    fi
}

# Main function
main() {
    # Handle help options
    if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
        show_help
        exit 0
    fi

    log_info "Starting Dockerfile generation for Planning Poker app"

    # Change to script directory
    cd "$(dirname "$0")"

    # Build clippier
    build_clippier

    # Generate dockerfile
    if generate_dockerfile; then
        log_success "Dockerfile generation complete!"
        log_info "You can now build optimized Docker images with minimal dependencies"
        echo
        echo "Example usage:"
        echo "  docker build -f packages/app/PlanningPoker.Dockerfile -t planning-poker ."
        echo
        echo "For Kubernetes deployment:"
        echo "  docker build -f packages/app/PlanningPoker.Dockerfile -t registry.digitalocean.com/your-registry/planning-poker ."
        echo "  docker push registry.digitalocean.com/your-registry/planning-poker"
    else
        log_error "Dockerfile generation failed"
        exit 1
    fi
}

# Run main function
main "$@"
