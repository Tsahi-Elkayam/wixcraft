//! Tests for reference data (directories, properties, keywords, snippets, migrations)

use wixkb::db::Database;

#[test]
fn test_standard_directory_insert_and_get() {
    let db = Database::open_memory().unwrap();

    db.conn().execute(
        "INSERT INTO standard_directories (name, description, windows_path, example)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            "ProgramFilesFolder",
            "32-bit Program Files",
            "C:\\Program Files (x86)",
            "<StandardDirectory Id=\"ProgramFilesFolder\" />"
        ],
    ).unwrap();

    let dir = db.get_standard_directory("ProgramFilesFolder").unwrap().unwrap();
    assert_eq!(dir.name, "ProgramFilesFolder");
    assert_eq!(dir.description, Some("32-bit Program Files".to_string()));
    assert_eq!(dir.windows_path, Some("C:\\Program Files (x86)".to_string()));
}

#[test]
fn test_standard_directory_not_found() {
    let db = Database::open_memory().unwrap();
    let result = db.get_standard_directory("NonExistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_all_standard_directories() {
    let db = Database::open_memory().unwrap();

    let dirs = vec![
        "ProgramFilesFolder",
        "ProgramFiles64Folder",
        "SystemFolder",
        "WindowsFolder",
    ];

    for name in dirs {
        db.conn().execute(
            "INSERT INTO standard_directories (name) VALUES (?1)",
            rusqlite::params![name],
        ).unwrap();
    }

    let all_dirs = db.get_all_standard_directories().unwrap();
    assert_eq!(all_dirs.len(), 4);
}

#[test]
fn test_builtin_property_insert_and_get() {
    let db = Database::open_memory().unwrap();

    db.conn().execute(
        "INSERT INTO builtin_properties (name, property_type, description, default_value, readonly)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            "ProductName",
            "string",
            "The name of the product",
            "",
            0
        ],
    ).unwrap();

    let prop = db.get_builtin_property("ProductName").unwrap().unwrap();
    assert_eq!(prop.name, "ProductName");
    assert_eq!(prop.property_type, Some("string".to_string()));
    assert!(!prop.readonly);
}

#[test]
fn test_builtin_property_not_found() {
    let db = Database::open_memory().unwrap();
    let result = db.get_builtin_property("NonExistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_snippet_insert_and_get_by_prefix() {
    let db = Database::open_memory().unwrap();

    let snippets = vec![
        ("wix-pkg", "Package", "<Package />"),
        ("wix-comp", "Component", "<Component />"),
        ("wix-file", "File", "<File />"),
    ];

    for (prefix, name, body) in snippets {
        db.conn().execute(
            "INSERT INTO snippets (prefix, name, description, body, scope)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![prefix, name, format!("Creates {}", name), body, "wxs"],
        ).unwrap();
    }

    let results = db.get_snippets("wix-c").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].prefix, "wix-comp");

    let results = db.get_snippets("wix").unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn test_get_all_snippets() {
    let db = Database::open_memory().unwrap();

    for i in 0..5 {
        db.conn().execute(
            "INSERT INTO snippets (prefix, name, body, scope) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![format!("snip{}", i), format!("Snippet {}", i), "body", "wxs"],
        ).unwrap();
    }

    let all = db.get_all_snippets().unwrap();
    assert_eq!(all.len(), 5);
}

#[test]
fn test_keywords_by_category() {
    let db = Database::open_memory().unwrap();

    let keywords = vec![
        ("Package", "element"),
        ("Component", "element"),
        ("File", "element"),
        ("define", "preprocessor"),
        ("if", "preprocessor"),
        ("endif", "preprocessor"),
        ("ProgramFilesFolder", "directory"),
    ];

    for (word, category) in keywords {
        db.conn().execute(
            "INSERT INTO keywords (word, category) VALUES (?1, ?2)",
            rusqlite::params![word, category],
        ).unwrap();
    }

    let elements = db.get_keywords("element").unwrap();
    assert_eq!(elements.len(), 3);
    assert!(elements.contains(&"Package".to_string()));

    let preprocessor = db.get_keywords("preprocessor").unwrap();
    assert_eq!(preprocessor.len(), 3);
    assert!(preprocessor.contains(&"define".to_string()));

    let directories = db.get_keywords("directory").unwrap();
    assert_eq!(directories.len(), 1);
}

#[test]
fn test_migrations() {
    let db = Database::open_memory().unwrap();

    let migrations = vec![
        ("renamed", "Product", "Package", "Root element renamed"),
        ("removed", "TARGETDIR", "", "Use StandardDirectory"),
        ("added", "", "Files", "Wildcard harvesting"),
    ];

    for (change_type, old, new, notes) in migrations {
        db.conn().execute(
            "INSERT INTO migrations (from_version, to_version, change_type, old_value, new_value, notes)
             VALUES ('v3', 'v4', ?1, ?2, ?3, ?4)",
            rusqlite::params![change_type, old, new, notes],
        ).unwrap();
    }

    let results = db.get_migrations("v3", "v4").unwrap();
    assert_eq!(results.len(), 3);

    let renamed = results.iter().find(|m| m.change_type == "renamed").unwrap();
    assert_eq!(renamed.old_value, Some("Product".to_string()));
    assert_eq!(renamed.new_value, Some("Package".to_string()));
}

#[test]
fn test_migrations_different_versions() {
    let db = Database::open_memory().unwrap();

    db.conn().execute(
        "INSERT INTO migrations (from_version, to_version, change_type, notes)
         VALUES ('v3', 'v4', 'renamed', 'v3 to v4')",
        [],
    ).unwrap();

    db.conn().execute(
        "INSERT INTO migrations (from_version, to_version, change_type, notes)
         VALUES ('v4', 'v5', 'added', 'v4 to v5')",
        [],
    ).unwrap();

    let v3_v4 = db.get_migrations("v3", "v4").unwrap();
    assert_eq!(v3_v4.len(), 1);

    let v4_v5 = db.get_migrations("v4", "v5").unwrap();
    assert_eq!(v4_v5.len(), 1);

    let v3_v5 = db.get_migrations("v3", "v5").unwrap();
    assert_eq!(v3_v5.len(), 0);
}
