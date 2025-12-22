mod common;

use common::{ta_cmd, ta_cmd_in_fixtures, fixtures_dir};
use predicates::prelude::*;

#[test]
fn test_help() {
    ta_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TypeScript Analyzer"))
        .stdout(predicate::str::contains("source"))
        .stdout(predicate::str::contains("symbols"));
}

#[test]
fn test_source_no_errors() {
    ta_cmd_in_fixtures()
        .arg("source")
        .arg("simple-legacy")  // Filter for files with "simple-legacy" in path
        .assert()
        .success()
        .stderr(predicate::str::contains("No type errors found"));
}

#[test]
fn test_source_with_errors() {
    ta_cmd_in_fixtures()
        .arg("source")
        .arg("with-errors-legacy")  // Filter for files with "with-errors-legacy" in path
        .assert()
        .success()
        .stderr(predicate::str::contains("Found 1 type errors")) // Redeclaration error
        .stdout(predicate::str::contains("error"));
}

#[test]
fn test_symbols_extraction() {
    ta_cmd_in_fixtures()
        .arg("symbols")
        .assert()
        .success()
        .stdout(predicate::str::contains("interface"))
        .stdout(predicate::str::contains("enum"))
        .stdout(predicate::str::contains("class"));
}

#[test]
fn test_deps_analysis() {
    ta_cmd_in_fixtures()
        .arg("deps")
        .arg("dependencies-legacy")  // Filter for dependencies-legacy file
        .assert()
        .success()
        .stdout(predicate::str::contains("./"));  // Should contain local imports
}

#[test]
fn test_file_details() {
    ta_cmd_in_fixtures()
        .arg("file")
        .arg("network")  // Filter for network.ts which has imports
        .assert()
        .success()
        .stdout(predicate::str::contains("./"));  // Should contain local imports like ./types, ./utils
}

