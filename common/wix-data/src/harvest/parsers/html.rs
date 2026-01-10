//! HTML parser for documentation sources

use crate::{Result, WixDataError};
use scraper::{Html, Selector};

/// Parsed documentation for an element or concept
#[derive(Debug, Clone, Default)]
pub struct DocEntry {
    pub name: String,
    pub description: Option<String>,
    pub remarks: Option<String>,
    pub example: Option<String>,
}

/// Parse WiX documentation page for element details
pub fn parse_wix_element_doc(html: &str, element_name: &str) -> Result<DocEntry> {
    let document = Html::parse_document(html);

    let mut entry = DocEntry {
        name: element_name.to_string(),
        ..Default::default()
    };

    // Try to find description - common patterns in WiX docs
    if let Ok(selector) = Selector::parse(".description, .summary, p.lead") {
        if let Some(elem) = document.select(&selector).next() {
            entry.description = Some(elem.text().collect::<String>().trim().to_string());
        }
    }

    // Try to find remarks
    if let Ok(selector) = Selector::parse(".remarks, #remarks + p, h2:contains('Remarks') + p") {
        if let Some(elem) = document.select(&selector).next() {
            entry.remarks = Some(elem.text().collect::<String>().trim().to_string());
        }
    }

    // Try to find example code
    if let Ok(selector) = Selector::parse("pre code, .example code, #example pre") {
        if let Some(elem) = document.select(&selector).next() {
            entry.example = Some(elem.text().collect::<String>().trim().to_string());
        }
    }

    Ok(entry)
}

/// Parse ICE rules from MSDN documentation
pub fn parse_ice_rules(html: &str) -> Result<Vec<IceRule>> {
    let document = Html::parse_document(html);
    let mut rules = Vec::new();

    // MSDN typically lists ICE rules in a table or definition list
    // Pattern: ICE## - Description
    let ice_pattern = regex::Regex::new(r"ICE(\d+)")
        .map_err(|e| WixDataError::Parse(e.to_string()))?;

    // Try table rows first
    if let Ok(row_selector) = Selector::parse("table tr, .ice-list li") {
        for row in document.select(&row_selector) {
            let text = row.text().collect::<String>();
            if let Some(caps) = ice_pattern.captures(&text) {
                let code = format!("ICE{}", &caps[1]);
                let description = text.replace(&code, "").trim().to_string();

                if !description.is_empty() {
                    rules.push(IceRule {
                        code,
                        severity: "error".to_string(),
                        description: Some(description),
                        tables_affected: Vec::new(),
                    });
                }
            }
        }
    }

    // Also try links that contain ICE references
    if let Ok(link_selector) = Selector::parse("a[href*='ice']") {
        for link in document.select(&link_selector) {
            let text = link.text().collect::<String>();
            if let Some(caps) = ice_pattern.captures(&text) {
                let code = format!("ICE{}", &caps[1]);
                // Check if we already have this rule
                if !rules.iter().any(|r| r.code == code) {
                    rules.push(IceRule {
                        code,
                        severity: "error".to_string(),
                        description: None,
                        tables_affected: Vec::new(),
                    });
                }
            }
        }
    }

    Ok(rules)
}

/// Parse MSI table documentation
pub fn parse_msi_tables(html: &str) -> Result<Vec<MsiTable>> {
    let document = Html::parse_document(html);
    let mut tables = Vec::new();

    // MSDN lists MSI tables with links
    if let Ok(selector) = Selector::parse("a[href*='database-tables'], table tr td a") {
        for link in document.select(&selector) {
            let name = link.text().collect::<String>().trim().to_string();
            // Filter to table names (typically PascalCase, no spaces)
            if !name.is_empty() && !name.contains(' ') && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                if !tables.iter().any(|t: &MsiTable| t.name == name) {
                    tables.push(MsiTable {
                        name,
                        description: None,
                        required: false,
                    });
                }
            }
        }
    }

    Ok(tables)
}

/// Parse standard directories from WiX documentation
pub fn parse_standard_directories(html: &str) -> Result<Vec<StandardDir>> {
    let document = Html::parse_document(html);
    let mut dirs = Vec::new();

    // Look for directory names in tables or definition lists
    let dir_pattern = regex::Regex::new(r"([A-Z][a-zA-Z]+Folder|WindowsVolume)")
        .map_err(|e| WixDataError::Parse(e.to_string()))?;

    if let Ok(selector) = Selector::parse("table tr, dt, code") {
        for elem in document.select(&selector) {
            let text = elem.text().collect::<String>();
            for caps in dir_pattern.captures_iter(&text) {
                let name = caps[0].to_string();
                if !dirs.iter().any(|d: &StandardDir| d.name == name) {
                    dirs.push(StandardDir {
                        name,
                        description: None,
                    });
                }
            }
        }
    }

    Ok(dirs)
}

