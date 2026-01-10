#!/usr/bin/env python3
"""Fix remaining minor gaps in the database."""

import sqlite3
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"


def main():
    """Fix minor gaps."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    # 1. Add parent relationships for elements that need them
    # These are *Ref elements that reference other elements
    # Most *SearchRef elements are children of search containers

    # Get element IDs
    cursor.execute("SELECT id, name FROM elements")
    element_ids = {row[1]: row[0] for row in cursor.fetchall()}

    # Define parent relationships for orphaned elements
    parent_mappings = {
        # Search reference elements - children of AppSearch, CCPSearch etc.
        "ComponentSearchRef": ["ComponentSearch"],  # References ComponentSearch
        "DirectorySearchRef": ["DirectorySearch"],
        "FileSearchRef": ["FileSearch"],
        "ProductSearchRef": ["ProductSearch"],
        "RegistrySearchRef": ["RegistrySearch"],
        "WindowsFeatureSearchRef": ["WindowsFeatureSearch"],

        # DotNet search references - children of Bundle
        "DotNetCompatibilityCheckRef": ["Chain", "Bundle"],
        "DotNetCoreSdkSearchRef": ["Bundle"],
        "DotNetCoreSdkFeatureBandSearchRef": ["Bundle"],
        "DotNetCoreSearchRef": ["Bundle"],

        # Query elements - children of Bundle
        "QueryNativeMachine": ["Bundle"],
        "QueryWindowsDirectories": ["Bundle"],
        "QueryWindowsDriverInfo": ["Bundle"],
        "QueryWindowsSuiteInfo": ["Bundle"],
        "QueryWindowsWellKnownSIDs": ["Bundle"],

        # Other elements
        "Include": ["Wix"],  # Include can be child of Wix (preprocessor)
        "RequiredPrivilege": ["Service", "ServiceInstall"],

        # Wix is the root element - no parent needed
    }

    parents_added = 0
    for child_name, parent_names in parent_mappings.items():
        if child_name not in element_ids:
            continue
        child_id = element_ids[child_name]

        for parent_name in parent_names:
            if parent_name not in element_ids:
                continue
            parent_id = element_ids[parent_name]

            # Check if relationship already exists
            cursor.execute("""
                SELECT 1 FROM element_parents
                WHERE element_id = ? AND parent_id = ?
            """, (child_id, parent_id))

            if not cursor.fetchone():
                cursor.execute("""
                    INSERT INTO element_parents (element_id, parent_id)
                    VALUES (?, ?)
                """, (child_id, parent_id))
                parents_added += 1

    print(f"Added {parents_added} parent relationships")

    # 2. Add syntax to msiexec restart options
    cli_syntax = {
        ("/norestart", "msiexec_restart"): "msiexec.exe /norestart /i package.msi",
        ("/promptrestart", "msiexec_restart"): "msiexec.exe /promptrestart /i package.msi",
        ("/forcerestart", "msiexec_restart"): "msiexec.exe /forcerestart /i package.msi",
    }

    for (name, category), syntax in cli_syntax.items():
        cursor.execute("""
            UPDATE cli_commands
            SET syntax = ?
            WHERE name = ? AND category = ?
        """, (syntax, name, category))

    print(f"Updated 3 CLI command syntax entries")

    # 3. Add message_template to the missing error
    cursor.execute("""
        UPDATE errors
        SET message_template = 'The bind path variable ''{{0}}'' was specified more than once on the command line. Only one value per variable is allowed.'
        WHERE code = 'DuplicateBindPathVariableOnCommandLine'
    """)
    print(f"Updated 1 error message_template")

    conn.commit()

    # Verify remaining orphans (should only be root elements like Wix)
    cursor.execute("""
        SELECT name FROM elements
        WHERE id NOT IN (SELECT element_id FROM element_parents)
        ORDER BY name
    """)
    orphans = [row[0] for row in cursor.fetchall()]
    print(f"Remaining elements without parents: {len(orphans)}")
    if orphans:
        for o in orphans:
            print(f"  - {o}")

    conn.close()


if __name__ == "__main__":
    main()
