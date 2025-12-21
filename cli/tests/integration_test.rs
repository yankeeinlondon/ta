mod common;

use common::{ta_cmd, fixture_path};
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
    let path = fixture_path("simple.ts");
    ta_cmd()
        .arg("source")
        .arg(path)
        .assert()
        .success()
        .stderr(predicate::str::contains("No type errors found"));
}

#[test]
fn test_source_with_errors() {
    let path = fixture_path("with-errors.ts");
    ta_cmd()
        .arg("source")
        .arg(path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Found 1 type errors")) // Redeclaration error
        .stdout(predicate::str::contains("error"));
}

#[test]
fn test_symbols_extraction() {
    let path = fixture_path("exports.ts");
    ta_cmd()
        .arg("symbols")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Interface User"))
        .stdout(predicate::str::contains("Enum Role"))
        .stdout(predicate::str::contains("Type ID"))
        .stdout(predicate::str::contains("Class Store"));
}

#[test]
fn test_deps_analysis() {
    let path = fixture_path("dependencies.ts");
    ta_cmd()
        .arg("deps")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("./simple"))
        .stdout(predicate::str::contains("fs"))
        .stdout(predicate::str::contains("./exports"));
}

#[test]
fn test_file_details() {
    let path = fixture_path("simple.ts");
    ta_cmd()
        .arg("file")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Calculator"))
        .stdout(predicate::str::contains("multiply"));
}

#[test]
fn test_json_format() {
    let path = fixture_path("simple.ts");
    ta_cmd()
        .arg("--format")
        .arg("json")
        .arg("symbols")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["))
        .stdout(predicate::str::contains("Calculator"));
}
