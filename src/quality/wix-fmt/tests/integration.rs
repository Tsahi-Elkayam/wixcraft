//! Integration tests for wix-fmt CLI

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn get_binary_path() -> String {
    let release = "target/release/wix-fmt";
    let debug = "target/debug/wix-fmt";

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

    let package = r#"{
        "name": "Package",
        "children": ["Directory", "Component", "Feature"],
        "attributes": {
            "Name": {"type": "string", "required": true},
            "Version": {"type": "version", "required": true}
        }
    }"#;
    fs::write(elements_dir.join("package.json"), package).unwrap();

    temp
}

#[test]
fn test_format_stdin() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Package><Component /></Package>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<Package>"));
    assert!(stdout.contains("</Package>"));
}

#[test]
fn test_format_file() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.wxs");

    fs::write(&file_path, "<Package><Component /></Package>").unwrap();

    let output = Command::new(&binary)
        .args([file_path.to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<Package>"));
}

#[test]
fn test_check_mode_formatted() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.wxs");

    // Already formatted content
    fs::write(&file_path, "<Package />\n").unwrap();

    let output = Command::new(&binary)
        .args(["--check", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
}

#[test]
fn test_check_mode_unformatted() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.wxs");

    // Unformatted content
    fs::write(&file_path, "<Package><Component></Component></Package>").unwrap();

    let output = Command::new(&binary)
        .args(["--check", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    // Should fail because file needs formatting
    assert!(!output.status.success());
}

#[test]
fn test_write_mode() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.wxs");

    fs::write(&file_path, "<Package><Component /></Package>").unwrap();

    let output = Command::new(&binary)
        .args(["--write", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());

    // File should be modified
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains('\n')); // Should have newlines now
}

#[test]
fn test_output_file() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();
    let input_path = temp.path().join("input.wxs");
    let output_path = temp.path().join("output.wxs");

    fs::write(&input_path, "<Package />").unwrap();

    let output = Command::new(&binary)
        .args([
            "--output",
            output_path.to_str().unwrap(),
            input_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    assert!(output_path.exists());
}

#[test]
fn test_indent_options() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--indent-style", "tab", "--indent-size", "1", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Package><Component /></Package>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\t<Component"));
}

#[test]
fn test_sort_attributes() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--sort-attributes", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Component Zebra=\"z\" Id=\"test\" />")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Id should come before Zebra
    let id_pos = stdout.find("Id=").unwrap();
    let zebra_pos = stdout.find("Zebra=").unwrap();
    assert!(id_pos < zebra_pos);
}

#[test]
fn test_with_wix_data() {
    let binary = get_binary_path();
    let wix_data = create_test_wix_data();

    let mut child = Command::new(&binary)
        .args([
            "--wix-data",
            wix_data.path().to_str().unwrap(),
            "--verbose",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Package />").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Loaded wix-data"));
}

#[test]
fn test_verbose_mode() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.wxs");

    fs::write(&file_path, "<Package />").unwrap();

    let output = Command::new(&binary)
        .args(["--verbose", "--write", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Configuration:"));
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
    assert!(stdout.contains("wix-fmt"));
    assert!(stdout.contains("--check"));
    assert!(stdout.contains("--write"));
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
    assert!(stdout.contains("wix-fmt"));
}

#[test]
fn test_invalid_xml() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"<Package><Unclosed>").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(!output.status.success());
}

#[test]
fn test_file_not_found() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["/nonexistent/file.wxs"])
        .output()
        .expect("Failed to run command");

    assert!(!output.status.success());
}