/// Parse built-in properties from MSI documentation
pub fn parse_builtin_properties(html: &str) -> Result<Vec<BuiltinProp>> {
    let document = Html::parse_document(html);
    let mut props = Vec::new();

    // MSI properties are typically ALLCAPS
    let prop_pattern = regex::Regex::new(r"\b([A-Z][A-Z0-9_]{2,})\b")
        .map_err(|e| WixDataError::Parse(e.to_string()))?;

    // Known MSI property names to filter
    let known_props = [
        "ALLUSERS", "ARPINSTALLLOCATION", "ARPPRODUCTICON", "ARPNOMODIFY",
        "ARPNOREMOVE", "ARPNOREPAIR", "INSTALLLEVEL", "INSTALLDIR",
        "ProductName", "ProductCode", "ProductVersion", "Manufacturer",
        "UpgradeCode", "VersionNT", "VersionNT64", "WindowsBuild",
    ];

    if let Ok(selector) = Selector::parse("table tr td, dt, code") {
        for elem in document.select(&selector) {
            let text = elem.text().collect::<String>();
            for caps in prop_pattern.captures_iter(&text) {
                let name = caps[0].to_string();
                if known_props.contains(&name.as_str()) && !props.iter().any(|p: &BuiltinProp| p.name == name) {
                    props.push(BuiltinProp {
                        name,
                        description: None,
                    });
                }
            }
        }
    }

    Ok(props)
}

/// ICE rule parsed from HTML
#[derive(Debug, Clone)]
pub struct IceRule {
    pub code: String,
    pub severity: String,
    pub description: Option<String>,
    pub tables_affected: Vec<String>,
}

/// MSI table parsed from HTML
#[derive(Debug, Clone)]
pub struct MsiTable {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

/// Standard directory parsed from HTML
#[derive(Debug, Clone)]
pub struct StandardDir {
    pub name: String,
    pub description: Option<String>,
}

/// Built-in property parsed from HTML
#[derive(Debug, Clone)]
pub struct BuiltinProp {
    pub name: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ice_rules_from_table() {
        let html = r#"
        <html><body>
        <table>
            <tr><td>ICE01</td><td>Validates the database</td></tr>
            <tr><td>ICE02</td><td>Checks for circular references</td></tr>
        </table>
        </body></html>
        "#;

        let rules = parse_ice_rules(html).unwrap();
        assert!(rules.len() >= 2);
        assert!(rules.iter().any(|r| r.code == "ICE01"));
        assert!(rules.iter().any(|r| r.code == "ICE02"));
    }

    #[test]
    fn test_parse_standard_directories() {
        let html = r#"
        <html><body>
        <table>
            <tr><td><code>ProgramFilesFolder</code></td><td>Program Files</td></tr>
            <tr><td><code>SystemFolder</code></td><td>System32</td></tr>
        </table>
        </body></html>
        "#;

        let dirs = parse_standard_directories(html).unwrap();
        assert!(dirs.iter().any(|d| d.name == "ProgramFilesFolder"));
        assert!(dirs.iter().any(|d| d.name == "SystemFolder"));
    }

    #[test]
    fn test_parse_msi_tables() {
        let html = r#"
        <html><body>
        <table>
            <tr><td><a href="/database-tables/component">Component</a></td></tr>
            <tr><td><a href="/database-tables/file">File</a></td></tr>
        </table>
        </body></html>
        "#;

        let tables = parse_msi_tables(html).unwrap();
        assert!(tables.iter().any(|t| t.name == "Component"));
        assert!(tables.iter().any(|t| t.name == "File"));
    }

    #[test]
    fn test_parse_wix_element_doc() {
        let html = r#"
        <html><body>
        <p class="description">This is the component element.</p>
        <pre><code>&lt;Component Id="MyComp" /&gt;</code></pre>
        </body></html>
        "#;

        let doc = parse_wix_element_doc(html, "Component").unwrap();
        assert_eq!(doc.name, "Component");
        assert!(doc.description.is_some());
    }
}
