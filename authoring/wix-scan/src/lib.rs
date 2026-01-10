//! Directory scanner for WiX
//!
//! Scans directories and generates WXS fragments with Component/File elements.
//!
//! # Example
//!
//! ```no_run
//! use wix_scan::{Scanner, ScanOptions};
//!
//! let options = ScanOptions::default();
//! let scanner = Scanner::new(options);
//! let result = scanner.scan("./dist").unwrap();
//! println!("{}", result.to_wxs());
//! ```

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// Scan errors
#[derive(Error, Debug)]
pub enum ScanError {
    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Options for scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    /// Generate GUIDs for components (false = use "*")
    pub generate_guids: bool,
    /// Component group ID
    pub component_group: String,
    /// Directory reference ID
    pub directory_ref: String,
    /// Prefix for component IDs
    pub component_prefix: String,
    /// Include hidden files
    pub include_hidden: bool,
    /// File patterns to exclude (glob patterns)
    pub exclude_patterns: Vec<String>,
    /// File patterns to include (glob patterns, empty = all)
    pub include_patterns: Vec<String>,
    /// Generate 64-bit components
    pub win64: bool,
    /// Source path variable (e.g., "SourceDir")
    pub source_var: Option<String>,
    /// Suppress root directory element
    pub suppress_root_dir: bool,
    /// Generate registry key per component for proper ref counting
    pub generate_registry_key: bool,
    /// Preserve directory structure
    pub preserve_structure: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            generate_guids: false,
            component_group: "ScannedComponents".to_string(),
            directory_ref: "INSTALLFOLDER".to_string(),
            component_prefix: "cmp".to_string(),
            include_hidden: false,
            exclude_patterns: vec![
                "*.pdb".to_string(),
                "*.obj".to_string(),
                "*.log".to_string(),
                ".git/**".to_string(),
                ".svn/**".to_string(),
            ],
            include_patterns: vec![],
            win64: false,
            source_var: None,
            suppress_root_dir: false,
            generate_registry_key: false,
            preserve_structure: true,
        }
    }
}

/// A scanned file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    /// Component ID
    pub component_id: String,
    /// File ID
    pub file_id: String,
    /// Source path (absolute)
    pub source_path: PathBuf,
    /// Relative path from scan root
    pub relative_path: PathBuf,
    /// File name
    pub name: String,
    /// Component GUID
    pub guid: String,
    /// Is key path
    pub key_path: bool,
}

/// A scanned directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedDirectory {
    /// Directory ID
    pub id: String,
    /// Directory name
    pub name: String,
    /// Parent directory ID
    pub parent_id: Option<String>,
    /// Files in this directory
    pub files: Vec<ScannedFile>,
    /// Subdirectories
    pub subdirs: Vec<ScannedDirectory>,
}

/// Result of scanning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanResult {
    /// Root directory
    pub root: Option<ScannedDirectory>,
    /// All files (flat list)
    pub files: Vec<ScannedFile>,
    /// All directories (flat list)
    pub directories: Vec<String>,
    /// Component group ID
    pub component_group: String,
    /// Directory reference
    pub directory_ref: String,
    /// Options used
    pub options: ScanOptions,
}

impl ScanResult {
    /// Generate WXS fragment
    pub fn to_wxs(&self) -> String {
        let mut wxs = String::new();

        wxs.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        wxs.push_str("<Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\">\n");
        wxs.push_str(&format!(
            "  <Fragment>\n    <ComponentGroup Id=\"{}\">\n",
            self.component_group
        ));

        // Generate component references
        for file in &self.files {
            wxs.push_str(&format!(
                "      <ComponentRef Id=\"{}\" />\n",
                file.component_id
            ));
        }

        wxs.push_str("    </ComponentGroup>\n  </Fragment>\n\n");

        // Generate directory structure and components
        wxs.push_str("  <Fragment>\n");
        wxs.push_str(&format!(
            "    <DirectoryRef Id=\"{}\">\n",
            self.directory_ref
        ));

        if let Some(ref root) = self.root {
            self.write_directory(&mut wxs, root, 3);
        }

        wxs.push_str("    </DirectoryRef>\n  </Fragment>\n");
        wxs.push_str("</Wix>\n");

        wxs
    }

