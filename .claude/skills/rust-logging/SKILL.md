---
name: rust-logging
description: Expert knowledge for logging and tracing in Rust applications using tracing, log, env_logger, fern, and OpenTelemetry. Use when adding observability, debugging async code, or setting up structured logging.
hash: 7ff1be522ade8935
---

# Rust Logging and Tracing

Instrumentation for Rust applications: from simple `log` facade to structured `tracing` with OpenTelemetry.

## Core Principles

- **New async apps**: Use `tracing` + `tracing-subscriber` - spans propagate context across `.await` points
- **Libraries**: Use `log` facade only - let consumers choose the implementation
- **Simple CLIs**: `log` + `env_logger` is sufficient
- **Production services**: `tracing` with `EnvFilter` + file appender + OpenTelemetry layer
- **Bridge ecosystems**: Use `tracing-log` to capture `log` records as `tracing` events
- **Filter at runtime**: Use `RUST_LOG` env var (e.g., `RUST_LOG=info,my_crate=debug`)
- **Structured data**: Prefer key-value fields over formatted strings for machine parsing
- **Async spans**: Use `.instrument(span)` on futures, never hold span guards across `.await`

## Quick Reference

### Choosing Your Stack

| Scenario | Recommended Stack |
|----------|-------------------|
| Library crate | `log` only |
| CLI tool | `log` + `env_logger` |
| Sync web service | `tracing` + `tracing-subscriber` |
| Async service (Tokio/Axum) | `tracing` + `tracing-subscriber` + `tracing-log` |
| Production with distributed tracing | Above + `tracing-opentelemetry` |

### Minimal `tracing` Setup

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

```rust
use tracing::{info, instrument};
use tracing_subscriber::{fmt, EnvFilter};

#[instrument]
async fn handle_request(user_id: u64) {
    info!("Processing request");
}

fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();
}
```

### Minimal `log` Setup

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

```rust
use log::{info, debug, error};

fn main() {
    env_logger::init();
    info!("Application started");
}
```

## Topics

### Tracing Ecosystem

- [tracing-guide.md](./tracing-guide.md) - Spans, events, subscribers, and async patterns
- [tracing-production.md](./tracing-production.md) - File logging, OpenTelemetry, and layer composition

### Log Ecosystem

- [log-implementations.md](./log-implementations.md) - env_logger, fern, flexi_logger, log4rs comparison

## Common Patterns

### Per-Request Tracing (Axum)

```rust
use tracing::{info_span, Instrument};

async fn handle(req: Request) -> Response {
    let span = info_span!("request", method = %req.method(), path = %req.uri());
    async move {
        // All events here are within the request span
        tracing::info!("handling request");
        // ...
    }
    .instrument(span)
    .await
}
```

### Structured Fields

```rust
// Use % for Display, ? for Debug
tracing::error!(
    error = ?e,           // Debug format
    user_id = %user.id,   // Display format
    "Request failed"
);
```

### Bridge `log` to `tracing`

```rust
use tracing_log::LogTracer;

fn main() {
    LogTracer::init().unwrap();
    tracing_subscriber::fmt::init();
    // Now log::info! and tracing::info! both work
}
```

## Resources

- [tracing crate docs](https://docs.rs/tracing)
- [tracing-subscriber docs](https://docs.rs/tracing-subscriber)
- [log crate docs](https://docs.rs/log)
- [Tokio tracing guide](https://tokio.rs/tokio/topics/tracing)
