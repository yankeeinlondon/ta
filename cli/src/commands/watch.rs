use clap::Parser;
use std::path::PathBuf;
use color_eyre::eyre::{Result, Context};
use ta_lib::watcher::{FileWatcher, WatchEvent, WatchHandler};
use ta_lib::analyzer::AnalysisOptions;
use ta_lib::output::OutputFormat;

/// Watch for file changes and run analysis
#[derive(Parser, Debug)]
pub struct WatchArgs {
    /// Paths to watch
    #[arg(default_value = ".")]
    pub paths: Vec<PathBuf>,
}

struct CliWatchHandler {
    _format: OutputFormat,
}

impl WatchHandler for CliWatchHandler {
    fn handle_event(&self, event: &WatchEvent) -> ta_lib::Result<()> {
        match event {
            WatchEvent::SymbolAdded { name, kind, file } => {
                println!("[+] Symbol Added: {:?} {} in {}", kind, name, file);
            }
            WatchEvent::SymbolRemoved { name, file } => {
                println!("[-] Symbol Removed: {} from {}", name, file);
            }
            WatchEvent::TestStatusChanged { file, test, status } => {
                println!("[*] Test Status Changed: {} > {} -> {:?}", file, test, status);
            }
            WatchEvent::NewFailingTest { file, test } => {
                println!("[!] NEW FAILING TEST: {} > {}", file, test);
            }
            WatchEvent::TestFixed { file, test } => {
                println!("[OK] Test Fixed: {} > {}", file, test);
            }
            _ => {
                println!("[?] Other Event: {:?}", event);
            }
        }
        Ok(())
    }
}

pub fn handle_watch(args: WatchArgs, format: OutputFormat) -> Result<()> {
    log::debug!("Handling watch command with args: {:?}", args);

    let options = AnalysisOptions {
        parallel: true,
        ..Default::default()
    };

    let handler = Box::new(CliWatchHandler { _format: format });
    let watcher = FileWatcher::new(options, vec![handler]);

    watcher.watch(&args.paths).wrap_err("File watcher failed")?;

    Ok(())
}