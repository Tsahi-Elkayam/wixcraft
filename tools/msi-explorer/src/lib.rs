//! MSI Explorer - Cross-platform MSI database explorer
//!
//! A modern alternative to Microsoft Orca with:
//! - Cross-platform support (Windows, macOS, Linux)
//! - Global search across all tables
//! - MSI comparison/diff
//! - ICE validation (via ice-validator)
//! - Export to JSON/CSV/SQL
//! - MSI building and analysis

pub mod types;
pub mod reader;
pub mod search;
pub mod diff;
pub mod export;
pub mod builder;

pub use types::*;
pub use reader::MsiFile;
pub use builder::*;

use thiserror::Error;

/// Errors that can occur in msi-explorer
#[derive(Error, Debug)]
pub enum MsiError {
    #[error("Failed to open MSI file: {0}")]
    OpenError(String),

    #[error("Table not found: {0}")]
    TableNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, MsiError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::*;
    use crate::diff::*;
    use crate::search::*;

    // ===========================================
    // Types Tests
    // ===========================================

    #[test]
    fn test_cell_value_display() {
        assert_eq!(CellValue::String("test".to_string()).display(), "test");
        assert_eq!(CellValue::Integer(42).display(), "42");
        assert_eq!(CellValue::Null.display(), "");
    }

    #[test]
    fn test_cell_value_is_null() {
        assert!(CellValue::Null.is_null());
        assert!(!CellValue::String("test".to_string()).is_null());
        assert!(!CellValue::Integer(42).is_null());
    }

    #[test]
    fn test_cell_value_as_str() {
        let s = CellValue::String("test".to_string());
        assert_eq!(s.as_str(), Some("test"));
        assert_eq!(CellValue::Integer(42).as_str(), None);
        assert_eq!(CellValue::Null.as_str(), None);
    }

    #[test]
    fn test_cell_value_display_trait() {
        let s = CellValue::String("hello".to_string());
        assert_eq!(format!("{}", s), "hello");

        let i = CellValue::Integer(123);
        assert_eq!(format!("{}", i), "123");

        let n = CellValue::Null;
        assert_eq!(format!("{}", n), "");
    }

    #[test]
    fn test_column_type_display_name() {
        assert_eq!(ColumnType::String.display_name(), "String");
        assert_eq!(ColumnType::Integer.display_name(), "Integer");
    }

    #[test]
    fn test_column_is_foreign_key() {
        let fk_col = Column {
            name: "Component_".to_string(),
            col_type: ColumnType::String,
            nullable: false,
            primary_key: false,
        };
        assert!(fk_col.is_foreign_key());
        assert_eq!(fk_col.referenced_table(), Some("Component"));

        let normal_col = Column {
            name: "Name".to_string(),
            col_type: ColumnType::String,
            nullable: false,
            primary_key: true,
        };
        assert!(!normal_col.is_foreign_key());
        assert_eq!(normal_col.referenced_table(), None);
    }

    #[test]
    fn test_table_column_index() {
        let table = Table {
            name: "Test".to_string(),
            columns: vec![
                Column {
                    name: "Id".to_string(),
                    col_type: ColumnType::String,
                    nullable: false,
                    primary_key: true,
                },
                Column {
                    name: "Value".to_string(),
                    col_type: ColumnType::Integer,
                    nullable: true,
                    primary_key: false,
                },
            ],
            rows: vec![],
        };

        assert_eq!(table.column_index("Id"), Some(0));
        assert_eq!(table.column_index("Value"), Some(1));
        assert_eq!(table.column_index("Unknown"), None);
    }

    #[test]
    fn test_table_primary_keys() {
        let table = Table {
            name: "Test".to_string(),
            columns: vec![
                Column {
                    name: "Key1".to_string(),
                    col_type: ColumnType::String,
                    nullable: false,
                    primary_key: true,
                },
                Column {
                    name: "Key2".to_string(),
                    col_type: ColumnType::String,
                    nullable: false,
                    primary_key: true,
                },
                Column {
                    name: "Value".to_string(),
                    col_type: ColumnType::Integer,
                    nullable: true,
                    primary_key: false,
                },
            ],
            rows: vec![],
        };

        let pks = table.primary_keys();
        assert_eq!(pks.len(), 2);
        assert!(pks.contains(&"Key1"));
        assert!(pks.contains(&"Key2"));
    }

