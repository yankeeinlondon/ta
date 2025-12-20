---
name: rust-logging
description: Comprehensive guide to logging and tracing in Rust applications
created: 2025-12-18
hash: 16113f0931a2076d
tags:
  - rust
  - logging
  - tracing
  - observability
  - diagnostics
---

# Rust Logging and Tracing

Effective instrumentation is fundamental for debugging, performance monitoring, and maintaining reliable software. The Rust ecosystem provides a rich landscape of crates for logging and tracing, each with distinct philosophies and trade-offs. This guide covers the foundational `log` facade, the modern `tracing` framework, and alternative solutions to help you choose the right approach for your project.

## Table of Contents

- [Mental Model: Logging vs Tracing](#mental-model-logging-vs-tracing)
- [The log Crate: The Standard Facade](#the-log-crate-the-standard-facade)
- [log Implementations](#log-implementations)
- [The tracing Framework](#the-tracing-framework)
- [tracing-subscriber: Collecting Trace Data](#tracing-subscriber-collecting-trace-data)
- [Bridging log and tracing](#bridging-log-and-tracing)
- [Alternative Solutions: fern and slog](#alternative-solutions-fern-and-slog)
- [OpenTelemetry Integration](#opentelemetry-integration)
- [Choosing the Right Tool](#choosing-the-right-tool)
- [Best Practices](#best-practices)
- [Quick Reference](#quick-reference)
- [Resources](#resources)

## Mental Model: Logging vs Tracing

Understanding the distinction between traditional logging and modern tracing is essential:

| Aspect | `log` Ecosystem | `tracing` Ecosystem |
|--------|-----------------|---------------------|
| **Design** | Simple logging facade with discrete events | Structured, span-based diagnostics |
| **Data Model** | Log events with level, target, and message | Spans (time periods) and events (moments) with typed key-value fields |
| **Async Support** | Not inherently async-aware | Built for async; spans handle `await` points correctly |
| **Context** | Manual context passing | Automatic context propagation through spans |
| **Best For** | Libraries, CLIs, synchronous apps | Async services, microservices, complex systems |

**Recommendation**: For new async applications (Tokio, Axum), start with `tracing`. For simple CLIs or libraries needing broad compatibility, use `log`.

## The log Crate: The Standard Facade

The `log` crate is a lightweight logging facade that decouples logging calls from their implementation. Libraries use `log` macros; applications choose an implementation.

### Log Levels

From highest to lowest severity:

1. `error!` - Operation failed, requires attention
2. `warn!` - Unexpected event, but system continues
3. `info!` - Normal system events (e.g., "service started")
4. `debug!` - Detailed information for debugging
5. `trace!` - Very verbose, low-level details

### Basic Usage

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

```rust
use log::{debug, error, info, trace, warn};

fn main() {
    // Initialize logger early - without this, logs are silently ignored
    env_logger::init();

    trace!("Very detailed trace information");
    debug!("Debugging information");
    info!("Application started");
    warn!("Something potentially problematic");
    error!("Operation failed: {}", "connection timeout");
}
```

### Structured Logging with Key-Value Pairs

The `log` crate supports structured data (unstable `kv` feature):

```rust
use log::info;

fn handle_request(user_id: u64, action: &str) {
    info!(
        target: "request_handler",
        user_id,
        action,
        status = "started";
        "Handling user request"
    );
}
```

### Using Targets for Filtering

```rust
// Library code - use a custom target for easy filtering
pub fn do_something() {
    log::info!(target: "my_lib", "Performing operation");
}
```

Filter with: `RUST_LOG="my_lib=debug,warn"`

## log Implementations

### env_logger: Simple and Environment-Driven

The most common choice for simple applications. Configured via `RUST_LOG` environment variable.

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

```rust
fn main() {
    // Reads RUST_LOG env var; defaults to error level
    env_logger::init();

    log::info!("Application started");
}
```

**RUST_LOG Examples:**

```bash
RUST_LOG=info cargo run                          # Global info level
RUST_LOG=debug cargo run                         # Global debug level
RUST_LOG=my_crate=debug,warn cargo run           # Debug for my_crate, warn for others
RUST_LOG="my_crate::db=trace,info" cargo run     # Trace for db module, info globally
```

**Pros:**
- Dead simple, no config files
- Environment variable configuration fits 12-factor apps
- Perfect for CLIs and development

**Cons:**
- Limited formatting options
- No file output or rotation
- Not async-aware

### flexi_logger: Flexible with File Rotation

For applications needing file logging with rotation and runtime reconfiguration:

```toml
[dependencies]
log = "0.4"
flexi_logger = "0.29"
```

```rust
use flexi_logger::{Cleanup, Criterion, Duplicate, Logger, Naming};

fn main() {
    Logger::try_with_env_or_str("info, my_crate::db=debug")
        .unwrap()
        .log_to_file()
        .duplicate_to_stderr(Duplicate::Info)
        .rotate(
            Criterion::Size(10_000_000),  // 10MB
            Naming::Numbers,
            Cleanup::KeepLogFiles(7),
        )
        .start()
        .unwrap();

    log::info!("Application with file rotation started");
}
```

### log4rs: Configuration-File Driven

For enterprise-style configuration via YAML/TOML:

```yaml
# log4rs.yaml
refresh_rate: 30 seconds

appenders:
  stdout:
    kind: console
    encoder:
      pattern: "{d} {l} {t} - {m}{n}"

  rolling_file:
    kind: rolling_file
    path: "logs/app.log"
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 10 mb
      roller:
        kind: fixed_window
        base: 1
        count: 5
        pattern: "logs/app.{}.log.gz"

root:
  level: info
  appenders:
    - stdout
    - rolling_file
```

```rust
fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::info!("Application with log4rs started");
}
```

## The tracing Framework

`tracing` is a framework for structured, context-aware diagnostics designed for async applications. It introduces spans (time periods) and events (moments) with typed key-value fields.

### Core Concepts

| Concept | Description | Analogy |
|---------|-------------|---------|
| **Span** | A period of time with beginning and end | A chapter in a book |
| **Event** | A moment in time within a span | A sentence within that chapter |
| **Subscriber** | Collects and processes spans/events | The publisher that prints the book |
| **Layer** | Composable component for processing | A notebook section |

### Basic Setup

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

```rust
use tracing::{info, instrument, span, Level};
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    // Initialize subscriber with env filter
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Application started");
}
```

### The #[instrument] Attribute

The easiest way to add tracing - automatically creates a span for a function:

```rust
use tracing::{info, instrument};

#[instrument]  // Creates span named "process_order" with order_id field
async fn process_order(order_id: u64) -> Result<(), Error> {
    info!("Starting order processing");

    // Events within this span inherit its context
    fetch_inventory(order_id).await?;

    info!("Order processing complete");
    Ok(())
}

#[instrument(skip(db_pool))]  // Skip non-Debug fields
async fn create_user(
    db_pool: &PgPool,
    username: String,
) -> Result<User, Error> {
    info!("Creating user");
    // ...
}
```

### Manual Span Creation

For more control over span lifecycle:

```rust
use tracing::{info, span, Level, Instrument};

fn process_data(data: &[u8]) {
    // Create and enter a span
    let span = span!(Level::DEBUG, "process_data", size = data.len());
    let _enter = span.enter();

    info!("Data received, beginning processing");
    // Span exits when _enter is dropped
}

// For async code, use .instrument()
async fn background_job(job_id: u32) {
    let span = tracing::info_span!("background_job", job_id);

    async move {
        tracing::info!("Executing job");
        // ...
    }
    .instrument(span)
    .await
}
```

### Structured Fields

Record typed data with events and spans:

```rust
use tracing::{event, Level};

fn handle_request(request_url: &str, user_id: u64) {
    event!(
        Level::INFO,
        url = %request_url,      // % = Display format
        user_id,                  // Field name from variable
        status = "processing",
        "Handling request"
    );
}

// On errors
fn handle_error(e: &Error) {
    event!(
        Level::ERROR,
        error = ?e,              // ? = Debug format
        "Request failed"
    );
}
```

### Async Context: Avoiding Common Pitfalls

Do NOT hold span guards across `.await` points:

```rust
// BAD - guard held across await
async fn bad_example() {
    let span = info_span!("my_span");
    let _enter = span.enter();  // Danger!
    some_async_function().await;  // Span context may be incorrect
}

// GOOD - use in_scope for sync work, then drop before await
async fn good_example() {
    let span = info_span!("my_span");
    let result = span.in_scope(|| {
        some_sync_work()
    });
    // Span exited before await
    some_async_function(result).await;
}

// GOOD - use .instrument() for async
async fn best_example() {
    some_async_function()
        .instrument(info_span!("my_span"))
        .await;
}
```

## tracing-subscriber: Collecting Trace Data

`tracing-subscriber` provides utilities for building subscribers with composable layers.

### Registry and Layers

```rust
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    fmt,
};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().pretty())
        .init();
}
```

### Formatting Options

```rust
tracing_subscriber::fmt()
    .json()                    // JSON output for machines
    .with_target(false)        // Hide module path
    .with_thread_ids(true)     // Show thread ID
    .with_file(true)           // Show source file
    .with_line_number(true)    // Show line number
    .init();
```

### File Logging with tracing-appender

```toml
[dependencies]
tracing-appender = "0.2"
```

```rust
use tracing_appender::rolling;
use tracing_subscriber::{layer::SubscriberExt, Registry};

fn init_with_file_logging() {
    let file_appender = rolling::daily("/var/log/myapp", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // IMPORTANT: _guard must be kept alive for the duration of your program
}
```

### Multiple Output Destinations

```rust
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_tree::HierarchicalLayer;

fn init_multi_output() {
    let file_appender = tracing_appender::rolling::daily("/var/log", "api.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    Registry::default()
        .with(HierarchicalLayer::new(2))  // Tree view for debugging
        .with(tracing_subscriber::fmt::layer().pretty())  // Pretty console
        .with(tracing_subscriber::fmt::layer()
            .json()
            .with_writer(non_blocking))  // JSON to file
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
```

## Bridging log and tracing

Use `tracing-log` to capture `log` records as `tracing` events:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-log = "0.2"
log = "0.4"
```

```rust
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt};

fn main() {
    // Forward log records into tracing
    LogTracer::init().unwrap();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer())
        .init();

    // Both work now
    log::info!("From log crate");
    tracing::info!("From tracing crate");
}
```

This is invaluable when:
- Migrating gradually from `log` to `tracing`
- Using dependencies that only support `log`
- Wanting unified diagnostic output

## Alternative Solutions: fern and slog

### fern: Configurable Builder API

A middle ground between `env_logger` simplicity and full `tracing` power:

```toml
[dependencies]
log = "0.4"
fern = "0.7"
chrono = "0.4"
```

```rust
use chrono::Local;
use log::LevelFilter;

fn setup_fern() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::Info)
        .level_for("noisy_crate", LevelFilter::Warn)
        .chain(std::io::stderr())
        .chain(fern::log_file("app.log")?)
        .apply()?;
    Ok(())
}
```

**Pros:** Good balance of simplicity and configurability, multi-target output
**Cons:** No span-based context, not async-aware

### slog: Structured Logging Ecosystem

A comprehensive ecosystem emphasizing structured, contextual logging:

```toml
[dependencies]
slog = "2.7"
slog-term = "2.9"
slog-async = "2.7"
```

```rust
use slog::{info, o, Drain, Logger};

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = Logger::root(drain, o!(
        "version" => env!("CARGO_PKG_VERSION"),
        "app" => "my_service"
    ));

    info!(log, "Service starting"; "port" => 8080);

    // Child logger with additional context
    let db_log = log.new(o!("component" => "database"));
    info!(db_log, "Connecting"; "host" => "localhost");
}
```

**Pros:** Strong structured logging, composable drains, contextual data
**Cons:** Steeper learning curve, less community momentum than `tracing`

## OpenTelemetry Integration

For distributed tracing with backends like Jaeger, Tempo, or Honeycomb:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-opentelemetry = "0.28"
opentelemetry = { version = "0.24", features = ["trace"] }
opentelemetry-otlp = "0.17"  # For OTLP export
```

```rust
use opentelemetry::global;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, Registry};

fn init_otel() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry::runtime::Tokio)?;

    let otel_layer = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

// Shutdown on exit
fn shutdown() {
    global::shutdown_tracer_provider();
}
```

## Choosing the Right Tool

| Use Case | Recommended Stack |
|----------|-------------------|
| **Simple CLI tool** | `log` + `env_logger` |
| **CLI needing file output** | `log` + `fern` |
| **Library crate** | `log` only (let consumers choose) |
| **Sync web service** | `tracing` + `tracing-subscriber` |
| **Async service (Tokio/Axum)** | `tracing` + `tracing-subscriber` + `tracing-log` |
| **Production microservice** | Above + `tracing-opentelemetry` |
| **Enterprise with config files** | `log` + `log4rs` |
| **Existing slog codebase** | Continue with `slog` unless refactoring |

### Decision Flowchart

1. **Is it a library?** Use `log` for maximum compatibility
2. **Is it async?** Use `tracing` - the context propagation is invaluable
3. **Do you need distributed tracing?** Add `tracing-opentelemetry`
4. **Is it a simple CLI?** `log` + `env_logger` is sufficient
5. **Need file rotation?** Consider `flexi_logger` or `tracing-appender`

## Best Practices

### Log Level Guidelines

- **ERROR**: Operation failed, may require immediate attention
- **WARN**: Unexpected event, but operation continues
- **INFO**: Normal events worth recording (startup, requests)
- **DEBUG**: Detailed info for development debugging
- **TRACE**: Very verbose, every function entry/exit

### Security

Never log sensitive data:

```rust
// BAD
info!("User logged in: {}", password);

// GOOD
info!(user_id = %user.id, "User logged in");
```

### Avoid Side Effects

Don't put code with side effects in log macros:

```rust
// BAD - side effect may not execute if level is filtered
info!("Count: {}", counter.fetch_add(1, Ordering::SeqCst));

// GOOD
let count = counter.fetch_add(1, Ordering::SeqCst);
info!("Count: {}", count);
```

### Production Configuration

```rust
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt};

fn init_production_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Default: info for our code, warn for dependencies
            EnvFilter::new("warn,my_crate=info")
        });

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer()
            .json()                    // Machine-readable
            .with_current_span(true)   // Include span context
            .with_span_list(true))     // Include span hierarchy
        .init();
}
```

## Quick Reference

### RUST_LOG Syntax

```bash
# Global level
RUST_LOG=info

# Per-crate levels
RUST_LOG=my_crate=debug,hyper=warn

# Per-module levels
RUST_LOG=my_crate::db=trace,my_crate::api=debug

# With span filtering (tracing)
RUST_LOG="my_crate[request]=debug"
```

### Common Crate Combinations

```toml
# Simple CLI
log = "0.4"
env_logger = "0.11"

# Async service
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
tracing-log = "0.2"  # For log compatibility

# With OpenTelemetry
tracing-opentelemetry = "0.28"
opentelemetry = { version = "0.24", features = ["trace"] }
opentelemetry-otlp = "0.17"
```

### Ecosystem at a Glance

| Crate | Role |
|-------|------|
| `log` | Logging facade |
| `env_logger` | Simple log implementation |
| `fern` | Configurable log implementation |
| `flexi_logger` | File rotation, runtime config |
| `log4rs` | Enterprise-style config files |
| `tracing` | Structured diagnostics framework |
| `tracing-subscriber` | Subscriber/layer utilities |
| `tracing-appender` | File appenders for tracing |
| `tracing-log` | Bridge log to tracing |
| `tracing-opentelemetry` | OpenTelemetry integration |
| `slog` | Alternative structured logging |

## Resources

- [log crate documentation](https://docs.rs/log)
- [tracing crate documentation](https://docs.rs/tracing)
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber)
- [tokio-rs/tracing GitHub repository](https://github.com/tokio-rs/tracing)
- [env_logger documentation](https://docs.rs/env_logger)
- [fern documentation](https://docs.rs/fern)
- [slog documentation](https://docs.rs/slog)
- [OpenTelemetry Rust](https://opentelemetry.io/docs/instrumentation/rust/)
