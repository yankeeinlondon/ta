use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use notify_debouncer_full::{new_debouncer, DebouncedEvent};
use notify_debouncer_full::notify::{RecursiveMode, EventKind};
use crate::models::{SymbolKind, TestStatus};
use crate::analyzer::{Analyzer, AnalysisResult, AnalysisOptions};
use crate::Result;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WatchEvent {
    SourceFileChanged { file: String, content: String },
    SourceFileCreated { file: String },
    SourceFileRemoved { file: String },
    SymbolRenamed { old_name: String, new_name: String, file: String },
    SymbolAdded { name: String, kind: SymbolKind, file: String },
    SymbolRemoved { name: String, file: String },
    ModuleDepChanged { file: String },
    ExternalDepChanged { package: String },
    TestStatusChanged { file: String, test: String, status: TestStatus },
    NewFailingTest { file: String, test: String },
    TestFixed { file: String, test: String },
    NewTestAdded { file: String, test: String },
}

pub trait WatchHandler: Send + Sync {
    fn handle_event(&self, event: &WatchEvent) -> Result<()>;
}

pub struct FileWatcher {
    analyzer: Analyzer,
    handlers: Vec<Box<dyn WatchHandler>>,
    previous_state: Arc<Mutex<Option<AnalysisResult>>>,
}

impl FileWatcher {
    pub fn new(options: AnalysisOptions, handlers: Vec<Box<dyn WatchHandler>>) -> Self {
        Self {
            analyzer: Analyzer::new(options),
            handlers,
            previous_state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn watch(&self, paths: &[PathBuf]) -> Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut debouncer = new_debouncer(Duration::from_millis(500), None, tx)
            .map_err(|e| crate::error::Error::AnalysisError(format!("Failed to create debouncer: {}", e)))?;

        for path in paths {
            debouncer.watch(path, RecursiveMode::Recursive)
                .map_err(|e| crate::error::Error::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        }

        println!("Watching for changes in {:?}...", paths);

        for result in rx {
            match result {
                Ok(events) => {
                    self.process_debounced_events(events)?;
                }
                Err(errors) => {
                    for error in errors {
                        eprintln!("Watch error: {:?}", error);
                    }
                }
            }
        }

        Ok(())
    }

    fn process_debounced_events(&self, events: Vec<DebouncedEvent>) -> Result<()> {
        let mut affected_files = Vec::new();
        for event in events {
            let kind = event.kind;
            match kind {
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                    for path in &event.paths {
                        if self.is_ts_file(path) {
                            affected_files.push(path.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        if affected_files.is_empty() {
            return Ok(());
        }

        let current_result = self.analyzer.analyze_files(&affected_files)?;
        let mut previous_state = self.previous_state.lock().unwrap();
        
        if let Some(prev) = previous_state.as_ref() {
            let diff_events = self.compute_diff(prev, &current_result);
            for event in diff_events {
                for handler in &self.handlers {
                    handler.handle_event(&event)?;
                }
            }
        }

        *previous_state = Some(current_result);
        Ok(())
    }

    fn is_ts_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s == "ts" || s == "tsx")
            .unwrap_or(false)
    }

    fn compute_diff(&self, old: &AnalysisResult, new: &AnalysisResult) -> Vec<WatchEvent> {
        let mut events = Vec::new();

        // 1. Detect Symbol changes
        for new_sym in &new.symbols {
            if !old.symbols.iter().any(|s| s.name == new_sym.name && s.kind == new_sym.kind && s.file == new_sym.file) {
                events.push(WatchEvent::SymbolAdded {
                    name: new_sym.name.clone(),
                    kind: new_sym.kind.clone(),
                    file: new_sym.file.clone(),
                });
            }
        }

        for old_sym in &old.symbols {
            if !new.symbols.iter().any(|s| s.name == old_sym.name && s.kind == old_sym.kind && s.file == old_sym.file) {
                events.push(WatchEvent::SymbolRemoved {
                    name: old_sym.name.clone(),
                    file: old_sym.file.clone(),
                });
            }
        }

        // 2. Detect Test changes
        for new_test in &new.tests {
            if let Some(old_test) = old.tests.iter().find(|t| t.file == new_test.file && t.test_name == new_test.test_name) {
                if old_test.status != new_test.status {
                    events.push(WatchEvent::TestStatusChanged {
                        file: new_test.file.clone(),
                        test: new_test.test_name.clone(),
                        status: new_test.status.clone(),
                    });
                    
                    if new_test.status == TestStatus::Failing && old_test.status != TestStatus::Failing {
                         events.push(WatchEvent::NewFailingTest {
                            file: new_test.file.clone(),
                            test: new_test.test_name.clone(),
                        });
                    } else if new_test.status == TestStatus::Passing && old_test.status == TestStatus::Failing {
                         events.push(WatchEvent::TestFixed {
                            file: new_test.file.clone(),
                            test: new_test.test_name.clone(),
                        });
                    }
                }
            } else {
                events.push(WatchEvent::NewTestAdded {
                    file: new_test.file.clone(),
                    test: new_test.test_name.clone(),
                });
            }
        }

        events
    }
}
