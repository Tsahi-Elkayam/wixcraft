//! WiX Knowledge Base Library
//!
//! Provides access to WiX Toolset metadata stored in a SQLite database.
//! Used by other wixcraft tools for autocomplete, linting, hover info, etc.

pub mod cache;
pub mod config;
pub mod db;
pub mod harvest;
pub mod models;

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default database filename
pub const DEFAULT_DB_NAME: &str = "wixkb.db";

/// Errors that can occur in wixkb
#[derive(Error, Debug)]
pub enum WixKbError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Database not initialized")]
    NotInitialized,
}

pub type Result<T> = std::result::Result<T, WixKbError>;

/// Main entry point for the WiX Knowledge Base
pub struct WixKb {
    db: db::Database,
    cache: cache::LruCache,
}

impl WixKb {
    /// Open an existing knowledge base
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = db::Database::open(path)?;
        let cache = cache::LruCache::new(1000);
        Ok(Self { db, cache })
    }

    /// Open the default knowledge base location
    pub fn open_default() -> Result<Self> {
        let path = default_db_path()?;
        Self::open(path)
    }

    /// Create a new knowledge base
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = db::Database::create(path)?;
        let cache = cache::LruCache::new(1000);
        Ok(Self { db, cache })
    }

    /// Get element by name
    pub fn get_element(&self, name: &str) -> Result<Option<models::Element>> {
        if let Some(elem) = self.cache.get_element(name) {
            return Ok(Some(elem));
        }
        self.db.get_element(name)
    }

    /// Search elements by prefix (for autocomplete)
    pub fn search_elements(&self, prefix: &str, limit: usize) -> Result<Vec<models::Element>> {
        self.db.search_elements(prefix, limit)
    }

    /// Full-text search elements
    pub fn search_elements_fts(&self, query: &str, limit: usize) -> Result<Vec<models::Element>> {
        self.db.search_elements_fts(query, limit)
    }

    /// Get attributes for an element
    pub fn get_attributes(&self, element_name: &str) -> Result<Vec<models::Attribute>> {
        self.db.get_attributes(element_name)
    }

    /// Get child elements for a parent
    pub fn get_children(&self, element_name: &str) -> Result<Vec<String>> {
        self.db.get_children(element_name)
    }

    /// Get parent elements for a child
    pub fn get_parents(&self, element_name: &str) -> Result<Vec<String>> {
        self.db.get_parents(element_name)
    }

    /// Get rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Result<Option<models::Rule>> {
        self.db.get_rule(rule_id)
    }

    /// Get all rules for a category
    pub fn get_rules_by_category(&self, category: &str) -> Result<Vec<models::Rule>> {
        self.db.get_rules_by_category(category)
    }

    /// Get all enabled rules
    pub fn get_enabled_rules(&self) -> Result<Vec<models::Rule>> {
        self.db.get_enabled_rules()
    }

    /// Search rules by text
    pub fn search_rules(&self, query: &str, limit: usize) -> Result<Vec<models::Rule>> {
        self.db.search_rules(query, limit)
    }

    /// Get error by code
    pub fn get_error(&self, code: &str) -> Result<Option<models::WixError>> {
        self.db.get_error(code)
    }

    /// Get ICE rule by code
    pub fn get_ice_rule(&self, code: &str) -> Result<Option<models::IceRule>> {
        self.db.get_ice_rule(code)
    }

    /// Get all ICE rules
    pub fn get_all_ice_rules(&self) -> Result<Vec<models::IceRule>> {
        self.db.get_all_ice_rules()
    }

    /// Get standard directory by name
    pub fn get_standard_directory(&self, name: &str) -> Result<Option<models::StandardDirectory>> {
        self.db.get_standard_directory(name)
    }

    /// Get all standard directories
    pub fn get_all_standard_directories(&self) -> Result<Vec<models::StandardDirectory>> {
        self.db.get_all_standard_directories()
    }

    /// Get builtin property by name
    pub fn get_builtin_property(&self, name: &str) -> Result<Option<models::BuiltinProperty>> {
        self.db.get_builtin_property(name)
    }

    /// Get snippets by prefix
    pub fn get_snippets(&self, prefix: &str) -> Result<Vec<models::Snippet>> {
        self.db.get_snippets(prefix)
    }

    /// Get all snippets
    pub fn get_all_snippets(&self) -> Result<Vec<models::Snippet>> {
        self.db.get_all_snippets()
    }

    /// Get keywords by category
    pub fn get_keywords(&self, category: &str) -> Result<Vec<String>> {
        self.db.get_keywords(category)
    }

    /// Get migration changes between versions
    pub fn get_migrations(&self, from: &str, to: &str) -> Result<Vec<models::Migration>> {
        self.db.get_migrations(from, to)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<models::DbStats> {
        self.db.get_stats()
    }

    /// Preload cache with common elements
    pub fn preload_cache(&mut self) -> Result<()> {
        let elements = self.db.get_common_elements(100)?;
        for elem in elements {
            self.cache.put_element(elem);
        }
        Ok(())
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get the underlying database connection (for advanced queries)
    pub fn db(&self) -> &db::Database {
        &self.db
    }
}

/// Get the default database path
pub fn default_db_path() -> Result<PathBuf> {
    // Try in order:
    // 1. WIXKB_DATABASE env var
    // 2. ./database/wixkb.db (current working directory)
    // 3. database/wixkb.db relative to executable
    // 4. ~/.wixcraft/wixkb.db

    if let Ok(path) = std::env::var("WIXKB_DATABASE") {
        return Ok(PathBuf::from(path));
    }

    // Check current working directory
    let cwd_path = PathBuf::from("database").join(DEFAULT_DB_NAME);
    if cwd_path.exists() {
        return Ok(cwd_path);
    }

    // Check relative to executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let db_path = parent.join("database").join(DEFAULT_DB_NAME);
            if db_path.exists() {
                return Ok(db_path);
            }
        }
    }

    // Check ~/.wixcraft/wixkb.db
    if let Some(home) = dirs::home_dir() {
        let home_path = home.join(".wixcraft").join(DEFAULT_DB_NAME);
        if home_path.exists() {
            return Ok(home_path);
        }
    }

    // Default to ~/.wixcraft/wixkb.db (for init)
    dirs::home_dir()
        .map(|h| h.join(".wixcraft").join(DEFAULT_DB_NAME))
        .ok_or_else(|| WixKbError::Config("Could not determine home directory".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_default_db_name() {
        assert_eq!(DEFAULT_DB_NAME, "wixkb.db");
    }
}
