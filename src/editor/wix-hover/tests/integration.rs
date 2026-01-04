//! Integration tests for wix-hover CLI

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn get_binary_path() -> String {
    let release = "target/release/wix-hover";
    let debug = "target/debug/wix-hover";

    if std::path::Path::new(release).exists() {
        release.to_string()
    } else {
        debug.to_string()
    }
}

fn create_test_wix_data() -> TempDir {
    let temp = TempDir::new().unwrap();

    let elements_dir = temp.path().join("elements");
    fs::create_dir(&elements_dir).unwrap();

    let component = r#"{
        "name": "Component",
        "description": "A component is a grouping of resources.",
        "documentation": "https://wixtoolset.org/docs/schema/wxs/component/",
        "since": "v3",
        "parents": ["Directory"],
        "children": ["File"],
        "attributes": {
            "Guid": {"type": "guid", "required": true, "description": "Component GUID"}
        }
    }"#;
    fs::write(elements_dir.join("component.json"), component).unwrap();

    let keywords_dir = temp.path().join("keywords");
    fs::create_dir(&keywords_dir).unwrap();
    let keywords = r#"{
        "standardDirectories": ["ProgramFilesFolder"],
        "builtinProperties": ["ProductName"],
        "elements": [],
        "preprocessorDirectives": []
    }"#;
    fs::write(keywords_dir.join("keywords.json"), keywords).unwrap();

    temp
}

#[test]
fn test_hover_element_from_file() {
    let binary = get_binary_path();
    let wix_data = create_test_wix_data();

    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.wxs");
    fs::write(&file_path, "<Component Guid=\"*\" />").unwrap();

    let output = Command::new(&binary)
        .args([
            "--wix-data",
            wix_data.path().to_str().unwrap(),
            file_path.to_str().unwrap(),
            "1",
            "3",
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Component"));
    assert!(stdout.contains("grouping of resources"));
}

#[test]
fn test_hover_element_from_stdin() {
    let binary = get_binary_path();
    let wix_data = create_test_wix_data();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data",
            wix_data.path().to_str().unwrap(),
            "-",
            "1",
            "3",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Component Guid=\"*\" />").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Component"));
}

#[test]
fn test_hover_attribute() {
    let binary = get_binary_path();
    let wix_data = create_test_wix_data();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data",
            wix_data.path().to_str().unwrap(),
            "-",
            "1",
            "13",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Component Guid=\"*\" />").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Guid"));
    assert!(stdout.contains("GUID"));
}

#[test]
fn test_hover_json_format() {
    let binary = get_binary_path();
    let wix_data = create_test_wix_data();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data",
            wix_data.path().to_str().unwrap(),
            "--format",
            "json",
            "-",
            "1",
            "3",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Component />").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"contents\""));
    assert!(stdout.contains("\"range\""));
}

#[test]
fn test_hover_plain_format() {
    let binary = get_binary_path();
    let wix_data = create_test_wix_data();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data",
            wix_data.path().to_str().unwrap(),
            "--format",
            "plain",
            "-",
            "1",
            "3",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Component />").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Plain format should not have markdown
    assert!(!stdout.contains("###"));
    assert!(!stdout.contains("**"));
}

#[test]
fn test_hover_verbose() {
    let binary = get_binary_path();
    let wix_data = create_test_wix_data();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data",
            wix_data.path().to_str().unwrap(),
            "--verbose",
            "-",
            "1",
            "3",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Component />").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Loaded wix-data"));
    assert!(stderr.contains("Position:"));
}

#[test]
fn test_hover_no_result() {
    let binary = get_binary_path();
    let wix_data = create_test_wix_data();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data",
            wix_data.path().to_str().unwrap(),
            "-",
            "1",
            "1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"   ").unwrap(); // Just whitespace
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty() || stdout.trim().is_empty());
}

#[test]
fn test_hover_unknown_element() {
    let binary = get_binary_path();
    let wix_data = create_test_wix_data();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data",
            wix_data.path().to_str().unwrap(),
            "-",
            "1",
            "3",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Unknown />").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    // No output for unknown element
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty() || stdout.trim().is_empty());
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
    assert!(stdout.contains("wix-hover"));
    assert!(stdout.contains("--wix-data"));
    assert!(stdout.contains("--format"));
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
    assert!(stdout.contains("wix-hover"));
}

#[test]
fn test_missing_wix_data() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["-", "1", "1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<X />").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--wix-data is required"));
}

#[test]
fn test_file_not_found() {
    let binary = get_binary_path();
    let wix_data = create_test_wix_data();

    let output = Command::new(&binary)
        .args([
            "--wix-data",
            wix_data.path().to_str().unwrap(),
            "/nonexistent/file.wxs",
            "1",
            "1",
        ])
        .output()
        .expect("Failed to run command");

    assert!(!output.status.success());
}
