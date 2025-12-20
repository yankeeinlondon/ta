# JSON Reporting

Generate structured JSON reports for CI/CD integration, dashboards, and automated tooling.

## Report Structure

```rust
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct FullReport {
    pub summary: Summary,
    pub files: HashMap<String, FileAnalysis>,
    pub dead_code: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct Summary {
    pub total_files: usize,
    pub total_functions: usize,
    pub scan_duration_ms: u128,
}

#[derive(Serialize, Debug)]
pub struct FileAnalysis {
    pub functions: Vec<String>,
    pub complexity_score: usize, // e.g., count of branches
}
```

## Parallel Collection into JSON

```rust
use dashmap::DashMap;
use std::time::Instant;
use std::sync::Arc;
use rayon::prelude::*;

fn generate_report(file_paths: Vec<PathBuf>) -> String {
    let start_time = Instant::now();
    let file_results = Arc::new(DashMap::new());
    let global_usages = Arc::new(DashSet::new());

    file_paths.par_iter().for_each(|path| {
        let allocator = Allocator::default();
        let source = std::fs::read_to_string(path).unwrap();
        let ret = Parser::new(&allocator, &source, SourceType::from_path(path).unwrap()).parse();
        let semantic = SemanticBuilder::new().build(&ret.program).semantic;

        let mut functions = Vec::new();

        for node in semantic.nodes().iter() {
            if let AstKind::Function(func) = node.kind() {
                if let Some(id) = &func.id {
                    functions.push(id.name.to_string());
                }
            }

            // Track usages for dead code analysis
            if let AstKind::CallExpression(call) = node.kind() {
                if let oxc::ast::ast::Expression::IdentifierReference(ident) = &call.callee {
                    global_usages.insert(ident.name.to_string());
                }
            }
        }

        file_results.insert(
            path.to_string_lossy().to_string(),
            FileAnalysis {
                functions,
                complexity_score: 0, // Placeholder
            }
        );
    });

    // Finalize report
    let total_functions = file_results.iter().map(|r| r.value().functions.len()).sum();

    // Identify dead code
    let dead_code: Vec<String> = file_results.iter()
        .flat_map(|r| r.value().functions.clone())
        .filter(|f| !global_usages.contains(f))
        .collect();

    let report = FullReport {
        summary: Summary {
            total_files: file_results.len(),
            total_functions,
            scan_duration_ms: start_time.elapsed().as_millis(),
        },
        files: file_results.into_read_only()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        dead_code,
    };

    serde_json::to_string_pretty(&report).unwrap()
}
```

## CI/CD Integration

Write the report and fail the build if dead code is found:

```rust
fn main() {
    let file_paths = /* ... */;
    let report_json = generate_report(file_paths);

    // Write to file
    std::fs::write("analysis-report.json", &report_json).unwrap();

    // Parse and check
    let report: FullReport = serde_json::from_str(&report_json).unwrap();

    if !report.dead_code.is_empty() {
        eprintln!("FAIL: Found {} unused functions:", report.dead_code.len());
        for func in &report.dead_code {
            eprintln!("  - {}", func);
        }
        std::process::exit(1);
    }

    println!("OK: No dead code detected");
}
```

## GitHub Actions Integration

```yaml
name: Code Analysis

on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run OXC Analysis
        run: cargo run --release -- analyze ./src

      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: analysis-report
          path: analysis-report.json

      - name: Check for Dead Code
        run: |
          if [ -s analysis-report.json ]; then
            echo "Analysis complete"
          fi
```

## Rich Reporting with Diagnostics

Include detailed diagnostic information:

```rust
#[derive(Serialize, Debug)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: String, // "error", "warning", "info"
    pub message: String,
    pub code: String, // e.g., "unused-function"
}

#[derive(Serialize, Debug)]
pub struct DetailedReport {
    pub summary: Summary,
    pub diagnostics: Vec<Diagnostic>,
}

fn create_diagnostic(
    source: &str,
    file: &str,
    span: oxc::span::Span,
    message: String,
    code: String
) -> Diagnostic {
    let line = source[..span.start as usize].lines().count();
    let column = source[..span.start as usize]
        .lines()
        .last()
        .map(|l| l.len())
        .unwrap_or(0);

    Diagnostic {
        file: file.to_string(),
        line,
        column,
        severity: "warning".to_string(),
        message,
        code,
    }
}
```

