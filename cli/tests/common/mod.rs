use assert_cmd::Command;
use std::path::PathBuf;

pub fn ta_cmd() -> Command {
    Command::cargo_bin("ta").unwrap()
}

pub fn fixture_path(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path.to_str().unwrap().to_string()
}