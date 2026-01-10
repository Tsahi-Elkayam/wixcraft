-- WiX Knowledge Base Schema
-- Version: 1.0.0

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- Metadata table for database versioning
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT DEFAULT (datetime('now'))
);

INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', '1.0.0');
INSERT OR REPLACE INTO metadata (key, value) VALUES ('created_at', datetime('now'));

-- Sources tracking for incremental updates
CREATE TABLE IF NOT EXISTS sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    url TEXT,
    source_type TEXT NOT NULL, -- xsd, documentation, ice, msi
    last_harvested TEXT,
    content_hash TEXT,
    enabled INTEGER DEFAULT 1
);

-- WiX Elements
CREATE TABLE IF NOT EXISTS elements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    namespace TEXT DEFAULT 'wix',
    since_version TEXT DEFAULT 'v4',
    deprecated_version TEXT,
    description TEXT,
    documentation_url TEXT,
    remarks TEXT,
    example TEXT
);

-- Full-text search for elements
CREATE VIRTUAL TABLE IF NOT EXISTS elements_fts USING fts5(
    name,
    description,
    remarks,
    content=elements,
    content_rowid=id
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS elements_ai AFTER INSERT ON elements BEGIN
    INSERT INTO elements_fts(rowid, name, description, remarks)
    VALUES (new.id, new.name, new.description, new.remarks);
END;

CREATE TRIGGER IF NOT EXISTS elements_ad AFTER DELETE ON elements BEGIN
    INSERT INTO elements_fts(elements_fts, rowid, name, description, remarks)
    VALUES ('delete', old.id, old.name, old.description, old.remarks);
END;

CREATE TRIGGER IF NOT EXISTS elements_au AFTER UPDATE ON elements BEGIN
    INSERT INTO elements_fts(elements_fts, rowid, name, description, remarks)
    VALUES ('delete', old.id, old.name, old.description, old.remarks);
    INSERT INTO elements_fts(rowid, name, description, remarks)
    VALUES (new.id, new.name, new.description, new.remarks);
END;

-- Element parent-child relationships
CREATE TABLE IF NOT EXISTS element_parents (
    element_id INTEGER NOT NULL,
    parent_id INTEGER NOT NULL,
    PRIMARY KEY (element_id, parent_id),
    FOREIGN KEY (element_id) REFERENCES elements(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES elements(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS element_children (
    element_id INTEGER NOT NULL,
    child_id INTEGER NOT NULL,
    min_occurs INTEGER DEFAULT 0,
    max_occurs INTEGER, -- NULL means unbounded
    PRIMARY KEY (element_id, child_id),
    FOREIGN KEY (element_id) REFERENCES elements(id) ON DELETE CASCADE,
    FOREIGN KEY (child_id) REFERENCES elements(id) ON DELETE CASCADE
);

-- Attributes
CREATE TABLE IF NOT EXISTS attributes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    element_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    attr_type TEXT NOT NULL, -- string, guid, yesno, integer, enum, version, identifier
    required INTEGER DEFAULT 0,
    default_value TEXT,
    description TEXT,
    since_version TEXT DEFAULT 'v4',
    deprecated_version TEXT,
    UNIQUE (element_id, name),
    FOREIGN KEY (element_id) REFERENCES elements(id) ON DELETE CASCADE
);

-- Enum values for enum-type attributes
CREATE TABLE IF NOT EXISTS attribute_enum_values (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    attribute_id INTEGER NOT NULL,
    value TEXT NOT NULL,
    description TEXT,
    UNIQUE (attribute_id, value),
    FOREIGN KEY (attribute_id) REFERENCES attributes(id) ON DELETE CASCADE
);

-- Lint Rules
CREATE TABLE IF NOT EXISTS rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id TEXT NOT NULL UNIQUE, -- e.g., WIX001, COMP001
    category TEXT NOT NULL, -- component, file, directory, feature, property, customaction, service, bundle, registry, package
    severity TEXT NOT NULL, -- error, warning, info
    name TEXT NOT NULL,
    description TEXT,
    rationale TEXT,
    fix_suggestion TEXT,
    enabled INTEGER DEFAULT 1,
    auto_fixable INTEGER DEFAULT 0
);

-- Full-text search for rules
CREATE VIRTUAL TABLE IF NOT EXISTS rules_fts USING fts5(
    rule_id,
    name,
    description,
    rationale,
    content=rules,
    content_rowid=id
);

CREATE TRIGGER IF NOT EXISTS rules_ai AFTER INSERT ON rules BEGIN
    INSERT INTO rules_fts(rowid, rule_id, name, description, rationale)
    VALUES (new.id, new.rule_id, new.name, new.description, new.rationale);
END;

CREATE TRIGGER IF NOT EXISTS rules_ad AFTER DELETE ON rules BEGIN
    INSERT INTO rules_fts(rules_fts, rowid, rule_id, name, description, rationale)
    VALUES ('delete', old.id, old.rule_id, old.name, old.description, old.rationale);
END;

CREATE TRIGGER IF NOT EXISTS rules_au AFTER UPDATE ON rules BEGIN
    INSERT INTO rules_fts(rules_fts, rowid, rule_id, name, description, rationale)
    VALUES ('delete', old.id, old.rule_id, old.name, old.description, old.rationale);
    INSERT INTO rules_fts(rowid, rule_id, name, description, rationale)
    VALUES (new.id, new.rule_id, new.name, new.description, new.rationale);
END;

-- Rule conditions (XPath-like patterns)
CREATE TABLE IF NOT EXISTS rule_conditions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id INTEGER NOT NULL,
    condition_type TEXT NOT NULL, -- element, attribute, pattern, xpath
    target TEXT NOT NULL,
    operator TEXT, -- equals, not_equals, matches, exists, not_exists, contains
    value TEXT,
    FOREIGN KEY (rule_id) REFERENCES rules(id) ON DELETE CASCADE
);

-- WiX Errors and Warnings
CREATE TABLE IF NOT EXISTS errors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE, -- WIX0001, LGHT0001, etc.
    severity TEXT NOT NULL, -- error, warning
    message_template TEXT NOT NULL,
    description TEXT,
    resolution TEXT,
    documentation_url TEXT
);

