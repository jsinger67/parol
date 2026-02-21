use anyhow::{Result, anyhow};
use std::fs;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::{env, path::PathBuf};
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

fn run_parol_output(args: &[&str]) -> Result<std::process::Output> {
    Command::new(binary_path!("parol"))
        .args(args)
        .output()
        .map_err(|e| anyhow!(e))
}

fn cs_runtime_build_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn dotnet_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        Command::new("dotnet")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

fn skip_if_no_dotnet(test_name: &str) -> bool {
    if dotnet_available() {
        false
    } else {
        eprintln!("Skipping {test_name}: dotnet SDK not found in PATH");
        true
    }
}

fn runtime_project_path() -> PathBuf {
    env::var_os("PAROL_RUNTIME_PROJECT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("d:\\Source\\parol-dotnet\\src\\Parol.Runtime\\Parol.Runtime.csproj")
        })
}

fn ensure_local_runtime_reference(
    project_path: &std::path::Path,
    project_name: &str,
) -> Result<()> {
    let csproj_path = project_path.join(format!("{}.csproj", project_name));
    let mut csproj_content = fs::read_to_string(&csproj_path)?;

    let runtime_project_path = runtime_project_path();
    if runtime_project_path.exists() {
        let replacement = format!(
            r#"<ProjectReference Include="{}" />"#,
            runtime_project_path.display()
        );
        csproj_content = csproj_content
            .lines()
            .map(|line| {
                if line.contains("PackageReference") && line.contains("Include=\"Parol.Runtime\"") {
                    replacement.clone()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        fs::write(&csproj_path, csproj_content)?;
    }

    Ok(())
}

fn with_parol_path() -> Result<String> {
    let parol_bin_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("debug");

    let mut path_entries = vec![parol_bin_dir];
    if let Some(existing) = std::env::var_os("PATH") {
        path_entries.extend(std::env::split_paths(&existing));
    }

    let joined = std::env::join_paths(path_entries)?;
    Ok(joined.to_string_lossy().into_owned())
}

#[test]
fn test_with_parol_path_prepends_binary_dir() -> Result<()> {
    let expected_parol_bin_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("debug");

    let joined_path = with_parol_path()?;
    let path_entries: Vec<_> =
        std::env::split_paths(&std::ffi::OsString::from(joined_path)).collect();

    assert!(!path_entries.is_empty(), "PATH must not be empty");
    assert_eq!(path_entries[0], expected_parol_bin_dir);

    Ok(())
}

#[test]
fn test_csharp_lalr1_is_rejected_early() -> Result<()> {
    let temp_dir = tempdir()?;
    let grammar_path = temp_dir.path().join("lalr_for_csharp.par");

    fs::write(
        &grammar_path,
        r#"%start S
%grammar_type 'LALR(1)'

%%

S: "a";
"#,
    )?;

    let parser_path = temp_dir.path().join("Parser.cs");
    let actions_path = temp_dir.path().join("IGrammarActions.cs");

    let output = run_parol_output(&[
        "-f",
        grammar_path.to_str().unwrap(),
        "-p",
        parser_path.to_str().unwrap(),
        "-a",
        actions_path.to_str().unwrap(),
        "-t",
        "Grammar",
        "-m",
        "Grammar",
        "-l",
        "c-sharp",
    ])?;

    assert!(
        !output.status.success(),
        "Expected parol to fail for C# + LALR(1), but it succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("C# code generation currently supports only LL(k) grammars")
            && stderr.contains("%grammar_type 'LALR(1)'")
    );

    Ok(())
}

fn dotnet_build(project_path: &std::path::Path, path: &str) -> Result<std::process::Output> {
    Command::new("dotnet")
        .current_dir(project_path)
        .arg("build")
        .env("PATH", path)
        .output()
        .map_err(|e| anyhow!(e))
}

#[test]
fn test_csharp_end_to_end() -> Result<()> {
    if skip_if_no_dotnet("test_csharp_end_to_end") {
        return Ok(());
    }

    let _guard = cs_runtime_build_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

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
    ensure_local_runtime_reference(&project_path, project_name)?;

    // 3. Build the project (should trigger parol generation via parol.targets)
    let new_path = with_parol_path()?;

    let output = dotnet_build(&project_path, &new_path)?;

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

#[test]
fn test_csharp_typed_generation_with_alternations() -> Result<()> {
    if skip_if_no_dotnet("test_csharp_typed_generation_with_alternations") {
        return Ok(());
    }

    let _guard = cs_runtime_build_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().join("cs_typed");
    let project_name = "cs_typed";

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

    // 2. Replace grammar with one that enforces multi-alternation non-terminal types
    let grammar_path = project_path.join(format!("{}.par", project_name));
    fs::write(
        &grammar_path,
        r#"%start CsTyped
%title "CsTyped grammar"
%comment "C# typed generation regression grammar"
%line_comment "//"

%%

CsTyped
    : Alt
    ;

Alt
    : HelloWorld
    | Number
    ;

HelloWorld
    : "Hello world!"
    ;

Number
    : "42"
    ;
"#,
    )?;

    // 3. Use local runtime if available
    ensure_local_runtime_reference(&project_path, project_name)?;

    // 4. Build and ensure generated sources compile
    let new_path = with_parol_path()?;
    let output = dotnet_build(&project_path, &new_path)?;
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "dotnet build failed");

    // 5. Inspect generated C# for typed API and mapping artifacts
    let generated_actions = fs::read_to_string(project_path.join("ICsTypedActions.cs"))?;
    assert!(
        generated_actions.contains("void OnCsTyped(CsTyped arg);")
            && generated_actions.contains("void OnAlt(Alt arg);")
    );
    assert!(
        generated_actions.contains("private static Alt MapAlt0(object[] children)")
            && generated_actions.contains("private static Alt MapAlt1(object[] children)")
    );

    let generated_parser = fs::read_to_string(project_path.join("CsTypedParser.cs"))?;
    assert!(generated_parser.contains(
        "public static void Parse(string input, string fileName, ICsTypedActions userActions)"
    ));

    Ok(())
}

#[test]
fn test_csharp_typed_generation_with_list_and_optional() -> Result<()> {
    if skip_if_no_dotnet("test_csharp_typed_generation_with_list_and_optional") {
        return Ok(());
    }

    let _guard = cs_runtime_build_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().join("cs_list_opt");
    let project_name = "cs_list_opt";

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

    // 2. Replace grammar with list + optional constructs
    let grammar_path = project_path.join(format!("{}.par", project_name));
    fs::write(
        &grammar_path,
        r#"%start CsListOpt
%title "CsListOpt grammar"
%comment "C# list/optional mapping regression grammar"
%line_comment "//"

%%

CsListOpt
    : Items MaybeHello
    ;

Items
    : { Number }
    ;

MaybeHello
    : [ HelloWorld ]
    ;

HelloWorld
    : "Hello world!"
    ;

Number
    : "42"
    ;
"#,
    )?;

    // 3. Use local runtime if available
    ensure_local_runtime_reference(&project_path, project_name)?;

    // 4. Build and ensure generated sources compile
    let new_path = with_parol_path()?;
    let output = dotnet_build(&project_path, &new_path)?;
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "dotnet build failed");

    // 5. Inspect generated C# for mapping logic patterns
    let generated_actions = fs::read_to_string(project_path.join("ICsListOptActions.cs"))?;
    assert!(
        generated_actions.contains("void OnItems(Items arg);")
            && generated_actions.contains("void OnMaybeHello(MaybeHello arg);")
    );

    // Repetition support in expanded grammar mapper types
    assert!(
        generated_actions
            .contains("private static List<ItemsList> MapItemsList0(object[] children)")
            && generated_actions
                .contains("private static List<ItemsList> MapItemsList1(object[] children)")
    );
    assert!(
        generated_actions.contains("new List<ItemsList>()")
            && generated_actions.contains("children.Length == 1 + 1")
    );

    // Optional support in expanded grammar mapper types
    assert!(
        generated_actions
            .contains("private static MaybeHelloOpt MapMaybeHelloOpt0(object[] children)")
            && generated_actions
                .contains("private static MaybeHelloOpt MapMaybeHelloOpt1(object[] children)")
    );
    assert!(
        generated_actions.contains("children.Length == 0")
            && generated_actions.contains("children.Length == 1")
    );

    Ok(())
}

#[test]
fn test_csharp_typed_generation_with_nt_type_conversion() -> Result<()> {
    if skip_if_no_dotnet("test_csharp_typed_generation_with_nt_type_conversion") {
        return Ok(());
    }

    let _guard = cs_runtime_build_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().join("cs_nt_type");
    let project_name = "cs_nt_type";

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

    // 2. Replace grammar with a non-terminal user type mapping
    let grammar_path = project_path.join(format!("{}.par", project_name));
    fs::write(
        &grammar_path,
        r#"%start CsNtType
%title "CsNtType grammar"
%comment "C# nt_type conversion regression grammar"
%line_comment "//"
%nt_type Value = CsNtType::CustomValue

%%

CsNtType
    : Value
    ;

Value
    : Number
    ;

Number
    : "42"
    ;
"#,
    )?;

    // 3. Add the custom C# user-defined type expected by %nt_type
    fs::write(
        project_path.join("CustomValue.cs"),
        r#"namespace CsNtType
{
    public sealed class CustomValue
    {
        public Value Value { get; }

        public CustomValue(Value value)
        {
            Value = value;
        }

        public override string ToString() => Value.ToString();
    }
}
"#,
    )?;

    // 4. Use local runtime if available
    ensure_local_runtime_reference(&project_path, project_name)?;

    // 5. Build and inspect generated mapping
    let new_path = with_parol_path()?;
    let output = dotnet_build(&project_path, &new_path)?;
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "dotnet build failed");

    let generated_actions = fs::read_to_string(project_path.join("ICsNtTypeActions.cs"))?;
    assert!(
        generated_actions.contains("ConvertValue<")
            && generated_actions.contains("MapValue")
            && generated_actions.contains("IProvidesValueConverter")
            && generated_actions.contains("IValueConverter ValueConverter")
            && generated_actions.contains("GeneratedValueConverter")
            && generated_actions.contains("RuntimeValueConverter.Convert")
    );
    assert!(
        generated_actions
            .contains("public sealed record CsNtType(global::CsNtType.CustomValue Value);")
            || generated_actions
                .contains("public sealed record CsNtType(CsNtType.CustomValue Value);")
            || generated_actions.contains("public sealed record CsNtType(CustomValue Value);")
    );
    assert!(
        generated_actions
            .contains("new CsNtType(ConvertValue<global::CsNtType.CustomValue>(children[0 + 0]))")
            || generated_actions
                .contains("new CsNtType(ConvertValue<CsNtType.CustomValue>(children[0 + 0]))")
            || generated_actions
                .contains("new CsNtType(ConvertValue<CustomValue>(children[0 + 0]))")
    );

    // 6. Run parse with matching input and verify success path
    fs::write(project_path.join("test.txt"), "42")?;
    let run_output = Command::new("dotnet")
        .current_dir(&project_path)
        .args(["run", "--", "test.txt"])
        .env("PATH", &new_path)
        .output()
        .map_err(|e| anyhow!(e))?;
    assert!(run_output.status.success(), "dotnet run failed");
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(
        stdout.contains("Success!"),
        "Expected success output, got: {stdout}"
    );

    Ok(())
}

#[test]
fn test_csharp_generated_actions_contains_converter_contract_markers() -> Result<()> {
    if skip_if_no_dotnet("test_csharp_generated_actions_contains_converter_contract_markers") {
        return Ok(());
    }

    let _guard = cs_runtime_build_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().join("cs_contract");
    let project_name = "cs_contract";

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

    // 2. Use local runtime if available
    ensure_local_runtime_reference(&project_path, project_name)?;

    // 3. Build to trigger generation
    let new_path = with_parol_path()?;
    let output = dotnet_build(&project_path, &new_path)?;
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "dotnet build failed");

    // 4. Snapshot critical generator markers for converter contract
    let generated_actions = fs::read_to_string(project_path.join("ICsContractActions.cs"))?;
    assert!(
        generated_actions.contains(
            "public interface ICsContractActions : IUserActions, IProvidesValueConverter {"
        )
    );
    assert!(generated_actions.contains(
        "public virtual IValueConverter ValueConverter { get; } = new GeneratedValueConverter();"
    ));
    assert!(
        generated_actions
            .contains("private sealed class GeneratedValueConverter : IValueConverter {")
    );
    assert!(
        generated_actions.contains("private static TTarget ConvertValue<TTarget>(object value) {")
    );
    assert!(generated_actions.contains("return RuntimeValueConverter.Convert<TTarget>(value);"));

    Ok(())
}

