use anyhow::{Result, anyhow};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

macro_rules! binary_path {
    ($binary:literal) => {
        format!(
            "{}{}",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/"),
            $binary
        )
    };
}

fn run_parol(args: &[&str]) -> Result<std::process::ExitStatus> {
    Command::new(binary_path!("parol"))
        .args(args)
        .status()
        .map_err(|e| anyhow!(e))
}

#[test]
fn test_csharp_end_to_end() -> Result<()> {
    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().join("cs_test");
    let project_name = "cs_test";

    // 1. Scaffold the project
    let status = run_parol(&[
        "new",
        "--path",
        project_path.to_str().unwrap(),
        "-b",
        "-L",
        "c-sharp",
    ])?;
    assert!(status.success(), "parol new failed");

    // 2. Update .csproj to point to local Parol.Runtime
    let csproj_path = project_path.join(format!("{}.csproj", project_name));
    let mut csproj_content = fs::read_to_string(&csproj_path)?;

    let runtime_project_path = "d:\\Source\\parol-dotnet\\src\\Parol.Runtime\\Parol.Runtime.csproj";
    if std::path::Path::new(runtime_project_path).exists() {
        csproj_content = csproj_content.replace(
            r#"<PackageReference Include="Parol.Runtime" Version="1.0.0" />"#,
            &format!(r#"<ProjectReference Include="{}" />"#, runtime_project_path),
        );
        fs::write(&csproj_path, csproj_content)?;
    }

    // 3. Build the project (should trigger parol generation via parol.targets)
    let parol_bin_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("debug");

    let path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{};{}", parol_bin_dir.display(), path);

    let output = Command::new("dotnet")
        .current_dir(&project_path)
        .arg("build")
        .env("PATH", &new_path)
        .output()
        .map_err(|e| anyhow!(e))?;

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "dotnet build failed");

    // 4. Run the project
    let test_file = project_path.join("test.txt");
    let status = Command::new("dotnet")
        .current_dir(&project_path)
        .args(["run", "--", test_file.to_str().unwrap()])
        .env("PATH", &new_path)
        .status()
        .map_err(|e| anyhow!(e))?;
    assert!(status.success(), "dotnet run failed");

    Ok(())
}
