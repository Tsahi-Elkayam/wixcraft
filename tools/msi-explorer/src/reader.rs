//! MSI file reader

use crate::types::*;
use crate::{MsiError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// MSI file handle for reading and exploring
pub struct MsiFile {
    path: PathBuf,
    package: msi::Package<std::fs::File>,
}

impl MsiFile {
    /// Open an MSI file for reading
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = std::fs::File::open(&path)?;
        let package = msi::Package::open(file)
            .map_err(|e| MsiError::OpenError(e.to_string()))?;

        Ok(Self { path, package })
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get list of all table names
    pub fn table_names(&self) -> Vec<String> {
        self.package.tables()
            .map(|t| t.name().to_string())
            .collect()
    }

    /// Get tables grouped by category
    pub fn tables_by_category(&self) -> HashMap<TableCategory, Vec<String>> {
        let mut result: HashMap<TableCategory, Vec<String>> = HashMap::new();

        for name in self.table_names() {
            let category = TableCategory::from_table_name(&name);
            result.entry(category).or_default().push(name);
        }

        for tables in result.values_mut() {
            tables.sort();
        }

        result
    }

    /// Check if a table exists
    pub fn has_table(&self, name: &str) -> bool {
        self.package.tables().any(|t| t.name() == name)
    }

    /// Get a table by name
    pub fn get_table(&mut self, name: &str) -> Result<Table> {
        let table_def = self.package.tables()
            .find(|t| t.name() == name)
            .ok_or_else(|| MsiError::TableNotFound(name.to_string()))?;

        let columns: Vec<Column> = table_def.columns()
            .iter()
            .map(|col| Column {
                name: col.name().to_string(),
                col_type: ColumnType::from_msi(col.coltype()),
                nullable: col.is_nullable(),
                primary_key: col.is_primary_key(),
            })
            .collect();

        let rows: Vec<Row> = self.package.select_rows(msi::Select::table(name))
            .map_err(|e| MsiError::Parse(e.to_string()))?
            .map(|row| {
                let values: Vec<CellValue> = (0..columns.len())
                    .map(|i| {
                        let val = &row[i];
                        if val.is_null() {
                            CellValue::Null
                        } else if let Some(s) = val.as_str() {
                            CellValue::String(s.to_string())
                        } else if let Some(n) = val.as_int() {
                            CellValue::Integer(n)
                        } else {
                            CellValue::Null
                        }
                    })
                    .collect();
                Row { values }
            })
            .collect();

        Ok(Table {
            name: name.to_string(),
            columns,
            rows,
        })
    }

    /// Get summary information stream
    pub fn summary_info(&self) -> Result<SummaryInfo> {
        let info = self.package.summary_info();

        Ok(SummaryInfo {
            title: info.title().map(|s: &str| s.to_string()),
            subject: info.subject().map(|s: &str| s.to_string()),
            author: info.author().map(|s: &str| s.to_string()),
            keywords: None, // Not available in this API
            comments: info.comments().map(|s: &str| s.to_string()),
            creating_app: info.creating_application().map(|s: &str| s.to_string()),
            uuid: info.uuid().map(|u| u.to_string()),
        })
    }

    /// Get the value of a property
    pub fn get_property(&mut self, name: &str) -> Result<Option<String>> {
        let table = self.get_table("Property")?;

        for row in &table.rows {
            if let Some(CellValue::String(prop_name)) = row.values.first() {
                if prop_name == name {
                    if let Some(CellValue::String(value)) = row.values.get(1) {
                        return Ok(Some(value.clone()));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get common properties
    pub fn get_common_properties(&mut self) -> Result<HashMap<String, String>> {
        let mut props = HashMap::new();
        let common = [
            "ProductName", "ProductVersion", "ProductCode", "UpgradeCode",
            "Manufacturer", "ProductLanguage", "ALLUSERS",
        ];

        for name in common {
            if let Ok(Some(value)) = self.get_property(name) {
                props.insert(name.to_string(), value);
            }
        }

        Ok(props)
    }

    /// Get file statistics
    pub fn stats(&mut self) -> Result<MsiStats> {
        let table_names = self.table_names();
        let mut total_rows = 0;
        let mut largest_table = String::new();
        let mut largest_count = 0;

        for name in &table_names {
            if let Ok(table) = self.get_table(name) {
                total_rows += table.rows.len();
                if table.rows.len() > largest_count {
                    largest_count = table.rows.len();
                    largest_table = name.clone();
                }
            }
        }

        let file_meta = std::fs::metadata(&self.path)?;

        Ok(MsiStats {
            file_size: file_meta.len(),
            table_count: table_names.len(),
            total_rows,
            largest_table,
            largest_table_rows: largest_count,
        })
    }
}
