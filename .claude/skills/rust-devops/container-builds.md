# Container Builds

Optimized Docker strategies for Rust applications with multi-stage builds and minimal images.

## Multi-Stage Dockerfile Pattern

Separates build environment from runtime, producing minimal final images.

```dockerfile
# Build stage
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/myapp /myapp
CMD ["/myapp"]
```

## Base Image Selection

| Image | Size | Use Case |
|-------|------|----------|
| `scratch` | 0 MB | Fully static binaries only |
| `gcr.io/distroless/static` | ~2 MB | Static binaries, includes CA certs |
| `gcr.io/distroless/cc` | ~20 MB | Binaries needing libc |
| `alpine` | ~5 MB | When you need a shell for debugging |

## Multi-Architecture Builds

Build for multiple architectures in a single pipeline using cargo-zigbuild.

```dockerfile
FROM rust:latest AS builder
RUN cargo install cargo-zigbuild
WORKDIR /app
COPY . .

# Build for both architectures
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl && \
    cargo zigbuild --release \
      --target x86_64-unknown-linux-musl \
      --target aarch64-unknown-linux-musl && \
    mkdir -p /app/linux && \
    cp target/aarch64-unknown-linux-musl/release/myapp /app/linux/arm64 && \
    cp target/x86_64-unknown-linux-musl/release/myapp /app/linux/amd64

FROM alpine:latest AS runtime
ARG TARGETPLATFORM
WORKDIR /app
COPY --from=builder /app/linux/${TARGETPLATFORM##*/} /app/myapp
CMD ["/app/myapp"]
```

Build command:

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t myapp:latest .
```

## Build Caching Strategies

### Dependency Layer Caching

Cache dependencies separately from source code:

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build actual application
COPY . .
RUN cargo build --release
```

### Using cargo-chef for Better Caching

```dockerfile
FROM rust:1.75 as chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/myapp /
CMD ["/myapp"]
```

## Optimization Tips

### Reduce Final Image Size

```toml
# Cargo.toml
[profile.release]
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

### Security Hardening

- Use distroless images (no shell, no package manager)
- Run as non-root user
- Scan images with Trivy or Snyk

```dockerfile
FROM gcr.io/distroless/static:nonroot
COPY --from=builder /app/target/release/myapp /
USER nonroot:nonroot
CMD ["/myapp"]
```

## Common Issues

### Missing CA Certificates

For HTTPS requests, use `distroless/static` or add certs:

```dockerfile
FROM alpine as certs
RUN apk add --no-cache ca-certificates

FROM scratch
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/release/myapp /
```

### Missing libc

If binary fails with "not found" error, you're missing libc. Either:
- Build with musl for static binary
- Use `distroless/cc` base image

## Related

- [Cross-Platform Builds](./cross-platform-builds.md)
- [Cloud Deployment](./cloud-deployment.md)
