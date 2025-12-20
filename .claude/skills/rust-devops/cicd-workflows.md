# CI/CD Workflows

GitHub Actions templates and automation patterns for Rust projects.

## Basic CI Workflow

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Test
        run: cargo test --all-features

      - name: Build
        run: cargo build --release
```

## Cross-Platform Release Build

Matrix build for multiple targets with artifact upload.

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            artifact: myapp-linux-amd64
          - target: aarch64-unknown-linux-musl
            os: ubuntu-latest
            artifact: myapp-linux-arm64
          - target: x86_64-apple-darwin
            os: macos-latest
            artifact: myapp-macos-amd64
          - target: aarch64-apple-darwin
            os: macos-latest
            artifact: myapp-macos-arm64
          - target: x86_64-pc-windows-gnu
            os: ubuntu-latest
            artifact: myapp-windows-amd64.exe

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cargo-zigbuild
        if: matrix.os == 'ubuntu-latest'
        run: pip install cargo-zigbuild

      - name: Build (zigbuild)
        if: matrix.os == 'ubuntu-latest'
        run: cargo zigbuild --release --target ${{ matrix.target }}

      - name: Build (native)
        if: matrix.os != 'ubuntu-latest'
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: target/${{ matrix.target }}/release/myapp*

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Download artifacts
        uses: actions/download-artifact@v4

      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            myapp-linux-amd64/myapp
            myapp-linux-arm64/myapp
            myapp-macos-amd64/myapp
            myapp-macos-arm64/myapp
            myapp-windows-amd64.exe/myapp.exe
```

## Docker Build and Push

```yaml
name: Docker

on:
  push:
    branches: [main]
    tags: ['v*']

jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: true
          tags: |
            ghcr.io/${{ github.repository }}:latest
            ghcr.io/${{ github.repository }}:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

## Security Scanning

```yaml
name: Security

on:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0'  # Weekly

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Security audit
        run: cargo audit

      - name: Check duplicates
        run: cargo tree --duplicates
```

## Caching Strategies

### Swatinem/rust-cache (Recommended)

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    cache-on-failure: true
    shared-key: "build"
```

### Manual Caching

```yaml
- name: Cache cargo registry
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: ${{ runner.os }}-cargo-
```

## Quality Gates

### Code Coverage with Tarpaulin

```yaml
coverage:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin

    - name: Generate coverage
      run: cargo tarpaulin --out xml

    - name: Upload to Codecov
      uses: codecov/codecov-action@v4
```

### Benchmarks with Criterion

```yaml
benchmark:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Run benchmarks
      run: cargo bench --all-features

    - name: Store benchmark result
      uses: benchmark-action/github-action-benchmark@v1
      with:
        tool: 'cargo'
        output-file-path: target/criterion/*/new/estimates.json
```

## Toolchain Pinning

### rust-toolchain.toml

```toml
[toolchain]
channel = "1.75.0"
components = ["clippy", "rustfmt"]
targets = ["x86_64-unknown-linux-musl"]
```

### In Workflow

```yaml
- name: Install Rust
  uses: dtolnay/rust-toolchain@master
  with:
    toolchain: "1.75.0"
```

## Best Practices

1. **Use rust-cache** - Dramatically speeds up builds
2. **Pin toolchain version** - Ensures reproducible builds
3. **Run clippy with `-D warnings`** - Treat warnings as errors
4. **Test with `--all-features`** - Catch feature flag issues
5. **Use matrix builds** - Parallel cross-platform builds
6. **Cache Docker layers** - Use `cache-from/cache-to` with buildx
7. **Security scan regularly** - Weekly cargo-audit at minimum

## Related

- [Cross-Platform Builds](./cross-platform-builds.md)
- [Cloud Deployment](./cloud-deployment.md)
