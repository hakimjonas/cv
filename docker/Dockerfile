FROM rust:1.88.0-slim-bullseye as builder
# NOTE: Consider updating to the latest stable Rust version in the future

# Create a new empty shell project
WORKDIR /usr/src/app

# Install build dependencies in a single layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy only the dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src && \
    rm -rf target/release/.fingerprint/cv-*

# Copy the actual source code
COPY src ./src/
COPY static ./static/
COPY templates ./templates/
COPY data ./data/

# Generate the website files first
RUN cargo run --bin cv

# Build the application
RUN cargo build --release --bin blog_api_server

# Runtime stage
FROM debian:bullseye-slim

# Create app directory
WORKDIR /app

# Install runtime dependencies only
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl1.1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user to run the application
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/blog_api_server /app/blog_api_server

# Copy static assets and generated website files
COPY --from=builder /usr/src/app/static /app/static
COPY --from=builder /usr/src/app/dist /app/dist
COPY --from=builder /usr/src/app/templates /app/templates

# Create data directory with proper permissions
# Use more restrictive permissions (700 instead of 777)
RUN mkdir -p /app/data && \
    chown -R appuser:appuser /app && \
    chmod 700 /app/data

# Set the environment variables
ENV RUST_LOG=info

# Expose the port
EXPOSE 3000

# Switch to non-root user
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Run the binary
CMD ["/app/blog_api_server"]
