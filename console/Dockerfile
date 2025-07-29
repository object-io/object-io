# ObjectIO Console Dockerfile
# Multi-stage build for Leptos WebAssembly frontend

#==============================================================================
# Build Stage
#==============================================================================
FROM rust:1.88-slim as builder

# Install system dependencies for WebAssembly builds
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install WebAssembly target and trunk
RUN rustup target add wasm32-unknown-unknown && \
    cargo install trunk --locked

# Set working directory
WORKDIR /app

# Copy console project files
COPY console/Cargo.toml console/Cargo.lock ./
COPY console/src/ ./src/
COPY console/index.html console/style.css ./

# Build the WebAssembly frontend
RUN trunk build --release

#==============================================================================
# Runtime Stage - Nginx
#==============================================================================
FROM nginx:alpine

# Copy built WebAssembly assets
COPY --from=builder /app/dist/ /usr/share/nginx/html/

# Copy nginx configuration
COPY console/nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost/ || exit 1

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
