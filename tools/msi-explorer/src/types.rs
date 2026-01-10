//! Common types for MSI exploration

use serde::{Deserialize, Serialize};

/// MSI table with columns and rows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

impl Table {
    /// Get column index by name
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }

    /// Get primary key column names
    pub fn primary_keys(&self) -> Vec<&str> {
        self.columns
            .iter()
            .filter(|c| c.primary_key)
            .map(|c| c.name.as_str())
            .collect()
    }

    /// Get row count
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get column count
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub col_type: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
}

impl Column {
    /// Check if this column references another table (foreign key)
    pub fn is_foreign_key(&self) -> bool {
        self.name.ends_with('_')
    }

    /// Get the referenced table name
    pub fn referenced_table(&self) -> Option<&str> {
        if self.name.ends_with('_') {
            Some(self.name.trim_end_matches('_'))
        } else {
            None
        }
    }
}

/// Column data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    String,
    Integer,
}

impl ColumnType {
    pub fn from_msi(t: msi::ColumnType) -> Self {
        match t {
            msi::ColumnType::Str(_) => ColumnType::String,
            _ => ColumnType::Integer,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ColumnType::String => "String",
            ColumnType::Integer => "Integer",
        }
    }
}

/// Table row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub values: Vec<CellValue>,
}

/// Cell value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CellValue {
    String(String),
    Integer(i32),
    Null,
}

impl CellValue {
    pub fn is_null(&self) -> bool {
        matches!(self, CellValue::Null)
    }

    pub fn display(&self) -> String {
        match self {
            CellValue::String(s) => s.clone(),
            CellValue::Integer(i) => i.to_string(),
            CellValue::Null => String::new(),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            CellValue::String(s) => Some(s),
            _ => None,
        }
    }
}

impl std::fmt::Display for CellValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// Summary information from MSI file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SummaryInfo {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub author: Option<String>,
    pub keywords: Option<String>,
    pub comments: Option<String>,
    pub creating_app: Option<String>,
    pub uuid: Option<String>,
}

impl SummaryInfo {
    pub fn platform(&self) -> Option<&str> {
        self.subject.as_ref().and_then(|s| {
            if s.contains("x64") || s.contains("Intel64") {
                Some("x64")
            } else if s.contains("Intel") || s.contains("x86") {
                Some("x86")
            } else if s.contains("Arm64") {
                Some("ARM64")
            } else {
                None
            }
        })
    }
}

/// MSI file statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiStats {
    pub file_size: u64,
    pub table_count: usize,
    pub total_rows: usize,
    pub largest_table: String,
    pub largest_table_rows: usize,
}

/// Table categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableCategory {
    Core,
    File,
    Registry,
    UI,
    CustomAction,
    Service,
    Sequence,
    Validation,
    Other,
}

impl TableCategory {
    pub fn from_table_name(name: &str) -> Self {
        match name {
            "Property" | "Component" | "Feature" | "Directory" | "Media" | "Upgrade" => {
                TableCategory::Core
            }
            "File" | "RemoveFile" | "DuplicateFile" | "MoveFile" | "IniFile" | "Environment" => {
                TableCategory::File
            }
            "Registry" | "RemoveRegistry" | "RegLocator" => TableCategory::Registry,
            "Dialog" | "Control" | "ControlEvent" | "TextStyle" | "UIText" | "RadioButton"
            | "ListBox" | "ComboBox" | "ListView" | "Billboard" => TableCategory::UI,
            "CustomAction" | "Binary" => TableCategory::CustomAction,
            "ServiceInstall" | "ServiceControl" | "ServiceConfigure" => TableCategory::Service,
            name if name.contains("Sequence") => TableCategory::Sequence,
            "_Validation" | "_Columns" | "_Tables" | "_Streams" | "_Storages" => {
                TableCategory::Validation
            }
            _ => TableCategory::Other,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TableCategory::Core => "Core",
            TableCategory::File => "Files",
            TableCategory::Registry => "Registry",
            TableCategory::UI => "User Interface",
            TableCategory::CustomAction => "Custom Actions",
            TableCategory::Service => "Services",
            TableCategory::Sequence => "Sequences",
            TableCategory::Validation => "Validation",
            TableCategory::Other => "Other",
        }
    }
}
