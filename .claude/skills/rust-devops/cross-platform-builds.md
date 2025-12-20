# Cross-Platform Builds

Strategies for building Rust binaries for multiple target platforms from a single development environment.

## Tool Selection

| Approach | Best For | Trade-offs |
|----------|----------|------------|
| **cargo-zigbuild** | Simple projects, CI/CD | Requires Zig; handles most cases |
| **Cross** | Complex native deps | Requires Docker; pre-configured environments |
| **Native toolchain** | Full control | Manual linker setup per target |

## cargo-zigbuild

Uses Zig's cross-linker to simplify compilation. No GCC toolchain installation required.

### Installation

```bash
# Via cargo
cargo install --locked cargo-zigbuild

# Via pip (includes Zig)
pip install cargo-zigbuild
```

### Usage

```bash
# Add target first
rustup target add aarch64-unknown-linux-gnu

# Build
cargo zigbuild --target aarch64-unknown-linux-gnu --release

# Build for multiple targets
cargo zigbuild --release \
  --target x86_64-unknown-linux-musl \
  --target aarch64-unknown-linux-musl
```

### glibc Version Control

```bash
# Target specific glibc version
cargo zigbuild --target x86_64-unknown-linux-gnu.2.17 --release
```

## Cross (Docker-based)

Uses pre-configured Docker containers with all necessary toolchains.

### Installation

```bash
cargo install cross
```

### Usage

```bash
# Replace cargo with cross
cross build --target aarch64-unknown-linux-gnu --release
cross test --target x86_64-unknown-linux-musl
```

### Custom Configuration

```toml
# Cross.toml
[target.aarch64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main"

[build.env]
passthrough = ["RUST_BACKTRACE", "MY_ENV_VAR"]
```

## Static Linking with musl

Creates fully portable Linux binaries with no runtime dependencies.

```bash
rustup target add x86_64-unknown-linux-musl
cargo zigbuild --target x86_64-unknown-linux-musl --release
```

**Trade-offs:**
- Larger binary size
- Some async I/O performance impact
- No glibc-specific features (e.g., certain DNS resolution)

## Platform-Specific Notes

### macOS Cross-Compilation

Building for macOS from non-Apple hardware is legally complex due to SDK licensing. Use macOS CI runners for production builds.

### Windows Targets

From Linux/macOS, prefer `-gnu` over `-msvc`:

```bash
rustup target add x86_64-pc-windows-gnu
cargo zigbuild --target x86_64-pc-windows-gnu --release
```

### WebAssembly

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release

# For maximum compatibility
RUSTFLAGS=-Ctarget-cpu=mvp cargo +nightly build \
  -Zbuild-std=panic_abort,std \
  --target wasm32-unknown-unknown
```

## Cargo Configuration

```toml
# .cargo/config.toml

# Per-target linker settings
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[target.x86_64-unknown-linux-musl]
linker = "rust-lld"

# Environment variables for all targets
[env]
RUST_BACKTRACE = "1"
```

## Target Tier System

| Tier | Guarantees | Examples |
|------|------------|----------|
| **Tier 1** | Builds, tested upstream | x86_64-unknown-linux-gnu |
| **Tier 2** | Builds, not fully tested | aarch64-unknown-linux-gnu |
| **Tier 3** | No guarantees | Many embedded targets |

For Tier 3, you may need to build the standard library:

```bash
cargo +nightly build -Zbuild-std=std,panic_abort --target <tier3-target>
```

## Related

- [Container Builds](./container-builds.md)
- [CI/CD Workflows](./cicd-workflows.md)