#[test]
fn test_csharp_t_type_terminal_conversion_is_applied() -> Result<()> {
    if skip_if_no_dotnet("test_csharp_t_type_terminal_conversion_is_applied") {
        return Ok(());
    }

    let _guard = cs_runtime_build_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().join("cs_t_type");
    let project_name = "cs_t_type";

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

    // 2. Replace grammar with a terminal user type mapping
    let grammar_path = project_path.join(format!("{}.par", project_name));
    fs::write(
        &grammar_path,
        r#"%start CsTType
%title "CsTType grammar"
%comment "C# t_type conversion regression grammar"
%line_comment "//"
%t_type CsTType::OwnedToken

%%

CsTType
    : Number
    ;

Number
    : "42"
    ;
"#,
    )?;

    // 3. Add the custom C# user-defined terminal type expected by %t_type
    fs::write(
        project_path.join("OwnedToken.cs"),
        r#"using Parol.Runtime;
using Parol.Runtime.Scanner;

namespace CsTType
{
    public sealed class OwnedToken
    {
        public Token Token { get; }

        public OwnedToken(Token token)
        {
            Token = token;
        }

        public override string ToString() => Token.ToString();
    }
}
"#,
    )?;

    // 4. Use local runtime if available
    ensure_local_runtime_reference(&project_path, project_name)?;

    // 5. Build and inspect generated mapping
    let new_path = with_parol_path()?;
    let output = dotnet_build(&project_path, &new_path)?;
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "dotnet build failed");

    let generated_actions = fs::read_to_string(project_path.join("ICsTTypeActions.cs"))?;
    assert!(
        generated_actions.contains("ConvertValue<")
            && generated_actions.contains("MapNumber")
            && generated_actions.contains("MapCsTType")
    );
    assert!(
        generated_actions
            .contains("new Number(ConvertValue<global::CsTType.OwnedToken>(children[0 + 0]))")
            || generated_actions
                .contains("new Number(ConvertValue<CsTType.OwnedToken>(children[0 + 0]))")
            || generated_actions.contains("new Number(ConvertValue<OwnedToken>(children[0 + 0]))")
    );

    // 6. Run parse with matching input and verify success path
    fs::write(project_path.join("test.txt"), "42")?;
    let run_output = Command::new("dotnet")
        .current_dir(&project_path)
        .args(["run", "--", "test.txt"])
        .env("PATH", &new_path)
        .output()
        .map_err(|e| anyhow!(e))?;
    assert!(run_output.status.success(), "dotnet run failed");

    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(
        stdout.contains("Success!"),
        "Expected success output, got: {stdout}"
    );

    Ok(())
}

