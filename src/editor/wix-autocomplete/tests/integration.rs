//! Integration tests for wix-autocomplete CLI

use std::process::{Command, Stdio};
use std::io::Write;
use std::fs;
use tempfile::TempDir;

fn create_test_wix_data() -> TempDir {
    let temp = TempDir::new().unwrap();

    // Create elements directory
    let elements_dir = temp.path().join("elements");
    fs::create_dir(&elements_dir).unwrap();

    let package = r#"{
        "name": "Package",
        "description": "Root package",
        "parents": ["Wix"],
        "children": ["Component", "Directory"],
        "attributes": {
            "Name": {"type": "string", "required": true}
        }
    }"#;
    fs::write(elements_dir.join("package.json"), package).unwrap();

    let component = r#"{
        "name": "Component",
        "description": "Component",
        "parents": ["Package", "Directory"],
        "children": ["File"],
        "attributes": {
            "Guid": {"type": "guid", "required": true}
        }
    }"#;
    fs::write(elements_dir.join("component.json"), component).unwrap();

    // Create keywords
    let keywords_dir = temp.path().join("keywords");
    fs::create_dir(&keywords_dir).unwrap();
    fs::write(
        keywords_dir.join("keywords.json"),
        r#"{"standardDirectories": ["ProgramFilesFolder"], "builtinProperties": [], "elements": [], "preprocessorDirectives": []}"#,
    ).unwrap();

    // Create snippets
    let snippets_dir = temp.path().join("snippets");
    fs::create_dir(&snippets_dir).unwrap();
    fs::write(snippets_dir.join("snippets.json"), r#"{"snippets": []}"#).unwrap();

    temp
}

fn get_binary_path() -> String {
    // Try release first, then debug
    let release = "target/release/wix-autocomplete";
    let debug = "target/debug/wix-autocomplete";

    if std::path::Path::new(release).exists() {
        release.to_string()
    } else {
        debug.to_string()
    }
}

#[test]
fn test_cli_json_output() {
    let wix_data = create_test_wix_data();
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data", wix_data.path().to_str().unwrap(),
            "-", "2", "4"
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Package>\n  <").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("\"label\""));
    assert!(stdout.contains("\"kind\""));
}

#[test]
fn test_cli_plain_output() {
    let wix_data = create_test_wix_data();
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data", wix_data.path().to_str().unwrap(),
            "--format", "plain",
            "-", "2", "4"
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Package>\n  <").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    // Plain output format: label kind detail
    assert!(stdout.contains("Element") || stdout.contains("Package") || stdout.contains("Component"));
}

#[test]
fn test_cli_verbose_mode() {
    let wix_data = create_test_wix_data();
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data", wix_data.path().to_str().unwrap(),
            "--verbose",
            "-", "2", "4"
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Package>\n  <").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success());
    // Verbose mode prints to stderr
    assert!(stderr.contains("Loaded"));
    assert!(stderr.contains("elements"));
}

#[test]
fn test_cli_limit() {
    let wix_data = create_test_wix_data();
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data", wix_data.path().to_str().unwrap(),
            "--limit", "1",
            "-", "2", "4"
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Package>\n  <").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    // With limit 1, should have only one item in JSON array
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.len() <= 1);
}

#[test]
fn test_cli_file_input() {
    let wix_data = create_test_wix_data();
    let binary = get_binary_path();

    // Create a temp file with WiX content
    let temp_file = TempDir::new().unwrap();
    let wix_file = temp_file.path().join("test.wxs");
    fs::write(&wix_file, "<Package>\n  <\n</Package>").unwrap();

    let output = Command::new(&binary)
        .args([
            "--wix-data", wix_data.path().to_str().unwrap(),
            wix_file.to_str().unwrap(),
            "2", "4"
        ])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("Component") || stdout.contains("Directory"));
}

#[test]
fn test_cli_element_completions() {
    let wix_data = create_test_wix_data();
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data", wix_data.path().to_str().unwrap(),
            "-", "2", "4"
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Package>\n  <").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("Component"));
}

#[test]
fn test_cli_attribute_completions() {
    let wix_data = create_test_wix_data();
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data", wix_data.path().to_str().unwrap(),
            "-", "1", "12"
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Component ").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("Guid"));
}

#[test]
fn test_cli_invalid_wix_data_path() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data", "/nonexistent/path",
            "-", "2", "4"
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Package>\n  <").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error"));
}

#[test]
fn test_cli_help() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["--help"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("wix-autocomplete"));
    assert!(stdout.contains("--wix-data"));
    assert!(stdout.contains("--format"));
}

#[test]
fn test_cli_version() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["--version"])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("wix-autocomplete"));
}
