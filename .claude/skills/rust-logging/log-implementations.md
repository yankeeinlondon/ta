# Log Crate Implementations

The `log` crate is a facade - it provides macros but no output. Choose an implementation based on your needs.

## Comparison

| Implementation | Best For | Key Features |
|----------------|----------|--------------|
| `env_logger` | CLIs, development | Simple, env-var config, minimal setup |
| `fern` | Apps needing file + console | Builder API, multiple outputs, custom formatting |
| `flexi_logger` | Production with rotation | File rotation, runtime reconfiguration |
| `log4rs` | Enterprise/ops-driven config | YAML/TOML config files, log4j-style |

## env_logger

The simplest choice - configured via `RUST_LOG` environment variable.

```toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

```rust
use log::{info, debug, warn, error};

fn main() {
    env_logger::init();

    info!("Application started");
    debug!("Debug details: {:?}", some_data);
}
```

### RUST_LOG Syntax

```bash
# Global level
RUST_LOG=info cargo run

# Per-crate
RUST_LOG=warn,my_crate=debug cargo run

# Per-module
RUST_LOG=my_crate::db=trace cargo run

# Multiple targets
RUST_LOG="my_crate=debug,hyper=warn,tokio=info" cargo run
```

### Customization

```rust
env_logger::Builder::from_default_env()
    .filter_level(log::LevelFilter::Info)
    .filter_module("noisy_crate", log::LevelFilter::Warn)
    .format_timestamp_secs()
    .init();
```

### Pros/Cons

- **Pros**: Zero config, 12-factor friendly, widely understood
- **Cons**: No file output, limited formatting, no rotation

## fern

Builder-style logger with multiple outputs.

```toml
[dependencies]
log = "0.4"
fern = "0.7"
chrono = "0.4"
```

```rust
use chrono::Local;
use log::LevelFilter;

fn setup_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::Info)
        .level_for("noisy_crate", LevelFilter::Warn)
        // Output to stderr
        .chain(std::io::stderr())
        // Also output to file
        .chain(fern::log_file("app.log")?)
        .apply()?;

    Ok(())
}

fn main() {
    setup_logging().unwrap();
    log::info!("Started with fern");
}
```

### Multiple Dispatchers

Route different logs to different destinations:

```rust
fern::Dispatch::new()
    .level(LevelFilter::Info)
    .chain(
        fern::Dispatch::new()
            .filter(|meta| meta.target().starts_with("my_app::api"))
            .chain(fern::log_file("api.log").unwrap())
    )
    .chain(
        fern::Dispatch::new()
            .filter(|meta| meta.target().starts_with("my_app::db"))
            .chain(fern::log_file("db.log").unwrap())
    )
    .chain(std::io::stderr())
    .apply()
    .unwrap();
```

### Pros/Cons

- **Pros**: Clean builder API, multiple outputs, good formatting control
- **Cons**: No built-in rotation, no runtime reconfiguration

## flexi_logger

Production-ready with rotation and runtime control.

```toml
[dependencies]
log = "0.4"
flexi_logger = "0.29"
```

```rust
use flexi_logger::{Logger, Duplicate, Criterion, Naming, Cleanup};

fn main() {
    Logger::try_with_env_or_str("info, my_crate::db=debug")
        .unwrap()
        .log_to_file()
        .duplicate_to_stderr(Duplicate::Info) // Also print to stderr
        .rotate(
            Criterion::Size(10_000_000), // 10MB
            Naming::Numbers,
            Cleanup::KeepLogFiles(7),
        )
        .start()
        .unwrap();

    log::info!("flexi_logger initialized");
}
```

### Runtime Reconfiguration

```rust
use flexi_logger::{Logger, LoggerHandle};

fn main() {
    let handle: LoggerHandle = Logger::try_with_str("info")
        .unwrap()
        .start()
        .unwrap();

    // Later, change log level at runtime
    handle.set_new_spec(flexi_logger::LogSpecification::parse("debug").unwrap());
}
```

### Pros/Cons

- **Pros**: File rotation, runtime control, env-style filtering
- **Cons**: More setup than env_logger, overkill for small tools

## log4rs

Enterprise-style with external config files.

```toml
[dependencies]
log = "0.4"
log4rs = "1"
```

### YAML Configuration

`log4rs.yaml`:
```yaml
refresh_rate: 30 seconds

appenders:
  stdout:
    kind: console
    encoder:
      pattern: "{d(%Y-%m-%d %H:%M:%S)} {l} {t} - {m}{n}"

  rolling_file:
    kind: rolling_file
    path: logs/app.log
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 10 mb
      roller:
        kind: fixed_window
        base: 1
        count: 5
        pattern: logs/app.{}.log.gz

root:
  level: info
  appenders:
    - stdout
    - rolling_file

loggers:
  my_app::db:
    level: debug
    appenders:
      - rolling_file
    additive: false
```

```rust
fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    log::info!("Configured via YAML");
}
```

### Programmatic Configuration

```rust
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};

fn main() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} - {m}{n}")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(log::LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
```

### Pros/Cons

- **Pros**: External config, hot reload, familiar to Java developers
- **Cons**: Heavier, more complexity, YAML dependency

## Recommendations

1. **Starting out**: Use `env_logger`
2. **Need file output**: Use `fern`
3. **Production with rotation**: Use `flexi_logger`
4. **Ops team manages config**: Use `log4rs`
5. **Async/complex diagnostics**: Consider `tracing` ecosystem instead

## Library Authors

Libraries should only depend on `log`, not any implementation:

```toml
[dependencies]
log = "0.4"
```

```rust
use log::{debug, info, error};

pub fn process(data: &str) -> Result<(), Error> {
    debug!("Processing: {}", data);
    // ...
}
```

This lets application authors choose their preferred logger.