#[test]
fn test_csharp_t_type_null_input_conversion_is_rejected() -> Result<()> {
    if skip_if_no_dotnet("test_csharp_t_type_null_input_conversion_is_rejected") {
        return Ok(());
    }

    let _guard = cs_runtime_build_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().join("cs_t_type_fail");
    let project_name = "cs_t_type_fail";

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

    // 2. Replace grammar with a terminal user type mapping
    let grammar_path = project_path.join(format!("{}.par", project_name));
    fs::write(
        &grammar_path,
        r#"%start CsTTypeFail
%title "CsTTypeFail grammar"
%comment "C# t_type negative conversion regression grammar"
%line_comment "//"
%t_type CsTTypeFail::OwnedToken

%%

CsTTypeFail
    : Number
    ;

Number
    : "42"
    ;
"#,
    )?;

    // 3. Add a terminal type used by %t_type
    fs::write(
        project_path.join("OwnedToken.cs"),
        r#"using Parol.Runtime;
    using Parol.Runtime.Scanner;

namespace CsTTypeFail
{
    public sealed class OwnedToken
    {
        public Token Token { get; }

        public OwnedToken(Token token)
        {
            Token = token;
        }
    }
}
"#,
    )?;

    // 4. Add a custom action type that checks null-input conversion handling
    fs::write(
        project_path.join("CustomActions.cs"),
        r#"using System;
using Parol.Runtime;

namespace CsTTypeFail
{
    public sealed class CustomActions : CsTTypeFailActions
    {
        public CustomActions()
        {
            try
            {
                RuntimeValueConverter.Convert<OwnedToken>(null!);
                throw new InvalidOperationException("Expected InvalidCastException for null conversion input");
            }
            catch (InvalidCastException)
            {
            }
        }
    }
}
"#,
    )?;

    let program_path = project_path.join("Program.cs");
    let program_content = fs::read_to_string(&program_path)?;
    fs::write(
        &program_path,
        program_content.replace(
            "ICsTTypeFailActions actions = new CsTTypeFailActions();",
            "ICsTTypeFailActions actions = new CustomActions();",
        ),
    )?;

    // 5. Use local runtime if available
    ensure_local_runtime_reference(&project_path, project_name)?;

    // 6. Build should succeed
    let new_path = with_parol_path()?;
    let output = dotnet_build(&project_path, &new_path)?;
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "dotnet build failed");

    let generated_actions = fs::read_to_string(project_path.join("ICsTTypeFailActions.cs"))?;
    assert!(
        generated_actions.contains("ConvertValue<")
            && generated_actions.contains("MapNumber")
            && generated_actions.contains("MapCsTTypeFail")
    );
    assert!(
        generated_actions
            .contains("new Number(ConvertValue<global::CsTTypeFail.OwnedToken>(children[0 + 0]))")
            || generated_actions
                .contains("new Number(ConvertValue<CsTTypeFail.OwnedToken>(children[0 + 0]))")
            || generated_actions.contains("new Number(ConvertValue<OwnedToken>(children[0 + 0]))")
    );

    // 7. Run parse and verify success (null-input check executes in CustomActions ctor)
    fs::write(project_path.join("test.txt"), "42")?;
    let run_output = Command::new("dotnet")
        .current_dir(&project_path)
        .args(["run", "--", "test.txt"])
        .env("PATH", &new_path)
        .output()
        .map_err(|e| anyhow!(e))?;
    assert!(run_output.status.success(), "dotnet run failed");

    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(
        stdout.contains("Success!"),
        "Expected success output, got: {stdout}"
    );

    Ok(())
}

