---
name: rust-devops
description: Expert guidance for Rust DevOps including cross-compilation with cargo-zigbuild and Cross, CI/CD pipelines with GitHub Actions, containerized deployments to AWS Lambda, Google Cloud Run, and Azure, static linking with musl, and release automation
hash: acb1e43ff3a58994
---

# Rust DevOps

Build, deploy, and release Rust applications across platforms. Covers cross-compilation, containerization, cloud deployment, and CI/CD automation.

## Core Principles

- Use `cargo-zigbuild` for simple cross-compilation without GCC toolchains
- Use `Cross` (Docker-based) for complex native dependencies
- Prefer `musl` targets for portable, statically-linked Linux binaries
- Use multi-stage Dockerfiles with distroless/scratch base images
- Pin toolchain versions via `rust-toolchain.toml` for reproducibility
- Audit dependencies with `cargo audit` and `cargo tree --duplicates`
- Use GitHub Actions matrix builds for multi-platform CI
- Compile to release mode with LTO for production binaries

## Quick Reference

### Add Cross-Compilation Target

```bash
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-gnu
rustup target add wasm32-unknown-unknown
```

### Build with cargo-zigbuild

```bash
cargo install --locked cargo-zigbuild
cargo zigbuild --target x86_64-unknown-linux-musl --release
```

### Build with Cross (Docker)

```bash
cargo install cross
cross build --target aarch64-unknown-linux-gnu --release
```

### Common Target Triples

| Target | Description |
|--------|-------------|
| `x86_64-unknown-linux-gnu` | Linux x64 (glibc) |
| `x86_64-unknown-linux-musl` | Linux x64 (static) |
| `aarch64-unknown-linux-gnu` | Linux ARM64 |
| `x86_64-pc-windows-gnu` | Windows x64 (MinGW) |
| `aarch64-apple-darwin` | macOS ARM64 |
| `wasm32-unknown-unknown` | WebAssembly |

## Topics

### Build & Compilation

- [Cross-Platform Builds](./cross-platform-builds.md) - Detailed cross-compilation strategies
- [Container Builds](./container-builds.md) - Multi-stage Dockerfiles and optimization

### Deployment

- [Cloud Deployment](./cloud-deployment.md) - AWS Lambda, Cloud Run, Azure Container Apps

### Automation

- [CI/CD Workflows](./cicd-workflows.md) - GitHub Actions templates and automation

## Common Patterns

### Minimal Dockerfile for Rust Binary

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/myapp /
CMD ["/myapp"]
```

### Cargo Config for Cross-Compilation

```toml
# .cargo/config.toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.x86_64-unknown-linux-musl]
linker = "rust-lld"
```

### Release Build Optimizations

```toml
# Cargo.toml
[profile.release]
lto = true
codegen-units = 1
strip = true
```

## Resources

- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)
- [Cross](https://github.com/cross-rs/cross)
- [Rustup Book](https://rust-lang.github.io/rustup/)
