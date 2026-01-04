//! Integration tests for wix-best-practices CLI

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn get_binary_path() -> String {
    let release = "target/release/wix-best-practices";
    let debug = "target/debug/wix-best-practices";

    if std::path::Path::new(release).exists() {
        release.to_string()
    } else {
        debug.to_string()
    }
}

#[test]
fn test_well_formed_file() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.wxs");

    fs::write(
        &file_path,
        r#"<Wix>
    <Package Name="Test" Version="1.0" UpgradeCode="{12345678-1234-1234-1234-123456789ABC}">
        <MajorUpgrade DowngradeErrorMessage="A newer version is installed." />
    </Package>
    <Directory Id="TARGETDIR">
        <Component Id="C_Main" Guid="*">
            <File Id="F1" Source="app.exe" />
        </Component>
    </Directory>
    <Feature Id="F_Main">
        <ComponentRef Id="C_Main" />
    </Feature>
</Wix>"#,
    )
    .unwrap();

    let output = Command::new(&binary)
        .args([file_path.to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
}

#[test]
fn test_missing_major_upgrade() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--show-rules", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Package Name=\"Test\" Version=\"1.0\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BP-IDIOM-001"));
    assert!(stdout.contains("Missing MajorUpgrade"));
}

#[test]
fn test_hardcoded_guid() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--show-rules", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Component Id=\"C1\" Guid=\"{12345678-1234-1234-1234-123456789ABC}\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BP-IDIOM-002"));
}

#[test]
fn test_unused_component() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--show-rules", "--categories", "efficiency", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Component Id=\"UnusedComp\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BP-EFF-002"));
    assert!(stdout.contains("Unused Component"));
}

#[test]
fn test_category_filter() {
    let binary = get_binary_path();

    // Only run idiom checks
    let mut child = Command::new(&binary)
        .args(["--show-rules", "--categories", "idiom", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Package Name=\"Test\" /><Component Id=\"UnusedComp\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have idiom issues
    assert!(stdout.contains("idiom"));

    // Should not have efficiency issues (unused component)
    assert!(!stdout.contains("BP-EFF-002"));
}

#[test]
fn test_min_impact_filter() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--show-rules", "--min-impact", "high", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Package Name=\"Test\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should only have HIGH impact issues
    assert!(stdout.contains("HIGH"));
    assert!(!stdout.contains("LOW"));
    assert!(!stdout.contains("MEDIUM"));
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
            .write_all(b"<Wix><Package Name=\"Test\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"suggestions\""));
    assert!(stdout.contains("\"rule_id\""));
}

#[test]
fn test_directory_analysis() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();

    // Create multiple WiX files
    fs::write(
        temp.path().join("file1.wxs"),
        r#"<Wix><Package Name="Test1" /></Wix>"#,
    )
    .unwrap();

    fs::write(
        temp.path().join("file2.wxs"),
        r#"<Wix><Package Name="Test2" /></Wix>"#,
    )
    .unwrap();

    let output = Command::new(&binary)
        .args([temp.path().to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 file(s) analyzed"));
}

#[test]
fn test_verbose_output() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--verbose", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Package Name=\"Test\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verbose should include detailed messages
    assert!(stdout.contains("Add"));
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
    assert!(stdout.contains("wix-best-practices"));
    assert!(stdout.contains("--categories"));
    assert!(stdout.contains("--min-impact"));
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
    assert!(stdout.contains("wix-best-practices"));
}

#[test]
fn test_hardcoded_path() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--show-rules", "--categories", "maintainability", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><File Id=\"F1\" Source=\"C:\\Build\\app.exe\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BP-MAINT-001"));
}

#[test]
fn test_multi_file_component() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--show-rules", "--categories", "performance", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Component Id=\"C1\"><File Id=\"F1\" /><File Id=\"F2\" /></Component></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BP-PERF-001"));
    assert!(stdout.contains("Multi-file Component"));
}