#[test]
fn test_csharp_nt_type_custom_value_converter_override_is_used() -> Result<()> {
    if skip_if_no_dotnet("test_csharp_nt_type_custom_value_converter_override_is_used") {
        return Ok(());
    }

    let _guard = cs_runtime_build_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().join("cs_nt_type_override");
    let project_name = "cs_nt_type_override";

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

    // 2. Replace grammar with a non-terminal user type mapping
    let grammar_path = project_path.join(format!("{}.par", project_name));
    fs::write(
        &grammar_path,
        r#"%start CsNtTypeOverride
%title "CsNtTypeOverride grammar"
%comment "C# custom value converter precedence regression grammar"
%line_comment "//"
%nt_type Value = CsNtTypeOverride::CustomValue

%%

CsNtTypeOverride
    : Value
    ;

Value
    : Number
    ;

Number
    : "42"
    ;
"#,
    )?;

    // 3. Add a custom C# user-defined type without constructor/operator fallback from Number
    fs::write(
        project_path.join("CustomValue.cs"),
        r#"namespace CsNtTypeOverride
{
    public sealed class CustomValue
    {
        public string Marker { get; }

        public CustomValue(string marker)
        {
            Marker = marker;
        }

        public override string ToString() => Marker;
    }
}
"#,
    )?;

    // 4. Add a derived action type with custom converter and wire Program.cs to use it
    fs::write(
        project_path.join("CustomActions.cs"),
        r#"using System;
using Parol.Runtime;

namespace CsNtTypeOverride
{
    public sealed class CustomActions : CsNtTypeOverrideActions
    {
        public override IValueConverter ValueConverter { get; } = new OverrideConverter();

        private sealed class OverrideConverter : IValueConverter
        {
            public bool TryConvert(object value, Type targetType, out object? convertedValue)
            {
                if (targetType == typeof(CustomValue) && value is Value)
                {
                    convertedValue = new CustomValue("override");
                    return true;
                }

                convertedValue = null;
                return false;
            }
        }
    }
}
"#,
    )?;

    let program_path = project_path.join("Program.cs");
    let program_content = fs::read_to_string(&program_path)?;
    fs::write(
        &program_path,
        program_content.replace(
            "ICsNtTypeOverrideActions actions = new CsNtTypeOverrideActions();",
            "ICsNtTypeOverrideActions actions = new CustomActions();",
        ),
    )?;

    // 5. Use local runtime if available
    ensure_local_runtime_reference(&project_path, project_name)?;

    // 6. Build and run
    let new_path = with_parol_path()?;
    let output = dotnet_build(&project_path, &new_path)?;
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "dotnet build failed");

    fs::write(project_path.join("test.txt"), "42")?;
    let run_output = Command::new("dotnet")
        .current_dir(&project_path)
        .args(["run", "--", "test.txt"])
        .env("PATH", &new_path)
        .output()
        .map_err(|e| anyhow!(e))?;
    assert!(run_output.status.success(), "dotnet run failed");

    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(
        stdout.contains("Success!"),
        "Expected success output, got: {stdout}"
    );

    Ok(())
}

