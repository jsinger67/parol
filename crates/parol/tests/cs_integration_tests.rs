use anyhow::{Result, anyhow};
use std::fs;
use std::process::{Command, ExitStatus};
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

fn run_parol(args: &[&str]) -> Result<ExitStatus> {
    Command::new(binary_path!("parol"))
        .args(args)
        .status()
        .map_err(|e| anyhow!(e))
}

fn run_dotnet(args: &[&str], cwd: &std::path::Path) -> Result<ExitStatus> {
    Command::new("dotnet")
        .current_dir(cwd)
        .args(args)
        .status()
        .map_err(|e| anyhow!(e))
}

#[test]
fn test_csharp_end_to_end() -> Result<()> {
    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().join("cs_test");

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

    // 2. Generate the parser
    // We need to run parol on the generated grammar
    let grammar_file = project_path.join("cs_test.par");
    let parser_file = project_path.join("CsTestParser.cs");
    let actions_file = project_path.join("ICsTestActions.cs"); // This is a bit of a guess for the filename

    // Let's check what the actual filenames are or use explicit ones
    let status = run_parol(&[
        "-f",
        grammar_file.to_str().unwrap(),
        "-p",
        parser_file.to_str().unwrap(),
        "-a",
        actions_file.to_str().unwrap(),
        "-t",
        "CsTest",
        "-m",
        "CsTest",
        "-l",
        "c-sharp",
    ])?;
    assert!(status.success(), "parol generation failed");

    // 3. Update .csproj to point to local Parol.Runtime (optional but needed if not available on NuGet)
    let csproj_path = project_path.join("cs_test.csproj");
    let mut csproj_content = fs::read_to_string(&csproj_path)?;

    // Replace the PackageReference with a ProjectReference to the local Parol.Runtime
    let runtime_project_path = "d:\\Source\\parol-dotnet\\src\\Parol.Runtime\\Parol.Runtime.csproj";
    if std::path::Path::new(runtime_project_path).exists() {
        csproj_content = csproj_content.replace(
            r#"<PackageReference Include="Parol.Runtime" Version="1.0.0" />"#,
            &format!(r#"<ProjectReference Include="{}" />"#, runtime_project_path),
        );
        fs::write(&csproj_path, csproj_content)?;
    }

    // 4. Build the project
    let status = run_dotnet(&["build"], &project_path)?;
    assert!(status.success(), "dotnet build failed");

    // 5. Run the project
    let test_file = project_path.join("test.txt");
    let status = run_dotnet(&["run", "--", test_file.to_str().unwrap()], &project_path)?;
    assert!(status.success(), "dotnet run failed");

    Ok(())
}
