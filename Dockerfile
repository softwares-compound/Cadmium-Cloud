# Build stage
FROM rust:1.75-slim AS builder

WORKDIR /app

# Copy only dependency files first to leverage cache
COPY Cargo.toml Cargo.lock ./

# Build dependencies only
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy the actual source code
COPY src ./src

# Build the actual application
RUN cargo build --release && \
    strip target/release/cadmium-cloud

# Runtime stage
FROM debian:stable-slim

# Install only the essential runtime library
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl3 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the compiled binary
COPY --from=builder /app/target/release/cadmium-cloud .

ENV RUST_LOG=info

EXPOSE 8080

CMD ["./cadmium-cloud"]