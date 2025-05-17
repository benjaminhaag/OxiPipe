# Stage 1: Build
FROM rust:1.87 as builder

WORKDIR /app

# Copy the whole project
COPY . .

# Build the binaries (adjust if needed)
RUN cargo build --release --bin cli && \
    cargo build --release --bin scheduler

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install SSL certs and create app directory
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries only
COPY --from=builder /app/target/release/cli /usr/local/bin/oxipipe-cli
COPY --from=builder /app/target/release/scheduler /usr/local/bin/oxipipe

# Copy example pipeline
COPY ./examples/hello.yml /app/hello.yml

# Default command (can be overridden)
CMD ["oxipipe", "--pipeline", "/app/hello.yml", "-j", "hello"]