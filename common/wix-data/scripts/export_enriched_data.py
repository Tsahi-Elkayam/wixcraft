#!/usr/bin/env python3
"""Export enriched data from database to JSON files for harvest.

This script exports curated/enriched data that was added to the database
back to JSON files so that harvest can repopulate everything correctly.
"""

import sqlite3
import json
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"
DATA_PATH = Path(__file__).parent.parent / "config" / "data"


def export_rule_enrichments():
    """Export rule rationales and fix suggestions."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    cursor.execute("""
        SELECT rule_id, rationale, fix_suggestion, auto_fixable
        FROM rules
        WHERE rationale IS NOT NULL AND rationale != ''
        ORDER BY rule_id
    """)

    enrichments = []
    for row in cursor.fetchall():
        enrichments.append({
            "rule_id": row["rule_id"],
            "rationale": row["rationale"],
            "fix_suggestion": row["fix_suggestion"],
            "auto_fixable": bool(row["auto_fixable"])
        })

    conn.close()

    output = {
        "_source": "Rule enrichments - rationale, fix suggestions, auto-fix flags",
        "_description": "Curated content explaining why rules matter and how to fix violations",
        "rule_enrichments": enrichments
    }

    path = DATA_PATH / "rule-enrichments.json"
    with open(path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"Exported {len(enrichments)} rule enrichments to {path.name}")


def export_rule_conditions():
    """Export all rule conditions."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    cursor.execute("""
        SELECT r.rule_id, rc.condition_type, rc.target, rc.operator, rc.value
        FROM rule_conditions rc
        JOIN rules r ON rc.rule_id = r.id
        ORDER BY r.rule_id, rc.id
    """)

    # Group by rule_id
    rules = {}
    for row in cursor.fetchall():
        rule_id = row["rule_id"]
        if rule_id not in rules:
            rules[rule_id] = []
        rules[rule_id].append({
            "condition_type": row["condition_type"],
            "target": row["target"],
            "operator": row["operator"],
            "value": row["value"]
        })

    conn.close()

    conditions = [
        {"rule_id": rule_id, "conditions": conds}
        for rule_id, conds in sorted(rules.items())
    ]

    output = {
        "_source": "Rule evaluation conditions for WiX linter",
        "_description": "Each rule has one or more conditions that define HOW the rule is evaluated",
        "rule_conditions": conditions
    }

    path = DATA_PATH / "rule-conditions.json"
    with open(path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"Exported conditions for {len(conditions)} rules to {path.name}")


def export_element_enrichments():
    """Export element examples and remarks."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    cursor.execute("""
        SELECT name, example, remarks
        FROM elements
        WHERE (example IS NOT NULL AND example != '')
           OR (remarks IS NOT NULL AND remarks != '')
        ORDER BY name
    """)

    enrichments = []
    for row in cursor.fetchall():
        entry = {"name": row["name"]}
        if row["example"]:
            entry["example"] = row["example"]
        if row["remarks"]:
            entry["remarks"] = row["remarks"]
        enrichments.append(entry)

    conn.close()

    output = {
        "_source": "Element enrichments - examples and remarks",
        "_description": "Curated WiX XML examples and usage notes for elements",
        "element_enrichments": enrichments
    }

    path = DATA_PATH / "element-enrichments.json"
    with open(path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"Exported {len(enrichments)} element enrichments to {path.name}")


def export_enum_descriptions():
    """Export attribute enum value descriptions."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    cursor.execute("""
        SELECT e.name as element, a.name as attribute, ev.value, ev.description
        FROM attribute_enum_values ev
        JOIN attributes a ON ev.attribute_id = a.id
        JOIN elements e ON a.element_id = e.id
        WHERE ev.description IS NOT NULL AND ev.description != ''
        ORDER BY e.name, a.name, ev.value
    """)

    descriptions = []
    for row in cursor.fetchall():
        descriptions.append({
            "element": row["element"],
            "attribute": row["attribute"],
            "value": row["value"],
            "description": row["description"]
        })

    conn.close()

    output = {
        "_source": "Attribute enum value descriptions",
        "_description": "Descriptions for enum attribute values",
        "enum_descriptions": descriptions
    }

    path = DATA_PATH / "attribute-enum-descriptions.json"
    with open(path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"Exported {len(descriptions)} enum descriptions to {path.name}")


def export_msi_table_columns():
    """Export MSI table column definitions."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    cursor.execute("""
        SELECT name, description, required, columns, documentation_url
        FROM msi_tables
        WHERE columns IS NOT NULL AND length(columns) > 2
        ORDER BY name
    """)

    tables = []
    for row in cursor.fetchall():
        tables.append({
            "name": row["name"],
            "description": row["description"],
            "required": bool(row["required"]),
            "columns": json.loads(row["columns"]) if row["columns"] else [],
            "documentation_url": row["documentation_url"]
        })

    conn.close()

    output = {
        "_source": "MSI database table definitions",
        "_description": "Complete column definitions for Windows Installer database tables",
        "msi_tables": tables
    }

    path = DATA_PATH / "msi-table-definitions.json"
    with open(path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"Exported {len(tables)} MSI table definitions to {path.name}")


def export_element_parents():
    """Export element parent relationships."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    cursor.execute("""
        SELECT e.name as element, p.name as parent
        FROM element_parents ep
        JOIN elements e ON ep.element_id = e.id
        JOIN elements p ON ep.parent_id = p.id
        ORDER BY e.name, p.name
    """)

    # Group by element
    relationships = {}
    for row in cursor.fetchall():
        element = row["element"]
        if element not in relationships:
            relationships[element] = []
        relationships[element].append(row["parent"])

    conn.close()

    parents = [
        {"element": element, "parents": sorted(parents)}
        for element, parents in sorted(relationships.items())
    ]

    output = {
        "_source": "Element parent relationships",
        "_description": "Valid parent elements for each WiX element",
        "element_parents": parents
    }

    path = DATA_PATH / "element-parents.json"
    with open(path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"Exported parent relationships for {len(parents)} elements to {path.name}")


def export_sources():
    """Export data sources."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    cursor.execute("""
        SELECT name, url, source_type, enabled
        FROM sources
        ORDER BY source_type, name
    """)

    sources = []
    for row in cursor.fetchall():
        sources.append({
            "name": row["name"],
            "url": row["url"],
            "source_type": row["source_type"],
            "enabled": bool(row["enabled"])
        })

    conn.close()

    output = {
        "_source": "Data provenance tracking",
        "_description": "Sources used to build the WiX knowledge base",
        "sources": sources
    }

    path = DATA_PATH / "data-sources.json"
    with open(path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"Exported {len(sources)} data sources to {path.name}")


def export_standard_directory_enrichments():
    """Export standard directory examples."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    cursor.execute("""
        SELECT id, name, description, windows_path, example
        FROM standard_directories
        WHERE example IS NOT NULL AND example != ''
        ORDER BY name
    """)

    directories = []
    for row in cursor.fetchall():
        directories.append({
            "name": row["name"],
            "description": row["description"],
            "windows_path": row["windows_path"],
            "example": row["example"]
        })

    conn.close()

    output = {
        "_source": "Standard directory definitions with examples",
        "_description": "Windows Installer standard directories with WiX usage examples",
        "standard_directories": directories
    }

    path = DATA_PATH / "standard-directories-enriched.json"
    with open(path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"Exported {len(directories)} standard directories to {path.name}")


def export_migration_notes():
    """Export migration notes."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    cursor.execute("""
        SELECT from_version, to_version, change_type, old_value, new_value, notes
        FROM migrations
        WHERE notes IS NOT NULL AND notes != ''
        ORDER BY from_version, to_version, change_type
    """)

    migrations = []
    for row in cursor.fetchall():
        migrations.append({
            "from_version": row["from_version"],
            "to_version": row["to_version"],
            "change_type": row["change_type"],
            "old_value": row["old_value"],
            "new_value": row["new_value"],
            "notes": row["notes"]
        })

    conn.close()

    output = {
        "_source": "WiX version migration notes",
        "_description": "Migration guidance for upgrading between WiX versions",
        "migrations": migrations
    }

    path = DATA_PATH / "migration-notes.json"
    with open(path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"Exported {len(migrations)} migration notes to {path.name}")


def export_cli_enrichments():
    """Export CLI command syntax."""
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()

    cursor.execute("""
        SELECT name, description, syntax, category
        FROM cli_commands
        WHERE syntax IS NOT NULL AND syntax != ''
        ORDER BY category, name
    """)

    commands = []
    for row in cursor.fetchall():
        commands.append({
            "name": row["name"],
            "description": row["description"],
            "syntax": row["syntax"],
            "category": row["category"]
        })

    conn.close()

    output = {
        "_source": "CLI command reference",
        "_description": "Command syntax for WiX and msiexec commands",
        "cli_commands": commands
    }

    path = DATA_PATH / "cli-commands-enriched.json"
    with open(path, "w") as f:
        json.dump(output, f, indent=2)

    print(f"Exported {len(commands)} CLI commands to {path.name}")


def main():
    """Export all enriched data."""
    print("Exporting enriched data from database to JSON files...\n")

    export_rule_enrichments()
    export_rule_conditions()
    export_element_enrichments()
    export_enum_descriptions()
    export_msi_table_columns()
    export_element_parents()
    export_sources()
    export_standard_directory_enrichments()
    export_migration_notes()
    export_cli_enrichments()

    print(f"\nAll exports complete. Files written to {DATA_PATH}")


if __name__ == "__main__":
    main()
