//! MSI file comparison/diff

use crate::reader::MsiFile;
use crate::types::{CellValue, Table};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Difference between two MSI files
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MsiDiff {
    pub tables_only_in_first: Vec<String>,
    pub tables_only_in_second: Vec<String>,
    pub table_diffs: Vec<TableDiff>,
    pub property_diffs: Vec<PropertyDiff>,
}

impl MsiDiff {
    pub fn has_differences(&self) -> bool {
        !self.tables_only_in_first.is_empty()
            || !self.tables_only_in_second.is_empty()
            || !self.table_diffs.is_empty()
            || !self.property_diffs.is_empty()
    }

    pub fn change_count(&self) -> usize {
        let table_changes: usize = self.table_diffs.iter().map(|t| t.change_count()).sum();
        self.tables_only_in_first.len()
            + self.tables_only_in_second.len()
            + table_changes
            + self.property_diffs.len()
    }
}

/// Difference in a single table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDiff {
    pub table_name: String,
    pub rows_added: Vec<RowChange>,
    pub rows_removed: Vec<RowChange>,
    pub rows_modified: Vec<RowModification>,
}

impl TableDiff {
    pub fn change_count(&self) -> usize {
        self.rows_added.len() + self.rows_removed.len() + self.rows_modified.len()
    }

    pub fn is_empty(&self) -> bool {
        self.change_count() == 0
    }
}

/// A row change (added or removed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowChange {
    pub primary_key: String,
    pub values: HashMap<String, String>,
}

/// A modified row with specific cell changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowModification {
    pub primary_key: String,
    pub cell_changes: Vec<CellChange>,
}

/// Change to a single cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellChange {
    pub column: String,
    pub old_value: String,
    pub new_value: String,
}

/// Property difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDiff {
    pub name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

impl PropertyDiff {
    pub fn change_type(&self) -> &'static str {
        match (&self.old_value, &self.new_value) {
            (None, Some(_)) => "added",
            (Some(_), None) => "removed",
            (Some(_), Some(_)) => "modified",
            (None, None) => "unchanged",
        }
    }
}

/// Compare two MSI files
pub fn compare(msi1: &mut MsiFile, msi2: &mut MsiFile) -> Result<MsiDiff> {
    let mut diff = MsiDiff::default();

    let tables1: HashSet<String> = msi1.table_names().into_iter().collect();
    let tables2: HashSet<String> = msi2.table_names().into_iter().collect();

    diff.tables_only_in_first = tables1.difference(&tables2).cloned().collect();
    diff.tables_only_in_second = tables2.difference(&tables1).cloned().collect();

    let common_tables: HashSet<_> = tables1.intersection(&tables2).cloned().collect();

    for table_name in common_tables {
        let table1 = msi1.get_table(&table_name)?;
        let table2 = msi2.get_table(&table_name)?;

        let table_diff = compare_tables(&table1, &table2);
        if !table_diff.is_empty() {
            diff.table_diffs.push(table_diff);
        }
    }

    diff.property_diffs = compare_properties(msi1, msi2)?;

    Ok(diff)
}

fn compare_tables(table1: &Table, table2: &Table) -> TableDiff {
    let mut diff = TableDiff {
        table_name: table1.name.clone(),
        rows_added: Vec::new(),
        rows_removed: Vec::new(),
        rows_modified: Vec::new(),
    };

    let pk_indices1: Vec<usize> = table1
        .columns
        .iter()
        .enumerate()
        .filter(|(_, c)| c.primary_key)
        .map(|(i, _)| i)
        .collect();

    let pk_indices2: Vec<usize> = table2
        .columns
        .iter()
        .enumerate()
        .filter(|(_, c)| c.primary_key)
        .map(|(i, _)| i)
        .collect();

    let rows1: HashMap<String, &crate::types::Row> = table1
        .rows
        .iter()
        .map(|r| {
            let pk: String = pk_indices1
                .iter()
                .filter_map(|&i| r.values.get(i))
                .map(|v| v.display())
                .collect::<Vec<_>>()
                .join("|");
            (pk, r)
        })
        .collect();

    let rows2: HashMap<String, &crate::types::Row> = table2
        .rows
        .iter()
        .map(|r| {
            let pk: String = pk_indices2
                .iter()
                .filter_map(|&i| r.values.get(i))
                .map(|v| v.display())
                .collect::<Vec<_>>()
                .join("|");
            (pk, r)
        })
        .collect();

    // Find added rows
    for (pk, row) in &rows2 {
        if !rows1.contains_key(pk) {
            let values: HashMap<String, String> = table2
                .columns
                .iter()
                .zip(row.values.iter())
                .map(|(col, val)| (col.name.clone(), val.display()))
                .collect();
            diff.rows_added.push(RowChange {
                primary_key: pk.clone(),
                values,
            });
        }
    }

    // Find removed rows
    for (pk, row) in &rows1 {
        if !rows2.contains_key(pk) {
            let values: HashMap<String, String> = table1
                .columns
                .iter()
                .zip(row.values.iter())
                .map(|(col, val)| (col.name.clone(), val.display()))
                .collect();
            diff.rows_removed.push(RowChange {
                primary_key: pk.clone(),
                values,
            });
        }
    }

    // Find modified rows
    for (pk, row1) in &rows1 {
        if let Some(row2) = rows2.get(pk) {
            let mut cell_changes = Vec::new();

            for col in &table1.columns {
                if let Some(col2_idx) = table2.column_index(&col.name) {
                    if let Some(col1_idx) = table1.column_index(&col.name) {
                        let val1 = row1.values.get(col1_idx);
                        let val2 = row2.values.get(col2_idx);

                        if val1 != val2 {
                            cell_changes.push(CellChange {
                                column: col.name.clone(),
                                old_value: val1.map(|v| v.display()).unwrap_or_default(),
                                new_value: val2.map(|v| v.display()).unwrap_or_default(),
                            });
                        }
                    }
                }
            }

            if !cell_changes.is_empty() {
                diff.rows_modified.push(RowModification {
                    primary_key: pk.clone(),
                    cell_changes,
                });
            }
        }
    }

    diff
}

fn compare_properties(msi1: &mut MsiFile, msi2: &mut MsiFile) -> Result<Vec<PropertyDiff>> {
    let mut diffs = Vec::new();

    let props1 = get_all_properties(msi1)?;
    let props2 = get_all_properties(msi2)?;

    let all_names: HashSet<_> = props1.keys().chain(props2.keys()).collect();

    for name in all_names {
        let val1 = props1.get(name).cloned();
        let val2 = props2.get(name).cloned();

        if val1 != val2 {
            diffs.push(PropertyDiff {
                name: name.clone(),
                old_value: val1,
                new_value: val2,
            });
        }
    }

    Ok(diffs)
}

fn get_all_properties(msi: &mut MsiFile) -> Result<HashMap<String, String>> {
    let mut props = HashMap::new();

    if let Ok(table) = msi.get_table("Property") {
        for row in &table.rows {
            if let (Some(CellValue::String(name)), Some(value)) =
                (row.values.first(), row.values.get(1))
            {
                props.insert(name.clone(), value.display());
            }
        }
    }

    Ok(props)
}