## Complexity Metrics

Calculate cyclomatic complexity while analyzing:

```rust
fn calculate_complexity(semantic: &Semantic) -> usize {
    let mut complexity = 1; // Base complexity

    for node in semantic.nodes().iter() {
        match node.kind() {
            AstKind::IfStatement(_) => complexity += 1,
            AstKind::SwitchCase(_) => complexity += 1,
            AstKind::WhileStatement(_) => complexity += 1,
            AstKind::ForStatement(_) => complexity += 1,
            AstKind::ForInStatement(_) => complexity += 1,
            AstKind::ForOfStatement(_) => complexity += 1,
            AstKind::ConditionalExpression(_) => complexity += 1,
            AstKind::LogicalExpression(expr) => {
                if matches!(expr.operator, LogicalOperator::And | LogicalOperator::Or) {
                    complexity += 1;
                }
            }
            _ => {}
        }
    }

    complexity
}
```

## Dashboard Integration

Generate HTML reports:

```rust
use serde_json::json;

fn generate_html_report(report: &FullReport) -> String {
    let json_data = serde_json::to_string_pretty(report).unwrap();

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Code Analysis Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f0f0f0; padding: 15px; border-radius: 5px; }}
        .dead-code {{ color: red; font-weight: bold; }}
        table {{ border-collapse: collapse; width: 100%; margin-top: 20px; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; }}
    </style>
</head>
<body>
    <h1>Code Analysis Report</h1>
    <div class="summary">
        <h2>Summary</h2>
        <p>Total Files: {}</p>
        <p>Total Functions: {}</p>
        <p>Scan Duration: {}ms</p>
        <p class="dead-code">Dead Code Items: {}</p>
    </div>
    <h2>Files Analyzed</h2>
    <table>
        <tr>
            <th>File</th>
            <th>Functions</th>
            <th>Complexity</th>
        </tr>
        {}
    </table>
    <h2>Raw JSON Data</h2>
    <pre>{}</pre>
</body>
</html>"#,
        report.summary.total_files,
        report.summary.total_functions,
        report.summary.scan_duration_ms,
        report.dead_code.len(),
        report.files.iter()
            .map(|(file, analysis)| {
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    file,
                    analysis.functions.len(),
                    analysis.complexity_score
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        json_data
    )
}
```

## Streaming JSON for Large Codebases

For very large reports, stream JSON instead of building in memory:

```rust
use std::fs::File;
use std::io::BufWriter;

fn stream_report(file_paths: Vec<PathBuf>) -> std::io::Result<()> {
    let file = File::create("analysis-report.json")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "{{")?;
    writeln!(writer, r#"  "files": {{"#)?;

    let mut first = true;

    for path in file_paths {
        // ... analyze file ...

        if !first {
            writeln!(writer, ",")?;
        }
        first = false;

        writeln!(
            writer,
            r#"    "{}": {{ "functions": {} }}"#,
            path.display(),
            serde_json::to_string(&functions)?
        )?;
    }

    writeln!(writer, "  }}")?;
    writeln!(writer, "}}")?;

    Ok(())
}
```

## Example Report Output

```json
{
  "summary": {
    "total_files": 150,
    "total_functions": 1243,
    "scan_duration_ms": 2450
  },
  "files": {
    "src/main.ts": {
      "functions": ["main", "init", "processData"],
      "complexity_score": 15
    },
    "src/utils.ts": {
      "functions": ["formatDate", "parseJSON"],
      "complexity_score": 3
    }
  },
  "dead_code": [
    "unusedHelper",
    "oldLegacyFunction"
  ]
}
```

## Integration Patterns

### Slack/Discord Notifications

```rust
fn send_notification(report: &FullReport) {
    if !report.dead_code.is_empty() {
        let message = format!(
            "⚠️ Code Analysis found {} unused functions",
            report.dead_code.len()
        );
        // Send to webhook...
    }
}
```

### Trend Analysis

Store historical reports and track changes over time:

```rust
#[derive(Serialize, Deserialize)]
struct HistoricalData {
    date: String,
    total_functions: usize,
    dead_code_count: usize,
}

fn save_historical_data(report: &FullReport) {
    let data = HistoricalData {
        date: chrono::Utc::now().to_rfc3339(),
        total_functions: report.summary.total_functions,
        dead_code_count: report.dead_code.len(),
    };

    // Append to historical log...
}
```