    #[test]
    fn test_table_counts() {
        let table = Table {
            name: "Test".to_string(),
            columns: vec![
                Column {
                    name: "Id".to_string(),
                    col_type: ColumnType::String,
                    nullable: false,
                    primary_key: true,
                },
            ],
            rows: vec![
                Row { values: vec![CellValue::String("A".to_string())] },
                Row { values: vec![CellValue::String("B".to_string())] },
            ],
        };

        assert_eq!(table.row_count(), 2);
        assert_eq!(table.column_count(), 1);
    }

    #[test]
    fn test_table_category_from_name() {
        assert_eq!(TableCategory::from_table_name("Property"), TableCategory::Core);
        assert_eq!(TableCategory::from_table_name("Component"), TableCategory::Core);
        assert_eq!(TableCategory::from_table_name("File"), TableCategory::File);
        assert_eq!(TableCategory::from_table_name("Registry"), TableCategory::Registry);
        assert_eq!(TableCategory::from_table_name("Dialog"), TableCategory::UI);
        assert_eq!(TableCategory::from_table_name("CustomAction"), TableCategory::CustomAction);
        assert_eq!(TableCategory::from_table_name("ServiceInstall"), TableCategory::Service);
        assert_eq!(TableCategory::from_table_name("InstallExecuteSequence"), TableCategory::Sequence);
        assert_eq!(TableCategory::from_table_name("_Validation"), TableCategory::Validation);
        assert_eq!(TableCategory::from_table_name("Unknown"), TableCategory::Other);
    }

    #[test]
    fn test_table_category_display_name() {
        assert_eq!(TableCategory::Core.display_name(), "Core");
        assert_eq!(TableCategory::File.display_name(), "Files");
        assert_eq!(TableCategory::Registry.display_name(), "Registry");
        assert_eq!(TableCategory::UI.display_name(), "User Interface");
        assert_eq!(TableCategory::CustomAction.display_name(), "Custom Actions");
        assert_eq!(TableCategory::Service.display_name(), "Services");
        assert_eq!(TableCategory::Sequence.display_name(), "Sequences");
        assert_eq!(TableCategory::Validation.display_name(), "Validation");
        assert_eq!(TableCategory::Other.display_name(), "Other");
    }

    #[test]
    fn test_summary_info_platform() {
        let mut info = SummaryInfo::default();
        assert_eq!(info.platform(), None);

        info.subject = Some("Intel;1033".to_string());
        assert_eq!(info.platform(), Some("x86"));

        info.subject = Some("x64;1033".to_string());
        assert_eq!(info.platform(), Some("x64"));

        info.subject = Some("Intel64;1033".to_string());
        assert_eq!(info.platform(), Some("x64"));

        info.subject = Some("Arm64;1033".to_string());
        assert_eq!(info.platform(), Some("ARM64"));
    }

    // ===========================================
    // Export Tests
    // ===========================================

    #[test]
    fn test_table_to_json() {
        let table = Table {
            name: "Property".to_string(),
            columns: vec![
                Column {
                    name: "Property".to_string(),
                    col_type: ColumnType::String,
                    nullable: false,
                    primary_key: true,
                },
                Column {
                    name: "Value".to_string(),
                    col_type: ColumnType::String,
                    nullable: true,
                    primary_key: false,
                },
            ],
            rows: vec![
                Row {
                    values: vec![
                        CellValue::String("ProductName".to_string()),
                        CellValue::String("Test Product".to_string()),
                    ],
                },
            ],
        };

        let json = table_to_json(&table);
        assert_eq!(json["name"], "Property");
        assert_eq!(json["row_count"], 1);
        assert_eq!(json["rows"][0]["Property"], "ProductName");
        assert_eq!(json["rows"][0]["Value"], "Test Product");
    }

