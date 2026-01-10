#!/usr/bin/env python3
"""Populate the sources table with data provenance information."""

import sqlite3
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"

# Data sources used to build the database
SOURCES = [
    # WiX Schema files
    ("wix.xsd", "https://wixtoolset.org/schemas/v4/wxs/wix.xsd", "xsd"),
    ("bal.xsd", "https://wixtoolset.org/schemas/v4/wxs/bal.xsd", "xsd"),
    ("complus.xsd", "https://wixtoolset.org/schemas/v4/wxs/complus.xsd", "xsd"),
    ("dependency.xsd", "https://wixtoolset.org/schemas/v4/wxs/dependency.xsd", "xsd"),
    ("difxapp.xsd", "https://wixtoolset.org/schemas/v4/wxs/difxapp.xsd", "xsd"),
    ("directx.xsd", "https://wixtoolset.org/schemas/v4/wxs/directx.xsd", "xsd"),
    ("firewall.xsd", "https://wixtoolset.org/schemas/v4/wxs/firewall.xsd", "xsd"),
    ("http.xsd", "https://wixtoolset.org/schemas/v4/wxs/http.xsd", "xsd"),
    ("iis.xsd", "https://wixtoolset.org/schemas/v4/wxs/iis.xsd", "xsd"),
    ("msmq.xsd", "https://wixtoolset.org/schemas/v4/wxs/msmq.xsd", "xsd"),
    ("netfx.xsd", "https://wixtoolset.org/schemas/v4/wxs/netfx.xsd", "xsd"),
    ("powershell.xsd", "https://wixtoolset.org/schemas/v4/wxs/powershell.xsd", "xsd"),
    ("sql.xsd", "https://wixtoolset.org/schemas/v4/wxs/sql.xsd", "xsd"),
    ("ui.xsd", "https://wixtoolset.org/schemas/v4/wxs/ui.xsd", "xsd"),
    ("util.xsd", "https://wixtoolset.org/schemas/v4/wxs/util.xsd", "xsd"),
    ("vs.xsd", "https://wixtoolset.org/schemas/v4/wxs/vs.xsd", "xsd"),

    # WiX Documentation
    ("wix-docs-main", "https://wixtoolset.org/docs/", "documentation"),
    ("wix-docs-schema", "https://wixtoolset.org/docs/schema/", "documentation"),
    ("wix-docs-tools", "https://wixtoolset.org/docs/tools/", "documentation"),
    ("wix-docs-reference", "https://wixtoolset.org/docs/reference/", "documentation"),

    # Microsoft MSI Documentation
    ("msi-database-reference", "https://learn.microsoft.com/en-us/windows/win32/msi/database-tables", "msi"),
    ("msi-property-reference", "https://learn.microsoft.com/en-us/windows/win32/msi/property-reference", "msi"),
    ("msi-standard-actions", "https://learn.microsoft.com/en-us/windows/win32/msi/standard-actions-reference", "msi"),
    ("msi-error-reference", "https://learn.microsoft.com/en-us/windows/win32/msi/error-codes", "msi"),
    ("msi-custom-actions", "https://learn.microsoft.com/en-us/windows/win32/msi/custom-actions", "msi"),
    ("msi-conditions", "https://learn.microsoft.com/en-us/windows/win32/msi/conditional-statement-syntax", "msi"),

    # ICE Rules Documentation
    ("ice-reference", "https://learn.microsoft.com/en-us/windows/win32/msi/ice-reference", "ice"),
    ("ice-01-32", "https://learn.microsoft.com/en-us/windows/win32/msi/ice01", "ice"),
    ("ice-33-64", "https://learn.microsoft.com/en-us/windows/win32/msi/ice33", "ice"),
    ("ice-65-99", "https://learn.microsoft.com/en-us/windows/win32/msi/ice65", "ice"),

    # FireGiant Documentation
    ("firegiant-tutorials", "https://www.firegiant.com/docs/wix/tutorial/", "documentation"),
    ("firegiant-howtos", "https://www.firegiant.com/docs/wix/howtos/", "documentation"),

    # WiX GitHub Repository
    ("wix-github-repo", "https://github.com/wixtoolset/wix", "documentation"),
    ("wix-issues", "https://github.com/wixtoolset/issues/issues", "documentation"),

    # Windows SDK Tools
    ("sdk-tools", "https://learn.microsoft.com/en-us/windows/win32/msi/windows-installer-development-tools", "msi"),
    ("msiexec-reference", "https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/msiexec", "msi"),

    # Burn/Bundle Documentation
    ("burn-reference", "https://wixtoolset.org/docs/tools/burn/", "documentation"),
    ("bundle-schema", "https://wixtoolset.org/docs/schema/wxs/bundle/", "documentation"),

    # Extension Documentation
    ("bal-extension", "https://wixtoolset.org/docs/schema/bal/", "documentation"),
    ("util-extension", "https://wixtoolset.org/docs/schema/util/", "documentation"),
    ("iis-extension", "https://wixtoolset.org/docs/schema/iis/", "documentation"),
    ("sql-extension", "https://wixtoolset.org/docs/schema/sql/", "documentation"),
    ("netfx-extension", "https://wixtoolset.org/docs/schema/netfx/", "documentation"),
    ("firewall-extension", "https://wixtoolset.org/docs/schema/firewall/", "documentation"),

    # v3 to v4/v5 Migration
    ("v3-v4-migration", "https://wixtoolset.org/docs/fourthree/", "documentation"),
    ("v4-v5-conversion", "https://wixtoolset.org/docs/fivefour/", "documentation"),
]


def main():
    """Populate sources table."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    added = 0
    for name, url, source_type in SOURCES:
        try:
            cursor.execute("""
                INSERT INTO sources (name, url, source_type, enabled)
                VALUES (?, ?, ?, 1)
            """, (name, url, source_type))
            added += 1
        except sqlite3.IntegrityError:
            # Already exists
            pass

    conn.commit()

    # Summary by type
    cursor.execute("""
        SELECT source_type, COUNT(*) FROM sources GROUP BY source_type ORDER BY source_type
    """)
    print("Sources by type:")
    for row in cursor.fetchall():
        print(f"  {row[0]}: {row[1]}")

    cursor.execute("SELECT COUNT(*) FROM sources")
    total = cursor.fetchone()[0]
    print(f"\nTotal sources: {total}")
    print(f"Added: {added}")

    conn.close()


if __name__ == "__main__":
    main()