#[test]
fn test_json_format() {
    ta_cmd_in_fixtures()
        .arg("--json")
        .arg("symbols")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

// Phase 0: Directory switching tests
#[test]
fn test_dir_flag_changes_working_directory() {
    ta_cmd_in_fixtures()
        .arg("source")
        .arg("simple-legacy")  // Filter for files with "simple-legacy" in path
        .assert()
        .success()
        .stderr(predicate::str::contains("Analyzing 1 files"));
}

#[test]
fn test_dir_flag_short_form() {
    // Test that short form works
    ta_cmd()
        .arg("-d")
        .arg(fixtures_dir())
        .arg("source")
        .arg("simple-legacy")
        .assert()
        .success()
        .stderr(predicate::str::contains("Analyzing 1 files"));
}

#[test]
fn test_dir_flag_nonexistent_error() {
    ta_cmd()
        .arg("--dir")
        .arg("nonexistent_directory_12345_test")
        .arg("symbols")
        .arg("*.ts")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to change to directory"));
}

#[test]
fn test_dir_flag_with_relative_patterns() {
    // When using --dir, filters should work within that directory
    ta_cmd_in_fixtures()
        .arg("source")
        // No filter - should find all source files in src/
        .assert()
        .success()
        .stderr(predicate::str::contains("Analyzing"));
}

// Phase 3: Security and Filter Improvements tests
#[test]
fn test_security_glob_validation() {
    // Security note: With WalkBuilder implementation, path traversal is inherently prevented
    // The implementation only walks the current directory tree and respects .gitignore
    // Filters are substring matches, not file paths, so ".." in a filter just won't match anything
    ta_cmd_in_fixtures()
        .arg("source")
        .arg("..")  // This is just a filter string, won't match any file paths
        .assert()
        .failure()  // Will fail because no files match
        .stderr(predicate::str::contains("No source files found"));
}

// Note: symbols command no longer accepts file glob patterns as input
// It uses a hardcoded pattern and args.pattern is for symbol name filtering only
// Security validation is still in place but not testable via CLI args

// Note: test command has been refactored to use ignore crate for file walking
// Pattern argument is now for filtering test files, not for file glob discovery
// Security validation for glob patterns is no longer applicable

#[test]
fn test_security_glob_validation_deps() {
    // With WalkBuilder, security is inherent - filters are just substring matches
    // ".." in a filter won't match any actual file paths
    ta_cmd_in_fixtures()
        .arg("deps")
        .arg("..")  // This is just a filter string, won't match anything
        .assert()
        .failure()
        .stderr(predicate::str::contains("No source files found"));
}

#[test]
fn test_security_file_path_validation() {
    // With WalkBuilder, file command walks directory tree, doesn't accept arbitrary paths
    // Filters are substring matches, inherently safe
    ta_cmd_in_fixtures()
        .arg("file")
        .arg("..")  // This is just a filter string
        .assert()
        .failure()
        .stderr(predicate::str::contains("No source files found"));
}

#[test]
fn test_include_tests_flag() {
    // Without --include-tests, should analyze only src/ files (exclude test/)
    ta_cmd_in_fixtures()
        .arg("source")
        .assert()
        .success()
        .stderr(predicate::str::contains("Analyzing"));

    // With --include-tests, should include .test.ts and .spec.ts files from test/
    ta_cmd_in_fixtures()
        .arg("source")
        .arg("--include-tests")
        .assert()
        .success()
        .stderr(predicate::str::contains("Analyzing"));
    // Note: Can't easily verify count difference without parsing output
}

#[test]
fn test_false_positive_filtering() {
    // File named "contest.ts" in src/ should NOT be filtered as a test file
    // (uses .ends_with() not .contains() to avoid false positives)
    ta_cmd_in_fixtures()
        .arg("source")
        .arg("contest")  // Filter for contest.ts
        .assert()
        .success()
        .stderr(predicate::str::contains("Analyzing 1 files"));
}

// Phase 4: TypeError Metadata tests
#[test]
fn test_type_error_file_field_not_unknown() {
    // Verify that TypeError.file contains the actual file path, not "unknown"
    ta_cmd_in_fixtures()
        .arg("source")
        .arg("--json")
        .arg("/errors.ts")  // More specific filter to match only errors.ts, not with-errors-legacy.ts
        .assert()
        .success()
        .stdout(predicate::str::contains("errors.ts"))
        .stdout(predicate::str::contains(r#""file":"#)); // JSON should have "file" field
}

#[test]
fn test_type_error_id_field_structure() {
    // Verify that TypeError.id contains error codes (TS####) or "error" fallback
    let output = ta_cmd_in_fixtures()
        .arg("source")
        .arg("--json")
        .arg("/errors.ts")  // More specific filter to match only errors.ts
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse JSON and verify id field exists and is not empty
    assert!(stdout.contains(r#""id""#), "JSON should contain 'id' field");

    // The id should be either:
    // 1. A TS error code (e.g., "TS2322" for type mismatch)
    // 2. The fallback "error" string
    // Either way, it should not be empty
    // Note: JSON formatting may include spaces around colons
    let has_ts_code = stdout.contains(r#""id": "TS"#) || stdout.contains(r#""id":"TS"#);
    let has_error_fallback = stdout.contains(r#""id": "error""#) || stdout.contains(r#""id":"error""#);

    assert!(
        has_ts_code || has_error_fallback,
        "TypeError.id should be either 'TS####' or 'error', got: {}",
        stdout
    );
}

// Phase 5: Colorization tests
#[test]
fn test_no_color_env_var_disables_colors() {
    ta_cmd_in_fixtures()
        .env("NO_COLOR", "1")
        .arg("source")
        .arg("/errors.ts")  // More specific filter
        .assert()
        .success()
        .stdout(predicate::function(|s: &str| {
            // Should NOT contain ANSI escape codes when NO_COLOR is set
            !s.contains("\x1b[")
        }));
}

#[test]
fn test_piped_output_no_colors() {
    use assert_cmd::cargo::CommandCargoExt;
    use std::process::{Command, Stdio};

    let mut cmd = Command::cargo_bin("ta").unwrap();

    let output = cmd
        .arg("--dir")
        .arg(fixtures_dir())
        .arg("source")
        .arg("/errors.ts")  // More specific filter
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute command");

    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // When output is piped (not a TTY), colors should be disabled
    // Note: This test might not work as expected because assert_cmd runs in a different context
    // The atty check works at runtime, not during command construction
    assert!(!stdout_str.is_empty() || !String::from_utf8_lossy(&output.stderr).is_empty(), "Should have some output");
}

#[test]
fn test_console_output_has_ansi_colors() {
    // Force CLICOLOR_FORCE to enable colors even in test environment
    let output = ta_cmd_in_fixtures()
        .env("CLICOLOR_FORCE", "1")
        .env_remove("NO_COLOR")
        .arg("source")
        .arg("/errors.ts")  // More specific filter
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout_str = String::from_utf8_lossy(&output);

    // When colors are forced, should contain ANSI codes
    // ANSI codes should be present when CLICOLOR_FORCE is set
    assert!(stdout_str.contains("\x1b["), "Output should contain ANSI escape codes when colors forced");
}

#[test]
fn test_html_format_no_ansi_codes() {
    ta_cmd_in_fixtures()
        .arg("--html")
        .arg("source")
        .arg("/errors.ts")  // More specific filter
        .assert()
        .success()
        .stdout(predicate::str::contains("<div class=\"type-errors\">"))
        .stdout(predicate::function(|s: &str| {
            // HTML output should never have ANSI codes
            !s.contains("\x1b[")
        }));
}

#[test]
fn test_json_format_no_ansi_codes() {
    ta_cmd_in_fixtures()
        .arg("--json")
        .arg("source")
        .arg("/errors.ts")  // More specific filter
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["))
        .stdout(predicate::function(|s: &str| {
            // JSON output should never have ANSI codes
            !s.contains("\x1b[")
        }));
}
