//! ICE rule loading from wixkb database

use crate::types::{IceRule, Severity};
use crate::Result;
use rusqlite::Connection;
use std::path::Path;

/// Load ICE rules from wixkb database
pub fn load_from_wixkb<P: AsRef<Path>>(db_path: P) -> Result<Vec<IceRule>> {
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT code, severity, description, resolution, tables_affected, documentation_url
         FROM ice_rules
         ORDER BY code"
    )?;

    let rules = stmt.query_map([], |row| {
        let code: String = row.get(0)?;
        let severity_str: String = row.get(1)?;
        let description: String = row.get(2)?;
        let resolution: Option<String> = row.get(3)?;
        let tables_str: Option<String> = row.get(4)?;
        let documentation_url: Option<String> = row.get(5)?;

        let severity = severity_str.parse().unwrap_or(Severity::Warning);
        let tables_affected = tables_str
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
            .unwrap_or_default();

        Ok(IceRule {
            code,
            severity,
            description,
            resolution,
            tables_affected,
            documentation_url,
        })
    })?;

    rules.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
}

/// Get default wixkb database path
pub fn default_wixkb_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".wixcraft").join("wixkb.db"))
}

/// Built-in subset of common ICE rules (fallback when wixkb not available)
pub fn builtin_rules() -> Vec<IceRule> {
    vec![
        IceRule {
            code: "ICE03".into(),
            severity: Severity::Error,
            description: "Basic data and foreign key validation".into(),
            resolution: Some("Ensure all foreign key references point to existing rows".into()),
            tables_affected: vec!["Component".into(), "Directory".into(), "Feature".into()],
            documentation_url: Some("https://learn.microsoft.com/en-us/windows/win32/msi/ice03".into()),
        },
        IceRule {
            code: "ICE06".into(),
            severity: Severity::Error,
            description: "Validates for missing column or tables in the database".into(),
            resolution: Some("Add missing required columns or tables".into()),
            tables_affected: vec!["_Validation".into()],
            documentation_url: Some("https://learn.microsoft.com/en-us/windows/win32/msi/ice06".into()),
        },
        IceRule {
            code: "ICE08".into(),
            severity: Severity::Error,
            description: "Checks for duplicate GUIDs in the ComponentId column".into(),
            resolution: Some("Ensure each component has a unique GUID".into()),
            tables_affected: vec!["Component".into()],
            documentation_url: Some("https://learn.microsoft.com/en-us/windows/win32/msi/ice08".into()),
        },
        IceRule {
            code: "ICE09".into(),
            severity: Severity::Error,
            description: "Validates permanent bit for SystemFolder components".into(),
            resolution: Some("Set permanent bit for components in system folders".into()),
            tables_affected: vec!["Component".into()],
            documentation_url: Some("https://learn.microsoft.com/en-us/windows/win32/msi/ice09".into()),
        },
        IceRule {
            code: "ICE18".into(),
            severity: Severity::Error,
            description: "Validates that KeyPath is not empty".into(),
            resolution: Some("Specify a KeyPath for each component".into()),
            tables_affected: vec!["Component".into()],
            documentation_url: Some("https://learn.microsoft.com/en-us/windows/win32/msi/ice18".into()),
        },
        IceRule {
            code: "ICE30".into(),
            severity: Severity::Error,
            description: "Validates that the same file is not installed to multiple directories".into(),
            resolution: Some("Use unique file names or different target directories".into()),
            tables_affected: vec!["File".into(), "Component".into()],
            documentation_url: Some("https://learn.microsoft.com/en-us/windows/win32/msi/ice30".into()),
        },
        IceRule {
            code: "ICE33".into(),
            severity: Severity::Warning,
            description: "Validates registry entries formatting".into(),
            resolution: Some("Ensure registry entries use correct formatting".into()),
            tables_affected: vec!["Registry".into()],
            documentation_url: Some("https://learn.microsoft.com/en-us/windows/win32/msi/ice33".into()),
        },
        IceRule {
            code: "ICE38".into(),
            severity: Severity::Error,
            description: "Validates that advertised components have a valid feature".into(),
            resolution: Some("Ensure advertised components are linked to features".into()),
            tables_affected: vec!["Component".into(), "Feature".into()],
            documentation_url: Some("https://learn.microsoft.com/en-us/windows/win32/msi/ice38".into()),
        },
        IceRule {
            code: "ICE43".into(),
            severity: Severity::Error,
            description: "Validates that non-advertised shortcuts do not reference features".into(),
            resolution: Some("Remove feature reference from non-advertised shortcuts".into()),
            tables_affected: vec!["Shortcut".into()],
            documentation_url: Some("https://learn.microsoft.com/en-us/windows/win32/msi/ice43".into()),
        },
        IceRule {
            code: "ICE57".into(),
            severity: Severity::Error,
            description: "Validates that components don't have both per-user and per-machine data".into(),
            resolution: Some("Split component into per-user and per-machine components".into()),
            tables_affected: vec!["Component".into(), "Registry".into()],
            documentation_url: Some("https://learn.microsoft.com/en-us/windows/win32/msi/ice57".into()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_rules() {
        let rules = builtin_rules();
        assert!(!rules.is_empty());
        assert!(rules.iter().any(|r| r.code == "ICE03"));
    }

    #[test]
    fn test_default_wixkb_path() {
        let path = default_wixkb_path();
        assert!(path.is_some());
        assert!(path.unwrap().ends_with("wixkb.db"));
    }
}
