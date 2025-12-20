# Tracing for Production

Advanced configuration for production Rust services: file logging, rotation, OpenTelemetry integration, and best practices.

## File Logging with Rotation

Use `tracing-appender` for non-blocking file output with rotation:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
```

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    // Create rolling file appender
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("app")
        .filename_suffix("log")
        .build("/var/log/myapp")
        .expect("Failed to create appender");

    // Make it non-blocking
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        // Console: human-readable, colored
        .with(fmt::layer().pretty())
        // File: JSON for machine parsing
        .with(fmt::layer().json().with_writer(non_blocking))
        .init();

    guard // MUST keep alive for duration of program
}

fn main() {
    let _guard = init_logging();
    // ...
}
```

### Rotation Options

```rust
use tracing_appender::rolling::Rotation;

Rotation::MINUTELY  // New file every minute
Rotation::HOURLY    // New file every hour
Rotation::DAILY     // New file every day
Rotation::NEVER     // Single file, no rotation
```

## OpenTelemetry Integration

Export traces to Jaeger, Tempo, or any OTLP-compatible backend:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-opentelemetry = "0.28"
opentelemetry = { version = "0.27", features = ["trace"] }
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
```

```rust
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace::TracerProvider};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

async fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    // Set up OTLP exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    // Create tracer provider
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .build();

    let tracer = provider.tracer("my-service");
    global::set_tracer_provider(provider);

    // Create OpenTelemetry layer
    let otel_layer = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .init();

    Ok(())
}

// On shutdown
fn shutdown() {
    global::shutdown_tracer_provider();
}
```

## Multi-Layer Production Setup

Complete example with console, file, and OpenTelemetry:

```rust
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_production_logging() {
    // File appender for JSON logs
    let file_appender = tracing_appender::rolling::daily("/var/log/app", "app.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        // Filtering (respects RUST_LOG)
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            // Production defaults
            "warn,my_app=info,tower_http=debug".into()
        }))
        // Console: compact for humans
        .with(fmt::layer()
            .compact()
            .with_target(true)
            .with_thread_ids(false))
        // File: JSON for log aggregation
        .with(fmt::layer()
            .json()
            .with_writer(file_writer)
            .with_span_list(true))
        .init();
}
```

## Axum/Tower HTTP Tracing

Automatic per-request tracing with `tower-http`:

```toml
[dependencies]
tower-http = { version = "0.6", features = ["trace"] }
```

```rust
use axum::{Router, routing::get};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnResponse};
use tracing::Level;

let app = Router::new()
    .route("/", get(handler))
    .layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new()
                .level(Level::INFO)
                .include_headers(true))
            .on_response(DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(tower_http::LatencyUnit::Micros))
    );
```

Custom span with request ID:

```rust
use axum::{extract::Request, middleware::Next, response::Response};
use tracing::{info_span, Instrument};
use uuid::Uuid;

async fn trace_middleware(req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "request",
        method = %req.method(),
        path = %req.uri().path(),
        request_id = %request_id
    );

    next.run(req).instrument(span).await
}
```

## Production Best Practices

### 1. Structured Fields Over Formatted Strings

```rust
// Good - structured, searchable
tracing::info!(user_id = %id, action = "login", "User authenticated");

// Avoid - harder to parse
tracing::info!("User {} performed action: login", id);
```

### 2. Appropriate Log Levels

| Level | Use For |
|-------|---------|
| ERROR | Operation failed, may need intervention |
| WARN | Unexpected but recoverable, degraded state |
| INFO | Normal milestones (startup, request handled) |
| DEBUG | Detailed flow for development debugging |
| TRACE | Very verbose, per-operation details |

### 3. Context Propagation

Always include correlation IDs for distributed systems:

```rust
#[tracing::instrument(fields(request_id = %req_id, user_id = %user.id))]
async fn handle_request(req_id: Uuid, user: User) {
    // All nested spans inherit these fields
    process_order(&user).await;
}
```

### 4. Sampling in High-Throughput Services

For very high traffic, sample traces:

```rust
use opentelemetry_sdk::trace::Sampler;

let provider = TracerProvider::builder()
    .with_sampler(Sampler::TraceIdRatioBased(0.1)) // 10% sampling
    .build();
```

### 5. Never Log Sensitive Data

```rust
#[instrument(skip(password, credit_card))]
fn process_payment(user_id: u64, password: &str, credit_card: &str) {
    // password and credit_card are not recorded
}
```

### 6. Guard Lifecycle

The `WorkerGuard` from non-blocking writers must be kept alive:

```rust
fn main() {
    let _guard = init_logging(); // Don't drop this!

    run_app();

    // Guard drops here, flushing remaining logs
}
```

## Debugging Tips

### Enable Verbose Logging Temporarily

```bash
# Full trace for your crate, info for dependencies
RUST_LOG="trace,hyper=info,tokio=info" cargo run
```

### View Span Hierarchy

Use `tracing-tree` for debugging:

```toml
[dependencies]
tracing-tree = "0.4"
```

```rust
use tracing_tree::HierarchicalLayer;

tracing_subscriber::registry()
    .with(HierarchicalLayer::new(2)) // 2-space indent
    .init();
```

### Test Logging Setup

```rust
#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_with_logging() {
        tracing::info!("This appears in test output");
        // Logs are captured and shown on test failure
    }
}
```
