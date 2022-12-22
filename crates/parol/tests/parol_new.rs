use std::{
    fs,
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

fn diff<L, R>(actual: L, expected: R)
where
    L: AsRef<Path>,
    R: AsRef<Path>,
{
    fs::remove_file(actual.as_ref().join("Cargo.lock")).unwrap();
    fs::remove_dir_all(actual.as_ref().join(".git")).unwrap();
    let diff = Command::new("git")
        .args(["diff", "--no-index"])
        .args([actual.as_ref(), expected.as_ref()])
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

    diff(path.path().join("lib"), snapshot_path("lib"));
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

    diff(path.path().join("bin"), snapshot_path("bin"));
}
