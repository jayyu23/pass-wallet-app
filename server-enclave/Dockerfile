# Use the official Rust image as a parent image
FROM rust:1.75 as builder

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Start a new stage from a smaller image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false appuser

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/server-enclave /app/server-enclave

# Change ownership of the binary to the appuser
RUN chown appuser:appuser /app/server-enclave

# Switch to the non-root user
USER appuser

# Expose the port the app runs on
EXPOSE 5001

# Set environment variable for HTTP port
ENV HTTP_PORT=5001

# Command to run the application
CMD ["./server-enclave"]