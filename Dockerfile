FROM rust:1.87.0-slim-bullseye as builder

# Create a new empty shell project
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Copy manifests and source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src/
COPY static ./static/

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Create app directory
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/blog_api_server /app/blog_api_server
# Copy static assets
COPY --from=builder /usr/src/app/static /app/static

# Create data directory with proper permissions
RUN mkdir -p /app/data && chmod 777 /app/data

# Set the environment variables
ENV RUST_LOG=info

# Expose the port
EXPOSE 3000

# Run the binary
CMD ["/app/blog_api_server"]
