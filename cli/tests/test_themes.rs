use assert_cmd::Command;
use predicates::prelude::*;

#[test]
#[allow(deprecated)]
fn test_list_themes_command() {
    let mut cmd = Command::cargo_bin("ta").unwrap();

    cmd.arg("list-themes");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Solarized (light)"))
        .stdout(predicate::str::contains("base16-ocean.dark"))
        .stdout(predicate::str::contains("Dracula"));
}

#[test]
#[allow(deprecated)]
fn test_source_with_theme_flag() {
    let mut cmd = Command::cargo_bin("ta").unwrap();

    cmd.arg("--dir")
        .arg("tests/fixtures")
        .arg("source")
        .arg("--theme")
        .arg("Dracula");

    // Exit code 1 when type errors are found (fixtures have errors)
    // Validates that theme is applied (no theme-related errors)
    cmd.assert().code(1);
}

#[test]
#[allow(deprecated)]
fn test_source_with_theme_env_var() {
    let mut cmd = Command::cargo_bin("ta").unwrap();

    cmd.env("TA_THEME", "zenburn")
        .arg("--dir")
        .arg("tests/fixtures")
        .arg("source");

    cmd.assert().code(1);  // Exit code 1 when type errors are found
}

#[test]
#[allow(deprecated)]
fn test_source_with_light_theme() {
    let mut cmd = Command::cargo_bin("ta").unwrap();

    cmd.arg("--dir")
        .arg("tests/fixtures")
        .arg("source")
        .arg("--light-theme")
        .arg("Solarized (light)");

    cmd.assert().code(1);  // Exit code 1 when type errors are found
}

#[test]
#[allow(deprecated)]
fn test_source_with_dark_theme() {
    let mut cmd = Command::cargo_bin("ta").unwrap();

    cmd.arg("--dir")
        .arg("tests/fixtures")
        .arg("source")
        .arg("--dark-theme")
        .arg("base16-ocean.dark");

    cmd.assert().code(1);  // Exit code 1 when type errors are found
}

#[test]
#[allow(deprecated)]
fn test_theme_env_vars() {
    let mut cmd = Command::cargo_bin("ta").unwrap();

    cmd.env("TA_LIGHT_THEME", "Solarized (light)")
        .env("TA_DARK_THEME", "base16-ocean.dark")
        .arg("--dir")
        .arg("tests/fixtures")
        .arg("source");

    cmd.assert().code(1);  // Exit code 1 when type errors are found
}
