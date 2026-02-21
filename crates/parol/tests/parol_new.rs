use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use assert_cmd::{Command, cargo::cargo_bin_cmd};
use owo_colors::OwoColorize;
use tempfile::tempdir;

/// Check if snapshots should be updated based on environment variable
fn should_update_snapshots() -> bool {
    std::env::var("PAROL_UPDATE_SNAPSHOTS")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Update snapshot directory with fresh generated output
fn update_snapshot_directory<P: AsRef<Path>>(
    source: P,
    target: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = source.as_ref();
    let target = target.as_ref();

    if !source.exists() {
        return Err(format!("Source directory does not exist: {}", source.display()).into());
    }

    // Create target directory if it doesn't exist
    if !target.exists() {
        fs::create_dir_all(target)?;
        std::io::stderr()
            .write_all(format!("Created snapshot directory: {}\n", target.display()).as_bytes())?;
    }

    // Copy files recursively, focusing on version-affected files
    copy_directory_contents(source, target)?;

    std::io::stderr().write_all(
        format!(
            "Updated snapshot directory: {} -> {}\n",
            source.display(),
            target.display()
        )
        .as_bytes(),
    )?;

    Ok(())
}

/// Recursively copy directory contents
fn copy_directory_contents<P: AsRef<Path>>(
    source: P,
    target: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = source.as_ref();
    let target = target.as_ref();

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let file_name = entry.file_name();
        let target_path = target.join(&file_name);

        if file_type.is_dir() {
            // Skip .git directories and other artifacts we don't want to copy
            if file_name == ".git" || file_name == "target" {
                continue;
            }

            if !target_path.exists() {
                fs::create_dir_all(&target_path)?;
            }
            copy_directory_contents(&source_path, &target_path)?;
        } else if file_type.is_file() {
            // Skip files we don't want to copy
            if file_name == "Cargo.lock" {
                continue;
            }

            fs::copy(&source_path, &target_path)?;

            // Log version-affected files specifically
            if file_name == "Cargo.toml" {
                std::io::stderr().write_all(
                    format!("Updated version-affected file: {}\n", target_path.display())
                        .as_bytes(),
                )?;
            }
        }
    }

    Ok(())
}

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
    let cargo_lock = actual.as_ref().join("Cargo.lock");
    if cargo_lock.exists() {
        fs::remove_file(cargo_lock).unwrap();
    }
    let git_dir = actual.as_ref().join(".git");
    if git_dir.exists() {
        fs::remove_dir_all(git_dir).unwrap();
    }

    let local_package_version = concat!("parol = \"", env!("CARGO_PKG_VERSION"), "\"");

    // Check if snapshots should be updated via environment variable
    if should_update_snapshots() {
        std::io::stderr()
            .write_all(
                "\nINFO! PAROL_UPDATE_SNAPSHOTS enabled - updating snapshot directory\n"
                    .to_string()
                    .bright_cyan()
                    .to_string()
                    .as_bytes(),
            )
            .unwrap();

        std::io::stderr()
            .write_all(
                format!(
                    "Updating with current version: {}\n\n",
                    local_package_version.bright_green()
                )
                .as_bytes(),
            )
            .unwrap();

        // Update the snapshot directory with fresh generated content
        if let Err(e) = update_snapshot_directory(actual.as_ref(), expected.as_ref()) {
            std::io::stderr()
                .write_all(
                    format!(
                        "ERROR! Failed to update snapshots: {}\n",
                        e.to_string().bright_red()
                    )
                    .as_bytes(),
                )
                .unwrap();
            panic!("Snapshot update failed: {}", e);
        }

        std::io::stderr()
            .write_all(
                "SUCCESS! Snapshots updated successfully.\n"
                    .to_string()
                    .bright_green()
                    .to_string()
                    .as_bytes(),
            )
            .unwrap();

        return; // Exit early when in update mode
    }

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

    cargo_bin_cmd!("parol")
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
    cargo_bin_cmd!("parol")
        .current_dir(&path)
        .args(["new", "--bin", "--path", "bin", "--name", "snapshot_bin"])
        .assert()
        .success();
    clean_build_artifacts("bin");
    diff(path.path().join("bin"), snapshot_path("bin"));
}

#[test]
fn snapshot_csharp() {
    let path = tempdir().unwrap();
    cargo_bin_cmd!("parol")
        .current_dir(&path)
        .args([
            "new",
            "--bin",
            "--path",
            "cs",
            "--name",
            "snapshot_cs",
            "--language",
            "c-sharp",
        ])
        .assert()
        .success();
    // No build artifacts to clean for C# yet in this test context
    diff(path.path().join("cs"), snapshot_path("cs"));
}
