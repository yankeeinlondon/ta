# Tracing Guide

The `tracing` crate is a framework for structured, async-aware diagnostics in Rust. It captures context and hierarchy of events, essential for debugging async applications.

## Core Concepts

| Concept | Description | Analogy |
|---------|-------------|---------|
| **Span** | A period of time with a beginning and end (e.g., handling a request) | A chapter in a book |
| **Event** | A moment in time within a span (e.g., "cache miss") | A sentence in that chapter |
| **Subscriber** | Collects and processes spans/events (e.g., prints to console) | The publisher |
| **Layer** | A composable component in a subscriber (e.g., filter, formatter) | A chapter section |

## Basic Setup

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

```rust
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer())
        .init();
}
```

## Instrumentation

### The `#[instrument]` Attribute

The easiest way to add tracing - automatically creates a span for the function:

```rust
use tracing::{info, instrument};

#[instrument]
fn process_order(order_id: u64) {
    info!("Processing order");
}

// Skip sensitive fields
#[instrument(skip(password))]
fn login(username: &str, password: &str) {
    info!("User logging in");
}

// Rename fields or the span
#[instrument(name = "db_query", fields(table = "users"))]
fn query_users() {
    info!("Querying database");
}
```

### Manual Spans

For more control over span lifecycle:

```rust
use tracing::{span, Level, info};

fn process_data(data: &[u8]) {
    let span = span!(Level::INFO, "process_data", size = data.len());
    let _guard = span.enter();

    info!("Processing {} bytes", data.len());
    // span exits when _guard is dropped
}
```

### Events

Record moments in time with structured fields:

```rust
use tracing::{event, info, warn, error, Level};

// Level-specific macros
info!("Application started");
warn!(retry_count = 3, "Connection unstable");
error!(error = ?e, "Failed to connect");

// Generic event! macro
event!(Level::DEBUG, answer = 42, "Computed result");

// Field formatting
// % = Display trait
// ? = Debug trait
info!(user_id = %id, details = ?user, "User created");
```

## Async Patterns

### CRITICAL: Span Guards and `.await`

Never hold a span guard across an `.await` point:

```rust
// WRONG - guard held across await
async fn bad() {
    let span = info_span!("my_span");
    let _guard = span.enter();
    some_async_fn().await; // BUG: incorrect trace context
}

// CORRECT - use .instrument()
async fn good() {
    let span = info_span!("my_span");
    some_async_fn().instrument(span).await;
}

// CORRECT - use in_scope for sync work before await
async fn also_good() {
    let span = info_span!("my_span");
    let result = span.in_scope(|| sync_work());
    some_async_fn().await;
}
```

### Instrumenting Async Functions

```rust
use tracing::{info_span, Instrument};

// Option 1: #[instrument] attribute (recommended)
#[tracing::instrument]
async fn fetch_user(user_id: u64) -> User {
    // Automatically creates span with user_id field
    tracing::info!("Fetching user");
    db.get_user(user_id).await
}

// Option 2: Manual .instrument()
async fn process_request(req_id: u64) {
    let span = info_span!("request", %req_id);
    async move {
        tracing::info!("Processing");
        // ...
    }
    .instrument(span)
    .await
}
```

## Filtering with EnvFilter

Control verbosity via `RUST_LOG` environment variable:

```bash
# Global level
RUST_LOG=info cargo run

# Per-crate levels
RUST_LOG=warn,my_crate=debug cargo run

# Per-module levels
RUST_LOG=my_crate::db=trace,my_crate::web=info cargo run

# With span filtering
RUST_LOG="my_crate[request]=debug" cargo run
```

Programmatic filtering:

```rust
use tracing_subscriber::EnvFilter;

let filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("info,my_crate=debug"));
```

## Layer Composition

Build complex subscribers by combining layers:

```rust
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    // Filter layer
    .with(EnvFilter::from_default_env())
    // Console output layer
    .with(fmt::layer().pretty())
    // JSON output layer (e.g., for files)
    .with(fmt::layer().json().with_writer(file_writer))
    .init();
```

## Key Crates

| Crate | Purpose |
|-------|---------|
| `tracing` | Core API - spans, events, macros |
| `tracing-subscriber` | Subscriber implementations, layers, EnvFilter |
| `tracing-appender` | Non-blocking file writers, log rotation |
| `tracing-log` | Bridge `log` records into `tracing` |
| `tracing-opentelemetry` | Export to OpenTelemetry collectors |
| `tower-http` (trace feature) | HTTP request tracing for Axum/Tower |
| `tracing-error` | Error context and SpanTrace |

## Common Gotchas

1. **Forgetting to initialize subscriber**: No output without `tracing_subscriber::fmt::init()` or similar
2. **Span guards across await**: Use `.instrument()` instead
3. **Blocking in subscriber**: Use non-blocking writers for production
4. **Over-instrumenting**: `#[instrument]` on hot paths adds overhead - profile first
5. **Missing env-filter feature**: Add `features = ["env-filter"]` to `tracing-subscriber`