    fn write_directory(&self, wxs: &mut String, dir: &ScannedDirectory, indent: usize) {
        let pad = "  ".repeat(indent);

        // Write directory element if not root or if we want the root
        let write_dir_element = !self.options.suppress_root_dir || indent > 3;

        if write_dir_element && !dir.name.is_empty() {
            wxs.push_str(&format!(
                "{}<Directory Id=\"{}\" Name=\"{}\">\n",
                pad, dir.id, dir.name
            ));
        }

        let inner_indent = if write_dir_element && !dir.name.is_empty() {
            indent + 1
        } else {
            indent
        };
        let inner_pad = "  ".repeat(inner_indent);

        // Write files as components
        for file in &dir.files {
            let win64_attr = if self.options.win64 {
                " Bitness=\"always64\""
            } else {
                ""
            };

            wxs.push_str(&format!(
                "{}<Component Id=\"{}\" Guid=\"{}\"{}>",
                inner_pad, file.component_id, file.guid, win64_attr
            ));

            let source = if let Some(ref var) = self.options.source_var {
                format!("$(var.{})\\{}", var, file.relative_path.display())
            } else {
                file.source_path.display().to_string()
            };

            wxs.push_str(&format!(
                "\n{}  <File Id=\"{}\" Source=\"{}\" KeyPath=\"yes\" />\n",
                inner_pad, file.file_id, source
            ));

            if self.options.generate_registry_key {
                wxs.push_str(&format!(
                    "{}  <RegistryValue Root=\"HKCU\" Key=\"Software\\[Manufacturer]\\[ProductName]\" Name=\"{}\" Type=\"string\" Value=\"1\" />\n",
                    inner_pad, file.file_id
                ));
            }

            wxs.push_str(&format!("{}</Component>\n", inner_pad));
        }

        // Write subdirectories recursively
        for subdir in &dir.subdirs {
            self.write_directory(wxs, subdir, inner_indent);
        }

        if write_dir_element && !dir.name.is_empty() {
            wxs.push_str(&format!("{}</Directory>\n", pad));
        }
    }

    /// Get statistics
    pub fn stats(&self) -> ScanStats {
        ScanStats {
            total_files: self.files.len(),
            total_directories: self.directories.len(),
            total_components: self.files.len(),
        }
    }
}

/// Statistics about scan
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanStats {
    pub total_files: usize,
    pub total_directories: usize,
    pub total_components: usize,
}

/// Directory scanner
pub struct Scanner {
    options: ScanOptions,
    exclude_matchers: Vec<glob::Pattern>,
    include_matchers: Vec<glob::Pattern>,
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new(ScanOptions::default())
    }
}

impl Scanner {
    pub fn new(options: ScanOptions) -> Self {
        let exclude_matchers: Vec<_> = options
            .exclude_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();

        let include_matchers: Vec<_> = options
            .include_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();

        Self {
            options,
            exclude_matchers,
            include_matchers,
        }
    }

    /// Scan files from a directory
    pub fn scan(&self, path: impl AsRef<Path>) -> Result<ScanResult, ScanError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ScanError::DirectoryNotFound(
                path.display().to_string(),
            ));
        }

        if !path.is_dir() {
            return Err(ScanError::InvalidPath(format!(
                "Not a directory: {}",
                path.display()
            )));
        }

        let mut result = ScanResult {
            component_group: self.options.component_group.clone(),
            directory_ref: self.options.directory_ref.clone(),
            options: self.options.clone(),
            ..Default::default()
        };

        let root_dir = self.scan_directory(path, path, None)?;
        self.flatten_files(&root_dir, &mut result.files);
        self.flatten_directories(&root_dir, &mut result.directories);
        result.root = Some(root_dir);

        Ok(result)
    }

    fn scan_directory(
        &self,
        path: &Path,
        root: &Path,
        parent_id: Option<&str>,
    ) -> Result<ScannedDirectory, ScanError> {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let id = self.generate_directory_id(&name, parent_id);

        let mut dir = ScannedDirectory {
            id: id.clone(),
            name,
            parent_id: parent_id.map(|s| s.to_string()),
            files: Vec::new(),
            subdirs: Vec::new(),
        };

        let entries: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();

        for entry in entries {
            let entry_path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files if configured
            if !self.options.include_hidden && file_name.starts_with('.') {
                continue;
            }

            // Get relative path for pattern matching
            let relative = entry_path
                .strip_prefix(root)
                .unwrap_or(&entry_path);

            // Check exclude patterns
            if self.should_exclude(relative) {
                continue;
            }

            if entry_path.is_dir() {
                let subdir = self.scan_directory(&entry_path, root, Some(&id))?;
                if !subdir.files.is_empty() || !subdir.subdirs.is_empty() {
                    dir.subdirs.push(subdir);
                }
            } else if entry_path.is_file() {
                // Check include patterns
                if !self.should_include(relative) {
                    continue;
                }

                let file = self.scan_file(&entry_path, root, &id)?;
                dir.files.push(file);
            }
        }

        Ok(dir)
    }

    fn scan_file(
        &self,
        path: &Path,
        root: &Path,
        _parent_id: &str,
    ) -> Result<ScannedFile, ScanError> {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let relative = path.strip_prefix(root).unwrap_or(path);

        let file_id = self.generate_file_id(&name, relative);
        let component_id = format!("{}_{}", self.options.component_prefix, file_id);

        let guid = if self.options.generate_guids {
            Uuid::new_v4().to_string().to_uppercase()
        } else {
            "*".to_string()
        };

        Ok(ScannedFile {
            component_id,
            file_id,
            source_path: path.to_path_buf(),
            relative_path: relative.to_path_buf(),
            name,
            guid,
            key_path: true,
        })
    }

    fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_matchers
            .iter()
            .any(|p| p.matches(&path_str) || p.matches_path(path))
    }

    fn should_include(&self, path: &Path) -> bool {
        if self.include_matchers.is_empty() {
            return true;
        }

        let path_str = path.to_string_lossy();
        self.include_matchers
            .iter()
            .any(|p| p.matches(&path_str) || p.matches_path(path))
    }

    fn generate_directory_id(&self, name: &str, parent_id: Option<&str>) -> String {
        let sanitized = self.sanitize_id(name);
        if let Some(parent) = parent_id {
            format!("{}_{}", parent, sanitized)
        } else {
            format!("dir_{}", sanitized)
        }
    }

    fn generate_file_id(&self, name: &str, relative: &Path) -> String {
        let sanitized = self.sanitize_id(name);

        // Include path hash for uniqueness
        let path_hash = self.hash_path(relative);
        format!("{}_{}", sanitized, path_hash)
    }

    fn sanitize_id(&self, name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    fn hash_path(&self, path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("{:08x}", hasher.finish() & 0xFFFFFFFF)
    }

    fn flatten_files(&self, dir: &ScannedDirectory, files: &mut Vec<ScannedFile>) {
        files.extend(dir.files.clone());
        for subdir in &dir.subdirs {
            self.flatten_files(subdir, files);
        }
    }

    fn flatten_directories(&self, dir: &ScannedDirectory, dirs: &mut Vec<String>) {
        if !dir.id.is_empty() {
            dirs.push(dir.id.clone());
        }
        for subdir in &dir.subdirs {
            self.flatten_directories(subdir, dirs);
        }
    }
}

