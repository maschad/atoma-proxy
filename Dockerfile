# Builder stage
FROM --platform=$BUILDPLATFORM rust:1.83-slim-bullseye AS builder

# Add platform-specific arguments
ARG TARGETPLATFORM
ARG BUILDPLATFORM
ARG TARGETARCH

# Trace level argument
ARG TRACE_LEVEL
ARG PROFILE

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/atoma-proxy

COPY . .


# Compile
RUN if [ "$PROFILE" = "cloud" ]; then \
    RUST_LOG=${TRACE_LEVEL} cargo build --release --bin atoma-proxy --features google-oauth; \
    else \
    RUST_LOG=${TRACE_LEVEL} cargo build --release --bin atoma-proxy; \
    fi

# Final stage
FROM --platform=$TARGETPLATFORM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create necessary directories
RUN mkdir -p /app/logs

# Copy the built binary from builder stage
COPY --from=builder /usr/src/atoma-proxy/target/release/atoma-proxy /usr/local/bin/atoma-proxy

# Set executable permissions explicitly
RUN chmod +x /usr/local/bin/atoma-proxy

# Copy and set up entrypoint script
COPY entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/entrypoint.sh

# Copy host client.yaml and modify keystore path
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

CMD ["/usr/local/bin/atoma-proxy", "--config-path", "/app/config.toml"]
