use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use owo_colors::OwoColorize;
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

        std::io::stderr()
            .write_all(
                format!(
                    "\nWARNING! {} is not released yet!\n\n",
                    local_package_version.bright_red()
                )
                .as_bytes(),
            )
            .unwrap();

        let output = std::process::Command::new("git")
            .args(["diff", "--no-index"])
            .args([actual.as_ref(), expected.as_ref()])
            .output()
            .unwrap();
        write_command_output(&output);
    } else {
        std::io::stderr()
            .write_all(
                format!(
                    "\nINFO! {} is already released!\n\n",
                    local_package_version.bright_green()
                )
                .as_bytes(),
            )
            .unwrap();

        // If not in pre-release mode we assert on diffs.
        let diff = Command::new("git")
            .args(["diff", "--no-index"])
            .args([actual.as_ref(), expected.as_ref()])
            .assert();

        let output = diff.get_output();
        write_command_output(output);
        diff.success();
    };
}

fn clean_build_artifacts(sub: &str) {
    // The following files could have been created by some language extensions.
    // To avoid the test to fail because of this we delete the generated elements.
    // We ignore possible io errors here.
    let _ = fs::remove_file(snapshot_path(sub).join("Cargo.lock"));
    let _ = fs::remove_dir_all(snapshot_path(sub).join("target"));
    let _ = fs::remove_file(snapshot_path(sub).join(format!("snapshot_{sub}-exp.par")));
}

#[test]
#[ignore = "Can fail depending on the cargo version used."]
fn snapshot_lib() {
    let path = tempdir().unwrap();
    Command::cargo_bin("parol")
        .unwrap()
        .current_dir(&path)
        .args(["new", "--lib", "--path", "lib", "--name", "snapshot_lib"])
        .assert()
        .success();
    clean_build_artifacts("lib");
    diff(path.path().join("lib"), snapshot_path("lib"));
}

#[test]
#[ignore = "Can fail depending on the cargo version used."]
fn snapshot_bin() {
    let path = tempdir().unwrap();
    Command::cargo_bin("parol")
        .unwrap()
        .current_dir(&path)
        .args(["new", "--bin", "--path", "bin", "--name", "snapshot_bin"])
        .assert()
        .success();
    clean_build_artifacts("bin");
    diff(path.path().join("bin"), snapshot_path("bin"));
}
