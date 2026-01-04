//! Integration tests for wix-symbols CLI

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn get_binary_path() -> String {
    let release = "target/release/wix-symbols";
    let debug = "target/debug/wix-symbols";

    if std::path::Path::new(release).exists() {
        release.to_string()
    } else {
        debug.to_string()
    }
}

#[test]
fn test_extract_from_file() {
    let binary = get_binary_path();
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.wxs");

    fs::write(&file_path, r#"<Wix><Component Id="TestComp" Guid="*" /></Wix>"#).unwrap();

    let output = Command::new(&binary)
        .args([file_path.to_str().unwrap()])
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Component: TestComp"));
}

#[test]
fn test_extract_from_stdin() {
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
            .write_all(b"<Wix><Component Id=\"StdinComp\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Component: StdinComp"));
}

#[test]
fn test_json_format() {
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
            .write_all(b"<Wix><Component Id=\"JsonComp\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"name\": \"JsonComp\""));
    assert!(stdout.contains("\"kind\":"));
}

#[test]
fn test_flat_mode() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--flat", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(
                b"<Wix><Directory Id=\"TARGETDIR\"><Component Id=\"Comp1\" /></Directory></Wix>",
            )
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // In flat mode, both should appear at same level (no indentation hierarchy)
    assert!(stdout.contains("TARGETDIR"));
    assert!(stdout.contains("Comp1"));
}

#[test]
fn test_query_filter() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--query", "main", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(
                b"<Wix>
                    <Component Id=\"MainComponent\" />
                    <Component Id=\"OtherComponent\" />
                    <Feature Id=\"MainFeature\" />
                </Wix>",
            )
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MainComponent"));
    assert!(stdout.contains("MainFeature"));
    assert!(!stdout.contains("OtherComponent"));
}

#[test]
fn test_verbose_output() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--verbose", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(b"<Wix><Component Id=\"Test\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Parsing source"));
    assert!(stderr.contains("Found"));
}

#[test]
fn test_nested_hierarchy() {
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
            .write_all(
                b"<Wix>
                    <Directory Id=\"TARGETDIR\">
                        <Directory Id=\"INSTALLFOLDER\" Name=\"MyApp\">
                            <Component Id=\"MainComp\">
                                <File Id=\"MainExe\" />
                            </Component>
                        </Directory>
                    </Directory>
                </Wix>",
            )
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check hierarchy indentation
    let lines: Vec<&str> = stdout.lines().collect();

    // Find TARGETDIR line
    let targetdir = lines.iter().position(|l| l.contains("TARGETDIR")).unwrap();
    let installfolder = lines
        .iter()
        .position(|l| l.contains("INSTALLFOLDER"))
        .unwrap();
    let maincomp = lines.iter().position(|l| l.contains("MainComp")).unwrap();
    let mainexe = lines.iter().position(|l| l.contains("MainExe")).unwrap();

    // Verify order (parents before children)
    assert!(targetdir < installfolder);
    assert!(installfolder < maincomp);
    assert!(maincomp < mainexe);
}

#[test]
fn test_empty_file() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["-"])
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty());
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
        stdin.write_all(b"<Wix><Invalid").unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error"));
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
    assert!(stdout.contains("wix-symbols"));
    assert!(stdout.contains("--format"));
    assert!(stdout.contains("--flat"));
    assert!(stdout.contains("--query"));
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
    assert!(stdout.contains("wix-symbols"));
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

#[test]
fn test_directory_with_name() {
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
            .write_all(b"<Wix><Directory Id=\"INSTALLFOLDER\" Name=\"MyApp\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("INSTALLFOLDER"));
    assert!(stdout.contains("(MyApp)"));
}

#[test]
fn test_feature_with_title() {
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
            .write_all(b"<Wix><Feature Id=\"MainFeature\" Title=\"Main Application\" /></Wix>")
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Feature: MainFeature"));
    assert!(stdout.contains("(Main Application)"));
}

#[test]
fn test_json_flat_mode() {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(["--format", "json", "--flat", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(
                b"<Wix><Directory Id=\"TARGETDIR\"><Component Id=\"Comp1\" /></Directory></Wix>",
            )
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be a flat array
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed.len(), 2);
}

#[test]
fn test_multiple_elements() {
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
            .write_all(
                b"<Wix>
                    <Property Id=\"INSTALLDIR\" />
                    <CustomAction Id=\"SetDir\" />
                    <Feature Id=\"Main\" />
                </Wix>",
            )
            .unwrap();
    }

    let output = child.wait_with_output().expect("Failed to read output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Property: INSTALLDIR"));
    assert!(stdout.contains("CustomAction: SetDir"));
    assert!(stdout.contains("Feature: Main"));
}
