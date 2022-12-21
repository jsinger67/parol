use std::{
    io::Write,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use tempfile::tempdir;

fn snapshot_path(name: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest_dir)
        .join("tests")
        .join("snapshots")
        .join(name)
}

fn diff<L, R>(left: L, right: R)
where
    L: AsRef<std::ffi::OsStr>,
    R: AsRef<std::ffi::OsStr>,
{
    let diff = Command::new("diff")
        .arg("-r")
        .args(["-C", "3"])
        .args(["--exclude", ".git"])
        .args(["--exclude", "Cargo.lock"])
        .arg(left)
        .arg(right)
        .assert();

    let output = diff.get_output();
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();

    diff.success();
}

#[test]
fn snapshot_lib() {
    let path = tempdir().unwrap();
    Command::cargo_bin("parol")
        .unwrap()
        .current_dir(&path)
        .args(["new", "--lib", "--path", "lib", "--name", "snapshot_lib"])
        .assert()
        .success();

    diff(
        path.path().join("lib").to_str().unwrap(),
        snapshot_path("lib"),
    )
}

#[test]
fn snapshot_bin() {
    let path = tempdir().unwrap();
    Command::cargo_bin("parol")
        .unwrap()
        .current_dir(&path)
        .args(["new", "--bin", "--path", "bin", "--name", "snapshot_bin"])
        .assert()
        .success();

    diff(
        path.path().join("bin").to_str().unwrap(),
        snapshot_path("bin"),
    )
}
