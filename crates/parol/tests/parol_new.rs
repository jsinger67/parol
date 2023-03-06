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

fn write_command_output(output: &std::process::Output) {
    std::io::stdout().write_all(&output.stdout).unwrap();
    std::io::stderr().write_all(&output.stderr).unwrap();
}

fn diff<L, R>(actual: L, expected: R)
where
    L: AsRef<Path>,
    R: AsRef<Path>,
{
    fs::remove_file(actual.as_ref().join("Cargo.lock")).unwrap();
    fs::remove_dir_all(actual.as_ref().join(".git")).unwrap();

    let local_package_version = concat!("parol = \"", env!("CARGO_PKG_VERSION"), "\"");

    // Check if the current package version is already released on crates.io.
    let pre_release_state = !std::str::from_utf8(
        &std::process::Command::new("cargo")
            .args([
                "search", "parol", "--limit", "1", "--quiet", "--color", "never",
            ])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .contains(local_package_version);

    if pre_release_state {
        // In pre-release mode we only print out the diffs here
        // because the comparisons would always fail.
        let output = std::process::Command::new("git")
            .args(["diff", "--no-index"])
            .args([actual.as_ref(), expected.as_ref()])
            .output()
            .unwrap();
        write_command_output(&output);
    } else {
        // If not in pre-release mode we assert on diffs.
        let diff = Command::new("git")
            .args(["diff", "--no-index"])
            .args([actual.as_ref(), expected.as_ref()])
            .assert();

        let output = diff.get_output();
        write_command_output(output);
    };
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
