# Build stage
FROM rust:1.75-slim AS builder

WORKDIR /app

# Copy only dependency files first to leverage Docker cache
COPY Cargo.toml Cargo.lock ./

# Build dependencies only
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY src ./src

# Build the application
RUN cargo build --release && \
    strip target/release/cadmium-cloud # Reduce binary size by stripping debug symbols

# Runtime stage
FROM debian:stable-slim

# Install minimal runtime libraries
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl3 ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the compiled binary
COPY --from=builder /app/target/release/cadmium-cloud .

# Set environment variables for logging and optimizations
ENV RUST_LOG=info

# Expose the application port
EXPOSE 8080

# Run the binary
CMD ["./cadmium-cloud"]
