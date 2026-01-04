//! Integration tests for wix-diagnostics CLI

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn get_binary_path() -> String {
    let release = "target/release/wix-diagnostics";
    let debug = "target/debug/wix-diagnostics";

    if std::path::Path::new(release).exists() {
        release.to_string()
    } else {
        debug.to_string()
    }
}

#[test]
fn test_valid_file() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.wxs");

    fs::write(
        &file_path,
        r#"<Wix>
    <Directory Id="TARGETDIR">
        <Component Id="C1" Guid="*">
            <File Id="F1" Source="test.exe" />
        </Component>
    </Directory>
    <Feature Id="Main">
        <ComponentRef Id="C1" />
    </Feature>
</Wix>"#,
    )
    .unwrap();

    let output = Command::new(&binary)
        .args([file_path.to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0 error(s)"));
}

#[test]
fn test_invalid_reference() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><ComponentRef Id=\"Missing\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No Component"));
    assert!(stdout.contains("1 error(s)"));
}

#[test]
fn test_invalid_parent() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Directory Id=\"D1\"><File Id=\"F1\" /></Directory></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cannot be a child of"));
}

#[test]
fn test_invalid_guid() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Component Guid=\"not-a-guid\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Invalid GUID"));
}

#[test]
fn test_json_output() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--format", "json", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><ComponentRef Id=\"Missing\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"diagnostics\""));
    assert!(stdout.contains("\"severity\""));
}

#[test]
fn test_cross_file_validation() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();

    // Definition file
    let def_path = temp.path().join("defs.wxs");
    fs::write(&def_path, r#"<Wix><Component Id="SharedComp" /></Wix>"#).unwrap();

    // Reference file
    let ref_path = temp.path().join("refs.wxs");
    fs::write(
        &ref_path,
        r#"<Wix><ComponentRef Id="SharedComp" /></Wix>"#,
    )
    .unwrap();

    let output = Command::new(&binary)
        .args([
            ref_path.to_str().unwrap(),
            "--include",
            def_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0 error(s)"));
}

#[test]
fn test_validators_filter() {
    let binary = get_binary_path();

    // Test with only references validator - should catch missing reference
    let mut child = Command::new(&binary)
        .args(["--validators", "references", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><ComponentRef Id=\"Missing\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1 error(s)"));
}

#[test]
fn test_errors_only() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--errors-only", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Wix />").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");
    assert!(output.status.success());
}

#[test]
fn test_help() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["--help"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wix-diagnostics"));
    assert!(stdout.contains("--format"));
    assert!(stdout.contains("--validators"));
}

#[test]
fn test_version() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["--version"])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wix-diagnostics"));
}

#[test]
fn test_missing_required_attribute() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Directory Name=\"Test\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("requires 'Id'"));
}

#[test]
fn test_invalid_enum_value() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Feature Id=\"F1\" Display=\"invalid\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Invalid value"));
}

#[test]
fn test_project_directory() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();

    // Create multiple files in project
    fs::write(
        temp.path().join("defs.wxs"),
        r#"<Wix><Component Id="C1" /></Wix>"#,
    )
    .unwrap();

    let ref_path = temp.path().join("refs.wxs");
    fs::write(&ref_path, r#"<Wix><ComponentRef Id="C1" /></Wix>"#).unwrap();

    let output = Command::new(&binary)
        .args([
            ref_path.to_str().unwrap(),
            "--project",
            temp.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0 error(s)"));
}