/// Quick scan with default options
pub fn scan(path: impl AsRef<Path>) -> Result<ScanResult, ScanError> {
    Scanner::default().scan(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs::{self, File};
    use tempfile::tempdir;

    fn create_test_structure(dir: &Path) {
        fs::create_dir_all(dir.join("bin")).unwrap();
        fs::create_dir_all(dir.join("lib")).unwrap();
        File::create(dir.join("app.exe")).unwrap();
        File::create(dir.join("bin/tool.exe")).unwrap();
        File::create(dir.join("lib/helper.dll")).unwrap();
    }

    #[test]
    fn test_scan_basic() {
        let dir = tempdir().unwrap();
        create_test_structure(dir.path());

        let result = scan(dir.path()).unwrap();

        assert_eq!(result.files.len(), 3);
        assert!(result.files.iter().any(|f| f.name == "app.exe"));
        assert!(result.files.iter().any(|f| f.name == "tool.exe"));
        assert!(result.files.iter().any(|f| f.name == "helper.dll"));
    }

    #[test]
    fn test_scan_generates_wxs() {
        let dir = tempdir().unwrap();
        create_test_structure(dir.path());

        let result = scan(dir.path()).unwrap();
        let wxs = result.to_wxs();

        assert!(wxs.contains("<Wix"));
        assert!(wxs.contains("<ComponentGroup"));
        assert!(wxs.contains("<Component"));
        assert!(wxs.contains("<File"));
        assert!(wxs.contains("KeyPath=\"yes\""));
    }

    #[test]
    fn test_scan_empty_dir() {
        let dir = tempdir().unwrap();

        let result = scan(dir.path()).unwrap();

        assert!(result.files.is_empty());
    }

    #[test]
    fn test_scan_nonexistent_dir() {
        let result = scan("/nonexistent/path");

        assert!(matches!(result, Err(ScanError::DirectoryNotFound(_))));
    }

    #[test]
    fn test_exclude_patterns() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("app.exe")).unwrap();
        File::create(dir.path().join("debug.pdb")).unwrap();
        File::create(dir.path().join("test.log")).unwrap();

        let options = ScanOptions {
            exclude_patterns: vec!["*.pdb".to_string(), "*.log".to_string()],
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].name, "app.exe");
    }

    #[test]
    fn test_include_patterns() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("app.exe")).unwrap();
        File::create(dir.path().join("helper.dll")).unwrap();
        File::create(dir.path().join("readme.txt")).unwrap();

        let options = ScanOptions {
            include_patterns: vec!["*.exe".to_string(), "*.dll".to_string()],
            exclude_patterns: vec![],
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.files.len(), 2);
        assert!(result.files.iter().all(|f| f.name.ends_with(".exe") || f.name.ends_with(".dll")));
    }

    #[test]
    fn test_generate_guids() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("app.exe")).unwrap();

        let options = ScanOptions {
            generate_guids: true,
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        assert!(!result.files[0].guid.is_empty());
        assert_ne!(result.files[0].guid, "*");
        // GUID format check
        assert!(result.files[0].guid.contains('-'));
    }

    #[test]
    fn test_auto_guid() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("app.exe")).unwrap();

        let options = ScanOptions {
            generate_guids: false,
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.files[0].guid, "*");
    }

    #[test]
    fn test_custom_component_group() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("app.exe")).unwrap();

        let options = ScanOptions {
            component_group: "MyAppFiles".to_string(),
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.component_group, "MyAppFiles");
        let wxs = result.to_wxs();
        assert!(wxs.contains("ComponentGroup Id=\"MyAppFiles\""));
    }

    #[test]
    fn test_directory_ref() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("app.exe")).unwrap();

        let options = ScanOptions {
            directory_ref: "ProgramFilesFolder".to_string(),
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        let wxs = result.to_wxs();
        assert!(wxs.contains("DirectoryRef Id=\"ProgramFilesFolder\""));
    }

    #[test]
    fn test_win64_components() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("app.exe")).unwrap();

        let options = ScanOptions {
            win64: true,
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        let wxs = result.to_wxs();
        assert!(wxs.contains("Bitness=\"always64\""));
    }

    #[test]
    fn test_source_variable() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("app.exe")).unwrap();

        let options = ScanOptions {
            source_var: Some("SourceDir".to_string()),
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        let wxs = result.to_wxs();
        assert!(wxs.contains("$(var.SourceDir)"));
    }

    #[test]
    fn test_nested_directories() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a/b/c")).unwrap();
        File::create(dir.path().join("a/b/c/deep.txt")).unwrap();

        let result = scan(dir.path()).unwrap();

        assert_eq!(result.files.len(), 1);
        assert!(result.directories.len() >= 3); // a, b, c
    }

    #[test]
    fn test_hidden_files_excluded() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("visible.txt")).unwrap();
        File::create(dir.path().join(".hidden")).unwrap();

        let options = ScanOptions {
            include_hidden: false,
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0].name, "visible.txt");
    }

    #[test]
    fn test_hidden_files_included() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("visible.txt")).unwrap();
        File::create(dir.path().join(".hidden")).unwrap();

        let options = ScanOptions {
            include_hidden: true,
            exclude_patterns: vec![],
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.files.len(), 2);
    }

    #[test]
    fn test_scan_stats() {
        let dir = tempdir().unwrap();
        create_test_structure(dir.path());

        let result = scan(dir.path()).unwrap();
        let stats = result.stats();

        assert_eq!(stats.total_files, 3);
        assert_eq!(stats.total_components, 3);
        assert!(stats.total_directories > 0);
    }

    #[test]
    fn test_component_prefix() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("app.exe")).unwrap();

        let options = ScanOptions {
            component_prefix: "MyApp".to_string(),
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        assert!(result.files[0].component_id.starts_with("MyApp_"));
    }

    #[test]
    fn test_file_not_directory() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        File::create(&file_path).unwrap();

        let result = Scanner::default().scan(&file_path);
        assert!(matches!(result, Err(ScanError::InvalidPath(_))));
    }

    #[test]
    fn test_registry_key_generation() {
        let dir = tempdir().unwrap();
        File::create(dir.path().join("app.exe")).unwrap();

        let options = ScanOptions {
            generate_registry_key: true,
            ..Default::default()
        };
        let scanner = Scanner::new(options);
        let result = scanner.scan(dir.path()).unwrap();

        let wxs = result.to_wxs();
        assert!(wxs.contains("<RegistryValue"));
        assert!(wxs.contains("Root=\"HKCU\""));
    }

    #[test]
    fn test_unique_file_ids() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a")).unwrap();
        fs::create_dir_all(dir.path().join("b")).unwrap();
        // Same filename in different directories
        File::create(dir.path().join("a/file.txt")).unwrap();
        File::create(dir.path().join("b/file.txt")).unwrap();

        let result = scan(dir.path()).unwrap();

        let file_ids: HashSet<_> = result.files.iter().map(|f| &f.file_id).collect();
        assert_eq!(file_ids.len(), 2); // Should be unique
    }

    #[test]
    fn test_default_options() {
        let options = ScanOptions::default();

        assert!(!options.generate_guids);
        assert!(!options.include_hidden);
        assert!(!options.win64);
        assert!(options.preserve_structure);
        assert!(!options.exclude_patterns.is_empty());
    }
}