-- ICE Rules
CREATE TABLE IF NOT EXISTS ice_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT NOT NULL UNIQUE, -- ICE01, ICE02, etc.
    severity TEXT NOT NULL,
    description TEXT,
    resolution TEXT,
    tables_affected TEXT, -- comma-separated list
    documentation_url TEXT
);

-- MSI Database Tables
CREATE TABLE IF NOT EXISTS msi_tables (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    required INTEGER DEFAULT 0,
    columns TEXT, -- JSON array of column definitions
    documentation_url TEXT
);

-- Standard Directories
CREATE TABLE IF NOT EXISTS standard_directories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    windows_path TEXT,
    example TEXT
);

-- Built-in Properties
CREATE TABLE IF NOT EXISTS builtin_properties (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    property_type TEXT, -- string, integer, path
    description TEXT,
    default_value TEXT,
    readonly INTEGER DEFAULT 0
);

-- Preprocessor Directives
CREATE TABLE IF NOT EXISTS preprocessor_directives (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    syntax TEXT,
    description TEXT,
    example TEXT
);

-- Standard MSI Actions
CREATE TABLE IF NOT EXISTS standard_actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    sequence INTEGER,
    description TEXT
);

-- Code Snippets
CREATE TABLE IF NOT EXISTS snippets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    prefix TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    body TEXT NOT NULL, -- The snippet template
    scope TEXT DEFAULT 'wxs', -- wxs, wxi, wxl
    UNIQUE (prefix, scope)
);

-- Keywords for syntax highlighting
CREATE TABLE IF NOT EXISTS keywords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    word TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL -- element, preprocessor, directory, property
);

-- Version migration mappings (v3 -> v4)
CREATE TABLE IF NOT EXISTS migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_version TEXT NOT NULL,
    to_version TEXT NOT NULL,
    change_type TEXT NOT NULL, -- renamed, removed, added, changed, namespace
    old_value TEXT,
    new_value TEXT,
    notes TEXT
);

-- Extension schemas
CREATE TABLE IF NOT EXISTS extensions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE, -- bal, util, netfx, etc.
    namespace TEXT NOT NULL,
    prefix TEXT NOT NULL,
    description TEXT,
    xsd_url TEXT
);

-- Extension elements (linked to extensions)
CREATE TABLE IF NOT EXISTS extension_elements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    extension_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    documentation_url TEXT,
    UNIQUE (extension_id, name),
    FOREIGN KEY (extension_id) REFERENCES extensions(id) ON DELETE CASCADE
);

-- Prerequisites (runtime dependencies)
CREATE TABLE IF NOT EXISTS prerequisites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT,
    version TEXT,
    download_url TEXT,
    detection_method TEXT, -- registry, file, command
    detection_value TEXT
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_elements_namespace ON elements(namespace);
CREATE INDEX IF NOT EXISTS idx_elements_name ON elements(name);
CREATE INDEX IF NOT EXISTS idx_attributes_element ON attributes(element_id);
CREATE INDEX IF NOT EXISTS idx_attributes_name ON attributes(name);
CREATE INDEX IF NOT EXISTS idx_rules_category ON rules(category);
CREATE INDEX IF NOT EXISTS idx_rules_severity ON rules(severity);
CREATE INDEX IF NOT EXISTS idx_errors_code ON errors(code);
CREATE INDEX IF NOT EXISTS idx_ice_code ON ice_rules(code);
CREATE INDEX IF NOT EXISTS idx_snippets_prefix ON snippets(prefix);
CREATE INDEX IF NOT EXISTS idx_keywords_category ON keywords(category);
