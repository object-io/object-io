# ObjectIO Server Dockerfile
# Multi-stage build for optimal size and security

#==============================================================================
# Build Stage
#==============================================================================
FROM rust:1.83-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all crates
COPY crates/ ./crates/

# Build the server binary
RUN cargo build --release --bin object-io-server

#==============================================================================
# Runtime Stage
#==============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r objectio \
    && useradd -r -g objectio objectio

# Create necessary directories
RUN mkdir -p /app/data /app/logs \
    && chown -R objectio:objectio /app

# Copy binary from builder stage
COPY --from=builder --chown=objectio:objectio /app/target/release/object-io-server /app/object-io-server

# Switch to non-root user
USER objectio

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["./object-io-server"]
