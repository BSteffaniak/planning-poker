# Multi-stage build for planning poker server
FROM rust:1.75 as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY packages/ packages/

# Build dependencies (this is cached if Cargo.toml doesn't change)
RUN cargo build --release --bin planning-poker-server

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Copy binary from builder stage
COPY --from=builder /app/target/release/planning-poker-server /usr/local/bin/planning-poker-server

# Create data directory
RUN mkdir -p /app/data && chown appuser:appuser /app/data

# Switch to app user
USER appuser

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the server
CMD ["planning-poker-server", "--host", "0.0.0.0", "--port", "8080", "--database-url", "sqlite:///app/data/planning_poker.db"]