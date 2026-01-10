//! Export MSI data to various formats

use crate::reader::MsiFile;
use crate::types::{CellValue, Table};
use crate::Result;
use serde_json::{json, Value as JsonValue};
use std::io::Write;

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
    Sql,
}

/// Export a single table to JSON
pub fn table_to_json(table: &Table) -> JsonValue {
    let rows: Vec<JsonValue> = table
        .rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (col, val) in table.columns.iter().zip(row.values.iter()) {
                let json_val = match val {
                    CellValue::String(s) => JsonValue::String(s.clone()),
                    CellValue::Integer(i) => JsonValue::Number((*i).into()),
                    CellValue::Null => JsonValue::Null,
                };
                obj.insert(col.name.clone(), json_val);
            }
            JsonValue::Object(obj)
        })
        .collect();

    json!({
        "name": table.name,
        "columns": table.columns.iter().map(|c| json!({
            "name": c.name,
            "type": c.col_type.display_name(),
            "nullable": c.nullable,
            "primary_key": c.primary_key,
        })).collect::<Vec<_>>(),
        "rows": rows,
        "row_count": table.rows.len(),
    })
}

/// Export a table to CSV
pub fn table_to_csv<W: Write>(table: &Table, writer: &mut W) -> Result<()> {
    let headers: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
    writeln!(writer, "{}", headers.join(","))?;

    for row in &table.rows {
        let values: Vec<String> = row
            .values
            .iter()
            .map(|v| {
                let s = v.display();
                if s.contains(',') || s.contains('"') || s.contains('\n') {
                    format!("\"{}\"", s.replace('"', "\"\""))
                } else {
                    s
                }
            })
            .collect();
        writeln!(writer, "{}", values.join(","))?;
    }

    Ok(())
}

/// Export a table to SQL INSERT statements
pub fn table_to_sql<W: Write>(table: &Table, writer: &mut W) -> Result<()> {
    let columns: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
    let columns_str = columns.join(", ");

    for row in &table.rows {
        let values: Vec<String> = row
            .values
            .iter()
            .map(|v| match v {
                CellValue::String(s) => format!("'{}'", s.replace('\'', "''")),
                CellValue::Integer(i) => i.to_string(),
                CellValue::Null => "NULL".to_string(),
            })
            .collect();

        writeln!(
            writer,
            "INSERT INTO {} ({}) VALUES ({});",
            table.name,
            columns_str,
            values.join(", ")
        )?;
    }

    Ok(())
}

/// Export entire MSI to JSON
pub fn msi_to_json(msi: &mut MsiFile) -> Result<JsonValue> {
    let table_names = msi.table_names();
    let summary = msi.summary_info()?;
    let stats = msi.stats()?;
    let props = msi.get_common_properties()?;

    let tables: Vec<JsonValue> = table_names
        .iter()
        .filter_map(|name| msi.get_table(name).ok())
        .map(|t| table_to_json(&t))
        .collect();

    Ok(json!({
        "summary": {
            "title": summary.title,
            "subject": summary.subject,
            "author": summary.author,
            "comments": summary.comments,
            "uuid": summary.uuid,
        },
        "properties": props,
        "stats": {
            "file_size": stats.file_size,
            "table_count": stats.table_count,
            "total_rows": stats.total_rows,
        },
        "tables": tables,
    }))
}
