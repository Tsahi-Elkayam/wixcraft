//! Global search across MSI tables

use crate::reader::MsiFile;
use crate::Result;
use serde::{Deserialize, Serialize};

/// Search options
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub tables: Option<Vec<String>>,
    pub columns: Option<Vec<String>>,
    pub max_results: Option<usize>,
}

/// A single search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub table: String,
    pub column: String,
    pub row_index: usize,
    pub primary_key: String,
    pub value: String,
    pub match_start: usize,
    pub match_end: usize,
}

impl SearchResult {
    /// Get highlighted value with match marked
    pub fn highlighted(&self, before: &str, after: &str) -> String {
        format!(
            "{}{}{}{}{}",
            &self.value[..self.match_start],
            before,
            &self.value[self.match_start..self.match_end],
            after,
            &self.value[self.match_end..]
        )
    }
}

/// Search across all tables in an MSI file
pub fn search(msi: &mut MsiFile, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    let table_names = msi.table_names();

    let search_query = if options.case_sensitive {
        query.to_string()
    } else {
        query.to_lowercase()
    };

    for table_name in &table_names {
        if let Some(ref filter_tables) = options.tables {
            if !filter_tables.iter().any(|t| t.eq_ignore_ascii_case(table_name)) {
                continue;
            }
        }

        let table = match msi.get_table(table_name) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let pk_indices: Vec<usize> = table
            .columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.primary_key)
            .map(|(i, _)| i)
            .collect();

        for (row_idx, row) in table.rows.iter().enumerate() {
            let pk_value: String = pk_indices
                .iter()
                .filter_map(|&i| row.values.get(i))
                .map(|v| v.display())
                .collect::<Vec<_>>()
                .join(", ");

            for (col_idx, col) in table.columns.iter().enumerate() {
                if let Some(ref filter_cols) = options.columns {
                    if !filter_cols.iter().any(|c| c.eq_ignore_ascii_case(&col.name)) {
                        continue;
                    }
                }

                if let Some(value) = row.values.get(col_idx) {
                    let value_str = value.display();
                    let search_str = if options.case_sensitive {
                        value_str.clone()
                    } else {
                        value_str.to_lowercase()
                    };

                    if let Some(pos) = search_str.find(&search_query) {
                        results.push(SearchResult {
                            table: table_name.clone(),
                            column: col.name.clone(),
                            row_index: row_idx,
                            primary_key: pk_value.clone(),
                            value: value_str,
                            match_start: pos,
                            match_end: pos + query.len(),
                        });

                        if let Some(max) = options.max_results {
                            if results.len() >= max {
                                return Ok(results);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}