    #[test]
    fn test_table_to_csv() {
        let table = Table {
            name: "Test".to_string(),
            columns: vec![
                Column {
                    name: "Name".to_string(),
                    col_type: ColumnType::String,
                    nullable: false,
                    primary_key: true,
                },
                Column {
                    name: "Count".to_string(),
                    col_type: ColumnType::Integer,
                    nullable: false,
                    primary_key: false,
                },
            ],
            rows: vec![
                Row {
                    values: vec![
                        CellValue::String("Item1".to_string()),
                        CellValue::Integer(10),
                    ],
                },
                Row {
                    values: vec![
                        CellValue::String("Item2".to_string()),
                        CellValue::Integer(20),
                    ],
                },
            ],
        };

        let mut output = Vec::new();
        table_to_csv(&table, &mut output).unwrap();
        let csv = String::from_utf8(output).unwrap();

        assert!(csv.contains("Name,Count"));
        assert!(csv.contains("Item1,10"));
        assert!(csv.contains("Item2,20"));
    }

    #[test]
    fn test_table_to_csv_with_special_chars() {
        let table = Table {
            name: "Test".to_string(),
            columns: vec![
                Column {
                    name: "Text".to_string(),
                    col_type: ColumnType::String,
                    nullable: false,
                    primary_key: true,
                },
            ],
            rows: vec![
                Row {
                    values: vec![CellValue::String("Hello, World".to_string())],
                },
                Row {
                    values: vec![CellValue::String("Quote\"Test".to_string())],
                },
            ],
        };

        let mut output = Vec::new();
        table_to_csv(&table, &mut output).unwrap();
        let csv = String::from_utf8(output).unwrap();

        // Values with commas should be quoted
        assert!(csv.contains("\"Hello, World\""));
        // Values with quotes should be escaped
        assert!(csv.contains("\"Quote\"\"Test\""));
    }

    #[test]
    fn test_table_to_sql() {
        let table = Table {
            name: "Property".to_string(),
            columns: vec![
                Column {
                    name: "Property".to_string(),
                    col_type: ColumnType::String,
                    nullable: false,
                    primary_key: true,
                },
                Column {
                    name: "Value".to_string(),
                    col_type: ColumnType::Integer,
                    nullable: true,
                    primary_key: false,
                },
            ],
            rows: vec![
                Row {
                    values: vec![
                        CellValue::String("Version".to_string()),
                        CellValue::Integer(100),
                    ],
                },
            ],
        };

        let mut output = Vec::new();
        table_to_sql(&table, &mut output).unwrap();
        let sql = String::from_utf8(output).unwrap();

        assert!(sql.contains("INSERT INTO Property"));
        assert!(sql.contains("Property, Value"));
        assert!(sql.contains("'Version', 100"));
    }

    #[test]
    fn test_table_to_sql_with_null() {
        let table = Table {
            name: "Test".to_string(),
            columns: vec![
                Column {
                    name: "Id".to_string(),
                    col_type: ColumnType::String,
                    nullable: false,
                    primary_key: true,
                },
                Column {
                    name: "Optional".to_string(),
                    col_type: ColumnType::String,
                    nullable: true,
                    primary_key: false,
                },
            ],
            rows: vec![
                Row {
                    values: vec![
                        CellValue::String("A".to_string()),
                        CellValue::Null,
                    ],
                },
            ],
        };

        let mut output = Vec::new();
        table_to_sql(&table, &mut output).unwrap();
        let sql = String::from_utf8(output).unwrap();

        assert!(sql.contains("'A', NULL"));
    }

    // ===========================================
    // Search Tests
    // ===========================================

