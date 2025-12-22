use assert_cmd::Command;
use std::path::PathBuf;

pub fn ta_cmd() -> Command {
    Command::cargo_bin("ta").unwrap()
}

pub fn ta_cmd_in_fixtures() -> Command {
    let mut cmd = Command::cargo_bin("ta").unwrap();
    cmd.arg("--dir").arg(fixtures_dir());
    cmd
}

pub fn fixtures_dir() -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.to_str().unwrap().to_string()
}