#[test]
fn test_csharp_nt_type_null_input_conversion_is_rejected() -> Result<()> {
    if skip_if_no_dotnet("test_csharp_nt_type_null_input_conversion_is_rejected") {
        return Ok(());
    }

    let _guard = cs_runtime_build_lock()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let temp_dir = tempdir()?;
    let project_path = temp_dir.path().join("cs_nt_type_null");
    let project_name = "cs_nt_type_null";

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

    // 2. Replace grammar with a non-terminal user type mapping
    let grammar_path = project_path.join(format!("{}.par", project_name));
    fs::write(
        &grammar_path,
        r#"%start CsNtTypeNull
%title "CsNtTypeNull grammar"
%comment "C# null conversion handling regression grammar"
%line_comment "//"
%nt_type Value = CsNtTypeNull::CustomValue

%%

CsNtTypeNull
    : Value
    ;

Value
    : Number
    ;

Number
    : "42"
    ;
"#,
    )?;

    // 3. Add a custom type with constructor fallback from Number
    fs::write(
        project_path.join("CustomValue.cs"),
        r#"namespace CsNtTypeNull
{
    public sealed class CustomValue
    {
        public Value Value { get; }

        public CustomValue(Value value)
        {
            Value = value;
        }
    }
}
"#,
    )?;

    // 4. Add a derived action type that checks null input handling in RuntimeValueConverter
    fs::write(
        project_path.join("CustomActions.cs"),
        r#"using System;
using Parol.Runtime;

namespace CsNtTypeNull
{
    public sealed class CustomActions : CsNtTypeNullActions
    {
        public CustomActions()
        {
            try
            {
                RuntimeValueConverter.Convert<CustomValue>(null!);
                throw new InvalidOperationException("Expected InvalidCastException for null conversion input");
            }
            catch (InvalidCastException)
            {
            }
        }
    }
}
"#,
    )?;

    let program_path = project_path.join("Program.cs");
    let program_content = fs::read_to_string(&program_path)?;
    fs::write(
        &program_path,
        program_content.replace(
            "ICsNtTypeNullActions actions = new CsNtTypeNullActions();",
            "ICsNtTypeNullActions actions = new CustomActions();",
        ),
    )?;

    // 5. Use local runtime if available
    ensure_local_runtime_reference(&project_path, project_name)?;

    // 6. Build and run (should succeed; null-input check is executed in CustomActions ctor)
    let new_path = with_parol_path()?;
    let output = dotnet_build(&project_path, &new_path)?;
    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success(), "dotnet build failed");

    fs::write(project_path.join("test.txt"), "42")?;
    let run_output = Command::new("dotnet")
        .current_dir(&project_path)
        .args(["run", "--", "test.txt"])
        .env("PATH", &new_path)
        .output()
        .map_err(|e| anyhow!(e))?;
    assert!(run_output.status.success(), "dotnet run failed");

    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(
        stdout.contains("Success!"),
        "Expected success output, got: {stdout}"
    );

    Ok(())
}
