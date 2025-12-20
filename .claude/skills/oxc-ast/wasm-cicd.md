# WASM CI/CD Pipeline

Automate WASM compilation and NPM publishing with GitHub Actions.

## GitHub Actions Workflow

**.github/workflows/publish.yml:**

```yaml
name: Publish WASM to NPM

on:
  push:
    tags:
      - 'v*' # Triggers on version tags like v1.0.0

jobs:
  build-and-publish:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Build WASM Package
        run: wasm-pack build crates/wasm --release --target nodejs --out-dir ../../pkg

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'

      - name: Publish to NPM
        run: |
          cd pkg
          npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

## Multi-Target Build

Build for Node.js, Web, and Bundlers in parallel:

```yaml
jobs:
  build-wasm:
    strategy:
      matrix:
        target: [nodejs, web, bundler]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Build WASM
        run: wasm-pack build crates/wasm --release --target ${{ matrix.target }} --out-dir ../../pkg-${{ matrix.target }}

      - name: Upload Artifact
        uses: actions/upload-artifact@v3
        with:
          name: wasm-${{ matrix.target }}
          path: pkg-${{ matrix.target }}/
```

## Testing Workflow

**.github/workflows/test.yml:**

```yaml
name: Test

on: [push, pull_request]

jobs:
  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run Tests
        run: cargo test --workspace

  test-wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Test WASM
        run: wasm-pack test crates/wasm --node
```

## Bundle Size Tracking

Track WASM bundle size over time:

```yaml
jobs:
  bundle-size:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build WASM
        run: wasm-pack build crates/wasm --release --target web

      - name: Check Bundle Size
        run: |
          WASM_SIZE=$(stat -f%z pkg/oxc_wasm_tools_bg.wasm)
          echo "WASM bundle size: $WASM_SIZE bytes"

          if [ $WASM_SIZE -gt 1048576 ]; then
            echo "❌ WASM bundle too large (> 1MB)"
            exit 1
          fi

          echo "✅ WASM bundle size OK"
```

## Automated Versioning

Automatically bump version and create release:

```yaml
name: Release

on:
  push:
    branches:
      - main

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Node.js
        uses: actions/setup-node@v4

      - name: Get Next Version
        id: version
        run: |
          # Calculate next semantic version
          LATEST=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
          echo "Latest: $LATEST"

          # Bump patch version
          NEXT=$(echo $LATEST | awk -F. '{$NF = $NF + 1;} 1' | sed 's/ /./g')
          echo "Next: $NEXT"
          echo "version=$NEXT" >> $GITHUB_OUTPUT

      - name: Update Cargo.toml
        run: |
          VERSION=${{ steps.version.outputs.version }}
          VERSION=${VERSION#v} # Remove 'v' prefix
          sed -i "s/^version = .*/version = \"$VERSION\"/" crates/wasm/Cargo.toml

      - name: Create Tag
        run: |
          git config user.name "github-actions"
          git config user.email "github-actions@github.com"
          git add .
          git commit -m "chore: bump version to ${{ steps.version.outputs.version }}"
          git tag ${{ steps.version.outputs.version }}
          git push origin main --tags
```

## Production Checklist

### 1. UTF-8 to UTF-16 Span Conversion

JavaScript uses UTF-16. OXC uses UTF-8 byte offsets.

```rust
pub fn get_line_column_offset(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;

    for (i, c) in source.char_indices() {
        if i >= byte_offset { break; }

        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}
```

### 2. Bundle Size Optimization

**In Cargo.toml:**

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

**Use wee_alloc:**

```rust
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

### 3. TypeScript Definitions

Create `manual.d.ts` for better type safety:

```typescript
export interface RenameEdit {
    start: number;
    end: number;
    replacement: string;
}

export interface AnalysisResult {
    interfaces: string[];
    total_symbols: number;
}

export function get_rename_edits(
    source: string,
    target_name: string,
    new_name: string
): RenameEdit[];

export function analyze_code(source: string): AnalysisResult;
```

## NPM Publishing

**package.json:**

```json
{
  "name": "oxc-analyzer-wasm",
  "version": "0.1.0",
  "description": "High-performance TypeScript/JavaScript analyzer compiled to WASM",
  "main": "pkg/oxc_wasm_tools.js",
  "types": "pkg/oxc_wasm_tools.d.ts",
  "files": [
    "pkg/",
    "README.md",
    "LICENSE"
  ],
  "keywords": [
    "ast",
    "typescript",
    "javascript",
    "parser",
    "analyzer",
    "wasm"
  ],
  "repository": {
    "type": "git",
    "url": "https://github.com/user/oxc-analyzer"
  },
  "license": "MIT"
}
```

## Continuous Deployment

```yaml
name: CD

on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Dependencies
        run: |
          curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Build for Multiple Targets
        run: |
          wasm-pack build crates/wasm --release --target nodejs --out-dir ../../pkg-nodejs
          wasm-pack build crates/wasm --release --target web --out-dir ../../pkg-web
          wasm-pack build crates/wasm --release --target bundler --out-dir ../../pkg-bundler

      - name: Publish to NPM
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
        run: |
          # Publish Node.js version
          cd pkg-nodejs
          npm publish
```

## Monitoring and Alerts

**Send Slack notification on publish:**

```yaml
      - name: Notify Slack
        if: success()
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "✅ WASM package published: ${{ github.ref_name }}"
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```

## Security Scanning

```yaml
jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run cargo-audit
        run: |
          cargo install cargo-audit
          cargo audit
```

## Performance Benchmarking

```yaml
jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build WASM
        run: wasm-pack build --release

      - name: Run Benchmarks
        run: |
          # Compare bundle size against baseline
          BASELINE_SIZE=512000  # 500KB
          CURRENT_SIZE=$(stat -f%z pkg/oxc_wasm_tools_bg.wasm)

          echo "Baseline: $BASELINE_SIZE bytes"
          echo "Current: $CURRENT_SIZE bytes"

          if [ $CURRENT_SIZE -gt $BASELINE_SIZE ]; then
            DIFF=$((CURRENT_SIZE - BASELINE_SIZE))
            echo "⚠️  Bundle grew by $DIFF bytes"
          else
            echo "✅ Bundle size within limits"
          fi
```

## Complete Production Workflow

```yaml
name: Production

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      - run: wasm-pack build --release --target nodejs

  publish:
    needs: build
    if: startsWith(github.ref, 'refs/tags/v')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      - run: wasm-pack build --release --target nodejs
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      - run: cd pkg && npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

## Summary

This CI/CD pipeline:

1. **Tests** Rust code and WASM bindings
2. **Builds** for multiple targets (Node, Web, Bundler)
3. **Optimizes** bundle size
4. **Publishes** to NPM automatically on tags
5. **Monitors** bundle size and security
6. **Notifies** team on successful deploys
