# Optimized Dockerfile for Git2Page
FROM rust:1.78-alpine as builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    build-base \
    perl \
    musl-dev \
    openssl-dev \
    pkgconfig

# Ensure explicit target is available in CI/buildx environments
RUN rustup target add x86_64-unknown-linux-musl

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl && rm -rf src

# Copy actual source code
COPY src ./src
COPY static ./static

# Build the application
RUN touch src/main.rs && cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    curl

# Create non-root user
RUN adduser -D -s /bin/sh git2page

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/git2page /usr/local/bin/git2page

# Copy static files
COPY --from=builder /app/static ./static

# Change ownership
RUN chown -R git2page:git2page /app

USER git2page

# Expose port
EXPOSE 5001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5001/config || exit 1

# Run the application
CMD ["git2page"]
