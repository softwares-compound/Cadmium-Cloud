# Stage 1: Build the application
FROM rust:1.66.0 as builder

# Set the working directory inside the container
WORKDIR /app

# Copy the entire project into the container
COPY . .

# Build the application in release mode
RUN cargo build --release

# Stage 2: Create a minimal runtime image
FROM debian:buster-slim

# Install necessary system dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/cadmium-cloud .

# Expose the port your application will run on
EXPOSE 8080

# Set the entry point to run the application
CMD ["./cadmium-cloud"]
