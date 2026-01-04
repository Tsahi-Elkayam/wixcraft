//! Integration tests for wix-references CLI

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn get_binary_path() -> String {
    let release = "target/release/wix-references";
    let debug = "target/debug/wix-references";

    if std::path::Path::new(release).exists() {
        release.to_string()
    } else {
        debug.to_string()
    }
}

#[test]
fn test_go_to_definition_from_ref() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.wxs");

    fs::write(
        &file_path,
        r#"<Wix><Component Id="MainComp" /><ComponentRef Id="MainComp" /></Wix>"#,
    )
    .unwrap();

    let output = Command::new(&binary)
        .args([
            "definition",
            file_path.to_str().unwrap(),
            "--line",
            "1",
            "--column",
            "50",
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MainComp"));
    assert!(stdout.contains("Component"));
}

#[test]
fn test_go_to_definition_from_stdin() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["definition", "-", "--line", "1", "--column", "50"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(
                b"<Wix><Component Id=\"MainComp\" /><ComponentRef Id=\"MainComp\" /></Wix>",
            )
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MainComp"));
}

#[test]
fn test_go_to_definition_missing() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["definition", "-", "--line", "1", "--column", "25"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><ComponentRef Id=\"MissingComp\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No") || stdout.contains("not found"));
}

#[test]
fn test_find_references() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["references", "-", "--line", "1", "--column", "20"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(
                b"<Wix><Component Id=\"Comp1\" /><ComponentRef Id=\"Comp1\" /><ComponentRef Id=\"Comp1\" /></Wix>",
            )
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 reference(s)"));
}

#[test]
fn test_find_references_include_definition() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args([
            "references",
            "-",
            "--line",
            "1",
            "--column",
            "20",
            "--include-definition",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Component Id=\"Comp1\" /><ComponentRef Id=\"Comp1\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[definition]"));
}

#[test]
fn test_json_output() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args([
            "definition",
            "-",
            "--line",
            "1",
            "--column",
            "50",
            "--format",
            "json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(
                b"<Wix><Component Id=\"MainComp\" /><ComponentRef Id=\"MainComp\" /></Wix>",
            )
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"definition\""));
    assert!(stdout.contains("\"id\""));
}

#[test]
fn test_cross_file_references() {
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
            "definition",
            ref_path.to_str().unwrap(),
            "--line",
            "1",
            "--column",
            "25",
            "--include",
            def_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("defs.wxs"));
    assert!(stdout.contains("SharedComp"));
}

#[test]
fn test_index_directory() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();

    fs::write(
        temp.path().join("file1.wxs"),
        r#"<Wix><Component Id="C1" /></Wix>"#,
    )
    .unwrap();
    fs::write(
        temp.path().join("file2.wxs"),
        r#"<Wix><Component Id="C2" /><ComponentRef Id="C1" /></Wix>"#,
    )
    .unwrap();

    let output = Command::new(&binary)
        .args(["index", temp.path().to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 files"));
    assert!(stdout.contains("definitions"));
    assert!(stdout.contains("references"));
}

#[test]
fn test_find_definition_by_id() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();

    let file_path = temp.path().join("test.wxs");
    fs::write(&file_path, r#"<Wix><Component Id="Target" /></Wix>"#).unwrap();

    let output = Command::new(&binary)
        .args([
            "find-definition",
            "-t",
            "Component",
            "-i",
            "Target",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test.wxs"));
}

#[test]
fn test_find_references_by_id() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();

    let file_path = temp.path().join("test.wxs");
    fs::write(
        &file_path,
        r#"<Wix><Component Id="C1" /><ComponentRef Id="C1" /><ComponentRef Id="C1" /></Wix>"#,
    )
    .unwrap();

    let output = Command::new(&binary)
        .args([
            "find-references",
            "-t",
            "Component",
            "-i",
            "C1",
            file_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 reference(s)"));
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
    assert!(stdout.contains("wix-references"));
    assert!(stdout.contains("definition"));
    assert!(stdout.contains("references"));
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
    assert!(stdout.contains("wix-references"));
}

#[test]
fn test_directory_ref() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["definition", "-", "--line", "1", "--column", "55"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(
                b"<Wix><Directory Id=\"TARGETDIR\" /><DirectoryRef Id=\"TARGETDIR\" /></Wix>",
            )
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TARGETDIR"));
}

#[test]
fn test_feature_ref() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["definition", "-", "--line", "1", "--column", "50"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Feature Id=\"MainFeature\" /><FeatureRef Id=\"MainFeature\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MainFeature"));
}

#[test]
fn test_missing_definitions_in_index() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();

    fs::write(
        temp.path().join("test.wxs"),
        r#"<Wix><ComponentRef Id="MissingComp" /></Wix>"#,
    )
    .unwrap();

    let output = Command::new(&binary)
        .args(["index", temp.path().to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Missing"));
    assert!(stdout.contains("MissingComp"));
}
