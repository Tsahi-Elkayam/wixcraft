#!/usr/bin/env python3
"""Fix documentation entries with scraping artifacts."""

import sqlite3
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"

# Clean content for documentation entries with scraping artifacts
CLEAN_DOCUMENTATION = {
    ("wilogutl", "tools"): """WiLogUtl.exe - Windows Installer Log Utility

WiLogUtl.exe is a Windows SDK tool that analyzes Windows Installer log files to identify errors, warnings, and potential issues during installation.

Usage:
  WiLogUtl.exe <logfile> [options]

Options:
  /q - Quiet mode, no UI
  /l - List errors only
  /e - Export to text file

Features:
- Parses verbose MSI log files
- Highlights errors and warnings
- Shows action timing information
- Identifies common installation failures

Example:
  WiLogUtl.exe install.log /l

The tool helps diagnose installation failures by parsing the detailed information in MSI log files created with msiexec /l*v.""",

    ("msival2", "tools"): """MsiVal2.exe - MSI Validation Tool

MsiVal2.exe validates Windows Installer packages against ICE (Internal Consistency Evaluators) rules to ensure package quality and compliance.

Usage:
  MsiVal2.exe <database> <cub_file> [-f] [-l <logfile>]

Parameters:
  database - Path to MSI or MSM file to validate
  cub_file - Path to CUB file containing ICE rules (darice.cub, mergemod.cub)
  -f - Full validation (run all ICEs)
  -l - Write results to log file

ICE Categories:
- ICE01-ICE32: Basic validation rules
- ICE33-ICE64: Component and feature rules
- ICE65-ICE99: Advanced validation

Example:
  MsiVal2.exe MyProduct.msi darice.cub -f -l validation.log

Returns error level based on validation results (0=pass, non-zero=failures).""",

    ("msimsp-tool", "tools"): """MsiMsp.exe - MSI Patch Creation Tool

MsiMsp.exe creates Windows Installer patch packages (.msp files) from patch creation properties files.

Usage:
  MsiMsp.exe -s <pcp_file> -p <msp_file> [-l <logfile>] [-f] [-d]

Parameters:
  -s - Source PCP (Patch Creation Properties) file
  -p - Output MSP patch file path
  -l - Log file path
  -f - Fail if any warnings occur
  -d - Display debug information

The PCP file defines:
- Target and upgraded product databases
- File-level patch transforms
- Patch metadata and properties
- Sequence information

Example:
  MsiMsp.exe -s MyPatch.pcp -p MyPatch.msp -l patch.log

Note: WiX v4+ uses wix msp command instead of this legacy tool.""",

    ("msidb-tool", "tools"): """MsiDb.exe - MSI Database Tool

MsiDb.exe imports, exports, and manages tables and streams in Windows Installer database files.

Usage:
  MsiDb.exe -d <database> [options]

Common Options:
  -c - Create new database
  -f <folder> - Folder for import/export files
  -i <table> - Import table from IDT file
  -e <table> - Export table to IDT file
  -m <merge_module> - Merge module into database
  -k - Kill (remove) table
  -t <transform> - Apply transform
  -x <stream> - Extract stream to file
  -a <stream> - Add stream from file

Examples:
  Export Property table:
    MsiDb.exe -d product.msi -f . -e Property

  Import modified table:
    MsiDb.exe -d product.msi -f . -i Property

  Extract embedded cabinet:
    MsiDb.exe -d product.msi -x _Cabinet -c data.cab

IDT (installer database text) format uses tab-delimited files with header rows.""",

    ("orca-tool", "tools"): """Orca.exe - MSI Database Editor

Orca is a graphical database table editor for Windows Installer packages included in the Windows SDK.

Features:
- View and edit all MSI database tables
- Validate packages against ICE rules
- Create and apply transforms (.mst files)
- Generate patches (.msp files)
- Merge modules (.msm files)

Common Uses:
1. Inspecting MSI package contents
2. Making quick edits to Property table
3. Validating packages before deployment
4. Creating admin transforms
5. Debugging installation issues

Menu Operations:
- File > Open: Load MSI/MSM/MST/PCP file
- Transform > New Transform: Start recording changes
- Transform > Generate Transform: Save recorded changes
- Tools > Validate: Run ICE validation

Keyboard Shortcuts:
- Ctrl+O: Open database
- Ctrl+V: Validate
- F5: Refresh view

Note: Orca is read-only for some internal tables. Use MsiDb.exe for programmatic access.""",

    ("msiexec-reference", "commands"): """msiexec.exe - Windows Installer Command Reference

msiexec.exe is the Windows Installer executable that installs, modifies, and removes MSI packages.

Basic Syntax:
  msiexec.exe [options] [package_path]

Install Options:
  /i <package> - Install product
  /a <package> - Administrative install
  /x <product> - Uninstall product
  /j[u|m] <package> - Advertise product
  /f[p|o|e|d|c|a|u|m|s|v] - Repair product

Display Options:
  /q[n|b|r|f] - Set UI level (n=none, b=basic, r=reduced, f=full)
  /passive - Unattended mode with progress bar

Logging Options:
  /l[i|w|e|a|r|u|c|m|o|p|v|x|+|!|*] <logfile>
  /l*v - Verbose logging (recommended for troubleshooting)

Restart Options:
  /norestart - Suppress restart prompts
  /promptrestart - Prompt before restart
  /forcerestart - Always restart after install

Properties:
  PROPERTY=value - Set public property
  TRANSFORMS=<path> - Apply transform file

Examples:
  Silent install:
    msiexec /i product.msi /qn

  Install with logging:
    msiexec /i product.msi /l*v install.log

  Uninstall by ProductCode:
    msiexec /x {GUID} /qn

  Administrative install:
    msiexec /a product.msi TARGETDIR=C:\\AdminImage"""
}


def main():
    """Fix documentation with scraping artifacts."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    fixed = 0
    for (source, topic), content in CLEAN_DOCUMENTATION.items():
        cursor.execute("""
            UPDATE documentation
            SET content = ?
            WHERE source = ? AND topic = ?
        """, (content, source, topic))
        if cursor.rowcount > 0:
            fixed += 1
            print(f"Fixed: {source}/{topic}")

    conn.commit()

    # Verify
    cursor.execute("""
        SELECT COUNT(*) FROM documentation
        WHERE content LIKE '%Table of contents%'
    """)
    remaining = cursor.fetchone()[0]

    print(f"\nFixed {fixed} documentation entries")
    print(f"Entries with scraping artifacts remaining: {remaining}")

    conn.close()


if __name__ == "__main__":
    main()
