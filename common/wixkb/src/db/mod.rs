//! Database operations for WiX Knowledge Base

use crate::models::*;
use crate::Result;
use rusqlite::{Connection, OpenFlags, params};
use std::path::Path;

/// Schema SQL embedded at compile time
const SCHEMA_SQL: &str = include_str!("../../config/schema.sql");

/// Database wrapper
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open an existing database
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(Self { conn })
    }

    /// Create a new database with schema
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(Self { conn })
    }

    /// Open in-memory database (for testing)
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(Self { conn })
    }

    /// Get element by name
    pub fn get_element(&self, name: &str) -> Result<Option<Element>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, namespace, since_version, deprecated_version,
                    description, documentation_url, remarks, example
             FROM elements WHERE name = ?1 COLLATE NOCASE"
        )?;

        let result = stmt.query_row(params![name], |row| {
            Ok(Element {
                id: row.get(0)?,
                name: row.get(1)?,
                namespace: row.get(2)?,
                since_version: row.get(3)?,
                deprecated_version: row.get(4)?,
                description: row.get(5)?,
                documentation_url: row.get(6)?,
                remarks: row.get(7)?,
                example: row.get(8)?,
            })
        });

        match result {
            Ok(elem) => Ok(Some(elem)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Search elements by prefix
    pub fn search_elements(&self, prefix: &str, limit: usize) -> Result<Vec<Element>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, namespace, since_version, deprecated_version,
                    description, documentation_url, remarks, example
             FROM elements
             WHERE name LIKE ?1 || '%' COLLATE NOCASE
             ORDER BY name
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![prefix, limit as i64], |row| {
            Ok(Element {
                id: row.get(0)?,
                name: row.get(1)?,
                namespace: row.get(2)?,
                since_version: row.get(3)?,
                deprecated_version: row.get(4)?,
                description: row.get(5)?,
                documentation_url: row.get(6)?,
                remarks: row.get(7)?,
                example: row.get(8)?,
            })
        })?;

        let mut elements = Vec::new();
        for row in rows {
            elements.push(row?);
        }
        Ok(elements)
    }

    /// Full-text search elements
    pub fn search_elements_fts(&self, query: &str, limit: usize) -> Result<Vec<Element>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT e.id, e.name, e.namespace, e.since_version, e.deprecated_version,
                    e.description, e.documentation_url, e.remarks, e.example
             FROM elements e
             JOIN elements_fts fts ON e.id = fts.rowid
             WHERE elements_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![query, limit as i64], |row| {
            Ok(Element {
                id: row.get(0)?,
                name: row.get(1)?,
                namespace: row.get(2)?,
                since_version: row.get(3)?,
                deprecated_version: row.get(4)?,
                description: row.get(5)?,
                documentation_url: row.get(6)?,
                remarks: row.get(7)?,
                example: row.get(8)?,
            })
        })?;

        let mut elements = Vec::new();
        for row in rows {
            elements.push(row?);
        }
        Ok(elements)
    }

    /// Get common elements for cache preloading
    pub fn get_common_elements(&self, limit: usize) -> Result<Vec<Element>> {
        // Return most commonly used elements (by name, common ones first)
        let common_names = [
            "Package", "Component", "File", "Directory", "Feature",
            "Fragment", "Property", "RegistryKey", "RegistryValue",
            "Shortcut", "CustomAction", "ServiceInstall", "Bundle",
        ];

        let mut elements = Vec::new();

        for name in common_names.iter().take(limit) {
            if let Some(elem) = self.get_element(name)? {
                elements.push(elem);
            }
        }

        Ok(elements)
    }

    /// Get attributes for an element
    pub fn get_attributes(&self, element_name: &str) -> Result<Vec<Attribute>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT a.id, a.element_id, a.name, a.attr_type, a.required,
                    a.default_value, a.description, a.since_version, a.deprecated_version
             FROM attributes a
             JOIN elements e ON a.element_id = e.id
             WHERE e.name = ?1 COLLATE NOCASE
             ORDER BY a.required DESC, a.name"
        )?;

        let rows = stmt.query_map(params![element_name], |row| {
            let id: i64 = row.get(0)?;
            let attr_type_str: String = row.get(3)?;
            Ok((
                Attribute {
                    id,
                    element_id: row.get(1)?,
                    name: row.get(2)?,
                    attr_type: AttributeType::from(attr_type_str.as_str()),
                    required: row.get(4)?,
                    default_value: row.get(5)?,
                    description: row.get(6)?,
                    since_version: row.get(7)?,
                    deprecated_version: row.get(8)?,
                    enum_values: Vec::new(),
                },
                id,
            ))
        })?;

        let mut attributes = Vec::new();
        for row in rows {
            let (mut attr, attr_id) = row?;
            // Load enum values if attribute type is enum
            if attr.attr_type == AttributeType::Enum {
                attr.enum_values = self.get_enum_values(attr_id)?;
            }
            attributes.push(attr);
        }
        Ok(attributes)
    }

    /// Get enum values for an attribute
    fn get_enum_values(&self, attribute_id: i64) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT value FROM attribute_enum_values WHERE attribute_id = ?1 ORDER BY value"
        )?;

        let rows = stmt.query_map(params![attribute_id], |row| row.get(0))?;

        let mut values = Vec::new();
        for row in rows {
            values.push(row?);
        }
        Ok(values)
    }

    /// Get child elements for a parent
    pub fn get_children(&self, element_name: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT c.name
             FROM element_children ec
             JOIN elements p ON ec.element_id = p.id
             JOIN elements c ON ec.child_id = c.id
             WHERE p.name = ?1 COLLATE NOCASE
             ORDER BY c.name"
        )?;

        let rows = stmt.query_map(params![element_name], |row| row.get(0))?;

        let mut children = Vec::new();
        for row in rows {
            children.push(row?);
        }
        Ok(children)
    }

    /// Get parent elements for a child
    pub fn get_parents(&self, element_name: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT p.name
             FROM element_parents ep
             JOIN elements c ON ep.element_id = c.id
             JOIN elements p ON ep.parent_id = p.id
             WHERE c.name = ?1 COLLATE NOCASE
             ORDER BY p.name"
        )?;

        let rows = stmt.query_map(params![element_name], |row| row.get(0))?;

        let mut parents = Vec::new();
        for row in rows {
            parents.push(row?);
        }
        Ok(parents)
    }

    /// Get rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Result<Option<Rule>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, rule_id, category, severity, name, description,
                    rationale, fix_suggestion, enabled, auto_fixable,
                    condition, target_kind, target_name, tags
             FROM rules WHERE rule_id = ?1"
        )?;

        let result = stmt.query_row(params![rule_id], |row| {
            let severity_str: String = row.get(3)?;
            Ok(Rule {
                id: row.get(0)?,
                rule_id: row.get(1)?,
                category: row.get(2)?,
                severity: Severity::from(severity_str.as_str()),
                name: row.get(4)?,
                description: row.get(5)?,
                rationale: row.get(6)?,
                fix_suggestion: row.get(7)?,
                enabled: row.get(8)?,
                auto_fixable: row.get(9)?,
                conditions: Vec::new(),
                condition: row.get(10)?,
                target_kind: row.get(11)?,
                target_name: row.get(12)?,
                tags: row.get(13)?,
            })
        });

        match result {
            Ok(mut rule) => {
                rule.conditions = self.get_rule_conditions(rule.id)?;
                Ok(Some(rule))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get conditions for a rule
    fn get_rule_conditions(&self, rule_id: i64) -> Result<Vec<RuleCondition>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, condition_type, target, operator, value
             FROM rule_conditions WHERE rule_id = ?1"
        )?;

        let rows = stmt.query_map(params![rule_id], |row| {
            Ok(RuleCondition {
                id: row.get(0)?,
                condition_type: row.get(1)?,
                target: row.get(2)?,
                operator: row.get(3)?,
                value: row.get(4)?,
            })
        })?;

        let mut conditions = Vec::new();
        for row in rows {
            conditions.push(row?);
        }
        Ok(conditions)
    }

    /// Get all rules for a category
    pub fn get_rules_by_category(&self, category: &str) -> Result<Vec<Rule>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, rule_id, category, severity, name, description,
                    rationale, fix_suggestion, enabled, auto_fixable,
                    condition, target_kind, target_name, tags
             FROM rules WHERE category = ?1
             ORDER BY rule_id"
        )?;

        let rows = stmt.query_map(params![category], |row| {
            let severity_str: String = row.get(3)?;
            Ok(Rule {
                id: row.get(0)?,
                rule_id: row.get(1)?,
                category: row.get(2)?,
                severity: Severity::from(severity_str.as_str()),
                name: row.get(4)?,
                description: row.get(5)?,
                rationale: row.get(6)?,
                fix_suggestion: row.get(7)?,
                enabled: row.get(8)?,
                auto_fixable: row.get(9)?,
                conditions: Vec::new(),
                condition: row.get(10)?,
                target_kind: row.get(11)?,
                target_name: row.get(12)?,
                tags: row.get(13)?,
            })
        })?;

        let mut rules = Vec::new();
        for row in rows {
            let mut rule = row?;
            rule.conditions = self.get_rule_conditions(rule.id)?;
            rules.push(rule);
        }
        Ok(rules)
    }

    /// Get all enabled rules
    pub fn get_enabled_rules(&self) -> Result<Vec<Rule>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, rule_id, category, severity, name, description,
                    rationale, fix_suggestion, enabled, auto_fixable,
                    condition, target_kind, target_name, tags
             FROM rules WHERE enabled = 1
             ORDER BY category, rule_id"
        )?;

        let rows = stmt.query_map([], |row| {
            let severity_str: String = row.get(3)?;
            Ok(Rule {
                id: row.get(0)?,
                rule_id: row.get(1)?,
                category: row.get(2)?,
                severity: Severity::from(severity_str.as_str()),
                name: row.get(4)?,
                description: row.get(5)?,
                rationale: row.get(6)?,
                fix_suggestion: row.get(7)?,
                enabled: row.get(8)?,
                auto_fixable: row.get(9)?,
                conditions: Vec::new(),
                condition: row.get(10)?,
                target_kind: row.get(11)?,
                target_name: row.get(12)?,
                tags: row.get(13)?,
            })
        })?;

        let mut rules = Vec::new();
        for row in rows {
            let mut rule = row?;
            rule.conditions = self.get_rule_conditions(rule.id)?;
            rules.push(rule);
        }
        Ok(rules)
    }

    /// Search rules by text
    pub fn search_rules(&self, query: &str, limit: usize) -> Result<Vec<Rule>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT r.id, r.rule_id, r.category, r.severity, r.name, r.description,
                    r.rationale, r.fix_suggestion, r.enabled, r.auto_fixable,
                    r.condition, r.target_kind, r.target_name, r.tags
             FROM rules r
             JOIN rules_fts fts ON r.id = fts.rowid
             WHERE rules_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![query, limit as i64], |row| {
            let severity_str: String = row.get(3)?;
            Ok(Rule {
                id: row.get(0)?,
                rule_id: row.get(1)?,
                category: row.get(2)?,
                severity: Severity::from(severity_str.as_str()),
                name: row.get(4)?,
                description: row.get(5)?,
                rationale: row.get(6)?,
                fix_suggestion: row.get(7)?,
                enabled: row.get(8)?,
                auto_fixable: row.get(9)?,
                conditions: Vec::new(),
                condition: row.get(10)?,
                target_kind: row.get(11)?,
                target_name: row.get(12)?,
                tags: row.get(13)?,
            })
        })?;

        let mut rules = Vec::new();
        for row in rows {
            rules.push(row?);
        }
        Ok(rules)
    }

    /// Get error by code
    pub fn get_error(&self, code: &str) -> Result<Option<WixError>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, code, severity, message_template, description, resolution, documentation_url
             FROM errors WHERE code = ?1"
        )?;

        let result = stmt.query_row(params![code], |row| {
            let severity_str: String = row.get(2)?;
            Ok(WixError {
                id: row.get(0)?,
                code: row.get(1)?,
                severity: Severity::from(severity_str.as_str()),
                message_template: row.get(3)?,
                description: row.get(4)?,
                resolution: row.get(5)?,
                documentation_url: row.get(6)?,
            })
        });

        match result {
            Ok(err) => Ok(Some(err)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get ICE rule by code
    pub fn get_ice_rule(&self, code: &str) -> Result<Option<IceRule>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, code, severity, description, resolution, tables_affected, documentation_url
             FROM ice_rules WHERE code = ?1"
        )?;

        let result = stmt.query_row(params![code], |row| {
            let severity_str: String = row.get(2)?;
            let tables_str: Option<String> = row.get(5)?;
            let tables = tables_str
                .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                .unwrap_or_default();

            Ok(IceRule {
                id: row.get(0)?,
                code: row.get(1)?,
                severity: Severity::from(severity_str.as_str()),
                description: row.get(3)?,
                resolution: row.get(4)?,
                tables_affected: tables,
                documentation_url: row.get(6)?,
            })
        });

        match result {
            Ok(rule) => Ok(Some(rule)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all ICE rules
    pub fn get_all_ice_rules(&self) -> Result<Vec<IceRule>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, code, severity, description, resolution, tables_affected, documentation_url
             FROM ice_rules ORDER BY code"
        )?;

        let rows = stmt.query_map([], |row| {
            let severity_str: String = row.get(2)?;
            let tables_str: Option<String> = row.get(5)?;
            let tables = tables_str
                .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                .unwrap_or_default();

            Ok(IceRule {
                id: row.get(0)?,
                code: row.get(1)?,
                severity: Severity::from(severity_str.as_str()),
                description: row.get(3)?,
                resolution: row.get(4)?,
                tables_affected: tables,
                documentation_url: row.get(6)?,
            })
        })?;

        let mut rules = Vec::new();
        for row in rows {
            rules.push(row?);
        }
        Ok(rules)
    }

    /// Get standard directory by name
    pub fn get_standard_directory(&self, name: &str) -> Result<Option<StandardDirectory>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, description, windows_path, example
             FROM standard_directories WHERE name = ?1"
        )?;

        let result = stmt.query_row(params![name], |row| {
            Ok(StandardDirectory {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                windows_path: row.get(3)?,
                example: row.get(4)?,
            })
        });

        match result {
            Ok(dir) => Ok(Some(dir)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all standard directories
    pub fn get_all_standard_directories(&self) -> Result<Vec<StandardDirectory>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, description, windows_path, example
             FROM standard_directories ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(StandardDirectory {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                windows_path: row.get(3)?,
                example: row.get(4)?,
            })
        })?;

        let mut dirs = Vec::new();
        for row in rows {
            dirs.push(row?);
        }
        Ok(dirs)
    }

    /// Get builtin property by name
    pub fn get_builtin_property(&self, name: &str) -> Result<Option<BuiltinProperty>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, property_type, description, default_value, readonly
             FROM builtin_properties WHERE name = ?1"
        )?;

        let result = stmt.query_row(params![name], |row| {
            Ok(BuiltinProperty {
                id: row.get(0)?,
                name: row.get(1)?,
                property_type: row.get(2)?,
                description: row.get(3)?,
                default_value: row.get(4)?,
                readonly: row.get(5)?,
            })
        });

        match result {
            Ok(prop) => Ok(Some(prop)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get snippets by prefix
    pub fn get_snippets(&self, prefix: &str) -> Result<Vec<Snippet>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, prefix, name, description, body, scope
             FROM snippets WHERE prefix LIKE ?1 || '%'
             ORDER BY prefix"
        )?;

        let rows = stmt.query_map(params![prefix], |row| {
            Ok(Snippet {
                id: row.get(0)?,
                prefix: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                body: row.get(4)?,
                scope: row.get(5)?,
            })
        })?;

        let mut snippets = Vec::new();
        for row in rows {
            snippets.push(row?);
        }
        Ok(snippets)
    }

    /// Get all snippets
    pub fn get_all_snippets(&self) -> Result<Vec<Snippet>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, prefix, name, description, body, scope
             FROM snippets ORDER BY prefix"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Snippet {
                id: row.get(0)?,
                prefix: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                body: row.get(4)?,
                scope: row.get(5)?,
            })
        })?;

        let mut snippets = Vec::new();
        for row in rows {
            snippets.push(row?);
        }
        Ok(snippets)
    }

    /// Get keywords by category
    pub fn get_keywords(&self, category: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT word FROM keywords WHERE category = ?1 ORDER BY word"
        )?;

        let rows = stmt.query_map(params![category], |row| row.get(0))?;

        let mut keywords = Vec::new();
        for row in rows {
            keywords.push(row?);
        }
        Ok(keywords)
    }

    /// Get migration changes between versions
    pub fn get_migrations(&self, from: &str, to: &str) -> Result<Vec<Migration>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, from_version, to_version, change_type, old_value, new_value, notes
             FROM migrations WHERE from_version = ?1 AND to_version = ?2
             ORDER BY change_type, old_value"
        )?;

        let rows = stmt.query_map(params![from, to], |row| {
            Ok(Migration {
                id: row.get(0)?,
                from_version: row.get(1)?,
                to_version: row.get(2)?,
                change_type: row.get(3)?,
                old_value: row.get(4)?,
                new_value: row.get(5)?,
                notes: row.get(6)?,
            })
        })?;

        let mut migrations = Vec::new();
        for row in rows {
            migrations.push(row?);
        }
        Ok(migrations)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<DbStats> {
        let elements: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM elements", [], |row| row.get(0)
        )?;
        let attributes: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM attributes", [], |row| row.get(0)
        )?;
        let rules: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM rules", [], |row| row.get(0)
        )?;
        let errors: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM errors", [], |row| row.get(0)
        )?;
        let ice_rules: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM ice_rules", [], |row| row.get(0)
        )?;
        let msi_tables: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM msi_tables", [], |row| row.get(0)
        )?;
        let snippets: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM snippets", [], |row| row.get(0)
        )?;
        let keywords: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM keywords", [], |row| row.get(0)
        )?;

        let schema_version: String = self.conn.query_row(
            "SELECT value FROM metadata WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        ).unwrap_or_else(|_| "unknown".to_string());

        let last_updated: Option<String> = self.conn.query_row(
            "SELECT value FROM metadata WHERE key = 'last_updated'",
            [],
            |row| row.get(0),
        ).ok();

        Ok(DbStats {
            elements,
            attributes,
            rules,
            errors,
            ice_rules,
            msi_tables,
            snippets,
            keywords,
            schema_version,
            last_updated,
        })
    }

    /// Insert an element
    pub fn insert_element(&self, element: &Element) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO elements (name, namespace, since_version, deprecated_version,
                                              description, documentation_url, remarks, example)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                element.name,
                element.namespace,
                element.since_version,
                element.deprecated_version,
                element.description,
                element.documentation_url,
                element.remarks,
                element.example,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert an attribute
    pub fn insert_attribute(&self, attr: &Attribute) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO attributes (element_id, name, attr_type, required,
                                     default_value, description, since_version, deprecated_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                attr.element_id,
                attr.name,
                attr.attr_type.to_string(),
                attr.required,
                attr.default_value,
                attr.description,
                attr.since_version,
                attr.deprecated_version,
            ],
        )?;
        let attr_id = self.conn.last_insert_rowid();

        // Insert enum values if any
        for value in &attr.enum_values {
            self.conn.execute(
                "INSERT INTO attribute_enum_values (attribute_id, value) VALUES (?1, ?2)",
                params![attr_id, value],
            )?;
        }

        Ok(attr_id)
    }

    /// Insert a rule
    pub fn insert_rule(&self, rule: &Rule) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO rules (rule_id, category, severity, name, description,
                               rationale, fix_suggestion, enabled, auto_fixable,
                               condition, target_kind, target_name, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                rule.rule_id,
                rule.category,
                rule.severity.to_string(),
                rule.name,
                rule.description,
                rule.rationale,
                rule.fix_suggestion,
                rule.enabled,
                rule.auto_fixable,
                rule.condition,
                rule.target_kind,
                rule.target_name,
                rule.tags,
            ],
        )?;
        let rule_db_id = self.conn.last_insert_rowid();

        // Insert conditions
        for condition in &rule.conditions {
            self.conn.execute(
                "INSERT INTO rule_conditions (rule_id, condition_type, target, operator, value)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    rule_db_id,
                    condition.condition_type,
                    condition.target,
                    condition.operator,
                    condition.value,
                ],
            )?;
        }

        Ok(rule_db_id)
    }

    /// Add a child element relationship
    pub fn add_element_child(&self, parent_name: &str, child_name: &str) -> Result<()> {
        // Get parent ID
        let parent_id: Option<i64> = self.conn.query_row(
            "SELECT id FROM elements WHERE name = ?1 COLLATE NOCASE",
            params![parent_name],
            |row| row.get(0),
        ).ok();

        // Get child ID
        let child_id: Option<i64> = self.conn.query_row(
            "SELECT id FROM elements WHERE name = ?1 COLLATE NOCASE",
            params![child_name],
            |row| row.get(0),
        ).ok();

        // Insert relationship if both exist
        if let (Some(pid), Some(cid)) = (parent_id, child_id) {
            self.conn.execute(
                "INSERT OR IGNORE INTO element_children (element_id, child_id) VALUES (?1, ?2)",
                params![pid, cid],
            )?;
            self.conn.execute(
                "INSERT OR IGNORE INTO element_parents (element_id, parent_id) VALUES (?1, ?2)",
                params![cid, pid],
            )?;
        }
        Ok(())
    }

    /// Update last_updated metadata
    pub fn set_last_updated(&self) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('last_updated', datetime('now'))",
            [],
        )?;
        Ok(())
    }

    /// Get raw connection for advanced operations
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_memory_db() {
        let db = Database::open_memory().unwrap();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.elements, 0);
        assert_eq!(stats.schema_version, "1.0.0");
    }

    #[test]
    fn test_insert_and_get_element() {
        let db = Database::open_memory().unwrap();

        let elem = Element {
            id: 0,
            name: "Package".to_string(),
            namespace: "wix".to_string(),
            since_version: Some("v4".to_string()),
            deprecated_version: None,
            description: Some("Root package element".to_string()),
            documentation_url: Some("https://wixtoolset.org/docs/schema/wxs/package/".to_string()),
            remarks: None,
            example: None,
        };

        let id = db.insert_element(&elem).unwrap();
        assert!(id > 0);

        let retrieved = db.get_element("Package").unwrap().unwrap();
        assert_eq!(retrieved.name, "Package");
        assert_eq!(retrieved.namespace, "wix");
    }

    #[test]
    fn test_search_elements() {
        let db = Database::open_memory().unwrap();

        for name in ["Package", "Property", "PropertyRef", "Component"] {
            let elem = Element {
                id: 0,
                name: name.to_string(),
                namespace: "wix".to_string(),
                since_version: None,
                deprecated_version: None,
                description: None,
                documentation_url: None,
                remarks: None,
                example: None,
            };
            db.insert_element(&elem).unwrap();
        }

        let results = db.search_elements("Pro", 10).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|e| e.name == "Property"));
        assert!(results.iter().any(|e| e.name == "PropertyRef"));
    }

    #[test]
    fn test_get_stats() {
        let db = Database::open_memory().unwrap();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.schema_version, "1.0.0");
    }
}