    #[test]
    fn test_search_result_highlighted() {
        let result = SearchResult {
            table: "Property".to_string(),
            column: "Value".to_string(),
            row_index: 0,
            primary_key: "ProductName".to_string(),
            value: "My Product Name".to_string(),
            match_start: 3,
            match_end: 10,
        };

        let highlighted = result.highlighted("[", "]");
        assert_eq!(highlighted, "My [Product] Name");
    }

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert!(!options.case_sensitive);
        assert!(options.tables.is_none());
        assert!(options.columns.is_none());
        assert!(options.max_results.is_none());
    }

    // ===========================================
    // Diff Tests
    // ===========================================

    #[test]
    fn test_msi_diff_has_differences() {
        let mut diff = MsiDiff::default();
        assert!(!diff.has_differences());

        diff.tables_only_in_first.push("CustomTable".to_string());
        assert!(diff.has_differences());
    }

    #[test]
    fn test_msi_diff_change_count() {
        let mut diff = MsiDiff::default();
        assert_eq!(diff.change_count(), 0);

        diff.tables_only_in_first.push("Table1".to_string());
        diff.tables_only_in_second.push("Table2".to_string());
        assert_eq!(diff.change_count(), 2);

        diff.property_diffs.push(PropertyDiff {
            name: "ProductVersion".to_string(),
            old_value: Some("1.0".to_string()),
            new_value: Some("2.0".to_string()),
        });
        assert_eq!(diff.change_count(), 3);
    }

    #[test]
    fn test_table_diff_change_count() {
        let diff = TableDiff {
            table_name: "Property".to_string(),
            rows_added: vec![
                RowChange {
                    primary_key: "NewProp".to_string(),
                    values: std::collections::HashMap::new(),
                },
            ],
            rows_removed: vec![],
            rows_modified: vec![
                RowModification {
                    primary_key: "ExistingProp".to_string(),
                    cell_changes: vec![
                        CellChange {
                            column: "Value".to_string(),
                            old_value: "Old".to_string(),
                            new_value: "New".to_string(),
                        },
                    ],
                },
            ],
        };

        assert_eq!(diff.change_count(), 2);
        assert!(!diff.is_empty());
    }

    #[test]
    fn test_table_diff_is_empty() {
        let diff = TableDiff {
            table_name: "Property".to_string(),
            rows_added: vec![],
            rows_removed: vec![],
            rows_modified: vec![],
        };

        assert!(diff.is_empty());
        assert_eq!(diff.change_count(), 0);
    }

    #[test]
    fn test_property_diff_change_type() {
        let added = PropertyDiff {
            name: "NewProp".to_string(),
            old_value: None,
            new_value: Some("Value".to_string()),
        };
        assert_eq!(added.change_type(), "added");

        let removed = PropertyDiff {
            name: "OldProp".to_string(),
            old_value: Some("Value".to_string()),
            new_value: None,
        };
        assert_eq!(removed.change_type(), "removed");

        let modified = PropertyDiff {
            name: "ChangedProp".to_string(),
            old_value: Some("Old".to_string()),
            new_value: Some("New".to_string()),
        };
        assert_eq!(modified.change_type(), "modified");

        let unchanged = PropertyDiff {
            name: "WeirdProp".to_string(),
            old_value: None,
            new_value: None,
        };
        assert_eq!(unchanged.change_type(), "unchanged");
    }

    // ===========================================
    // Error Tests
    // ===========================================

    #[test]
    fn test_error_display() {
        let err = MsiError::TableNotFound("CustomTable".to_string());
        assert!(format!("{}", err).contains("CustomTable"));

        let err = MsiError::OpenError("invalid file".to_string());
        assert!(format!("{}", err).contains("invalid file"));

        let err = MsiError::Parse("syntax error".to_string());
        assert!(format!("{}", err).contains("syntax error"));
    }

    // ===========================================
    // Integration Tests (with mock data)
    // ===========================================

    #[test]
    fn test_row_serialization() {
        let row = Row {
            values: vec![
                CellValue::String("test".to_string()),
                CellValue::Integer(42),
                CellValue::Null,
            ],
        };

        let json = serde_json::to_string(&row).unwrap();
        let parsed: Row = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.values.len(), 3);
        assert_eq!(parsed.values[0], CellValue::String("test".to_string()));
        assert_eq!(parsed.values[1], CellValue::Integer(42));
        assert_eq!(parsed.values[2], CellValue::Null);
    }

    #[test]
    fn test_table_serialization() {
        let table = Table {
            name: "Test".to_string(),
            columns: vec![
                Column {
                    name: "Id".to_string(),
                    col_type: ColumnType::String,
                    nullable: false,
                    primary_key: true,
                },
            ],
            rows: vec![
                Row {
                    values: vec![CellValue::String("A".to_string())],
                },
            ],
        };

        let json = serde_json::to_string(&table).unwrap();
        let parsed: Table = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "Test");
        assert_eq!(parsed.columns.len(), 1);
        assert_eq!(parsed.rows.len(), 1);
    }

    #[test]
    fn test_summary_info_default() {
        let info = SummaryInfo::default();
        assert!(info.title.is_none());
        assert!(info.subject.is_none());
        assert!(info.author.is_none());
        assert!(info.keywords.is_none());
        assert!(info.comments.is_none());
        assert!(info.creating_app.is_none());
        assert!(info.uuid.is_none());
    }
}
