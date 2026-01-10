# WixCraft Tools List

Tools mapped to gaps and pain points from research.

---

## Gap to Tool Mapping

### MSI Pain Points → Tools

| Pain Point | Tool | Priority |
|------------|------|----------|
| Steep Learning Curve | Help Browser, Snippets, Tutorials | High |
| Cryptic Error Messages | Error Translator | High |
| Debugging Nightmares | Log Analyzer | High |
| Custom Action Debugging | CA Debugger Helper | Medium |
| Component Rules Complexity | Component Validator | High |
| MSI Security Vulnerabilities | Security Scanner | Medium |
| Patching/Upgrade Complexity | Upgrade Validator | Medium |
| MSI Table Structure Confusion | MSI Inspector | High |

### WiX Gaps → Tools

| Gap | Tool | Priority |
|-----|------|----------|
| No VS Code Extension | VS Code Extension + LSP | Critical |
| Poor Documentation | Help Browser, Hover Docs | High |
| No GUI/WYSIWYG | Code Generator/Wizard | Low |
| v3→v4→v5→v6 Migration | Migration Tool | High |
| Preprocessor Variable Errors | Linter (preprocessor rules) | High |
| Heat Problems | Modern Harvester | Medium |
| Burn Bootstrapper Issues | Bundle Analyzer | Low |
| Localization Complexity | Localization Helper | Low |
| CI/CD Issues | GitHub Action, Azure Task | Medium |

### Competitor Features → Tools

| Competitor Feature | Our Tool |
|--------------------|----------|
| Built-in ICE Fix Suggestions | Linter with auto-fix |
| One-click MSI Analysis | MSI Inspector |
| Real-time Validation | LSP Diagnostics |
| Silent Install Discovery | Property Discoverer |
| Log Analysis | Log Analyzer |
| Security Scanning | Security Scanner |

---

## Complete Tools List

### Tier 1: Core Tools (Engine + Plugin)

These provide the foundation for everything else.

| # | Tool | Type | Description |
|---|------|------|-------------|
| 1 | **Engine** | Library | Generic parsing, rules, formatting engine |
| 2 | **WiX Plugin** | Data | YAML definitions for WiX elements, rules, snippets |
| 3 | **LSP Server** | Service | Language Server Protocol implementation |

### Tier 2: Editor Integration

| # | Tool | Type | Description |
|---|------|------|-------------|
| 4 | **VS Code Extension** | Extension | Full WiX support for VS Code |
| 5 | **Sublime Text Package** | Extension | Full WiX support for Sublime |
| 6 | **TextMate Grammar** | Data | Syntax highlighting (shared by editors) |

### Tier 3: CLI Tools

| # | Tool | Type | Description |
|---|------|------|-------------|
| 7 | **Linter** | CLI | Static analysis, rule violations, best practices |
| 8 | **Formatter** | CLI | Code formatting, consistent style |
| 9 | **Help Browser** | CLI | `wix-help Component` - instant documentation |
| 10 | **Error Translator** | CLI | Human-readable error explanations |
| 11 | **Migration Tool** | CLI | Convert v3→v4→v5→v6 with warnings |
| 12 | **Code Generator** | CLI | Generate WiX from wizard/prompts |
| 13 | **Harvester** | CLI | Modern Heat replacement |

### Tier 4: Analysis Tools

| # | Tool | Type | Description |
|---|------|------|-------------|
| 14 | **MSI Inspector** | CLI/TUI | Explore MSI tables, properties, files |
| 15 | **Log Analyzer** | CLI | Parse MSI logs, find actual errors |
| 16 | **Security Scanner** | CLI | Detect privilege escalation vulnerabilities |
| 17 | **Upgrade Validator** | CLI | Check component/upgrade rules |
| 18 | **Component Validator** | CLI | GUID, KeyPath, reference validation |
| 19 | **Bundle Analyzer** | CLI | Visualize Burn bootstrapper chains |
| 20 | **Property Discoverer** | CLI | Extract silent install parameters from MSI |

### Tier 5: CI/CD Integration

| # | Tool | Type | Description |
|---|------|------|-------------|
| 21 | **GitHub Action** | Action | Lint, validate, build WiX in CI |
| 22 | **Azure DevOps Task** | Task | Same for Azure Pipelines |
| 23 | **Pre-commit Hook** | Script | Run linter before commit |

---

## Tool Details

### 1. Engine (Generic)

**Purpose:** Core parsing and analysis logic, language-agnostic.

**Capabilities:**
- XML/YAML/JSON parsing
- AST traversal
- Rule evaluation
- Symbol indexing
- Formatting engine
- Completion engine
- Reference resolution

**Input:** File + Plugin
**Output:** Diagnostics, formatted code, completions, hover info

---

### 2. WiX Plugin (YAML Data)

**Purpose:** All WiX-specific knowledge in human-editable YAML.

**Contents:**
```
wix-plugin/
├── plugin.yaml           # Metadata
├── elements/             # 100+ element definitions
├── rules/                # 50+ lint rules
├── snippets/             # 30+ code templates
├── formatting/           # Formatting preferences
├── msi/                  # MSI properties, tables, actions
└── errors/               # Error code explanations
```

---

### 3. LSP Server

**Purpose:** Bridge between engine and any LSP-compatible editor.

**Capabilities:**
- `textDocument/completion` - Autocomplete
- `textDocument/hover` - Documentation on hover
- `textDocument/definition` - Go to definition
- `textDocument/references` - Find all references
- `textDocument/formatting` - Format document
- `textDocument/diagnostic` - Real-time errors
- `textDocument/codeAction` - Quick fixes
- `textDocument/rename` - Safe rename

**Solves:**
- No VS Code Extension gap
- Real-time Validation gap
- Poor Documentation gap (hover)

---

### 4. VS Code Extension

**Purpose:** First-class WiX support in VS Code.

**Features:**
- Syntax highlighting
- IntelliSense (autocomplete)
- Error squiggles (diagnostics)
- Hover documentation
- Go to Definition
- Find References
- Formatting
- Snippets
- Quick fixes

**Solves:**
- #1 WiX Gap: "No VS Code Extension"

---

### 5. Sublime Text Package

**Purpose:** Same as VS Code, for Sublime users.

**Features:** Same as VS Code (via LSP)

---

### 6. TextMate Grammar

**Purpose:** Syntax highlighting definitions.

**Used by:** VS Code, Sublime, TextMate, GitHub, etc.

**Defines:**
- Keywords
- Elements
- Attributes
- Values
- Comments
- Preprocessor directives
- Strings

---

### 7. Linter

**Purpose:** Static analysis and rule checking.

**Usage:**
```bash
wix-lint myproject.wxs
wix-lint --fix myproject.wxs
wix-lint --format sarif myproject.wxs
```

**Rule Categories:**
- Component rules (GUID, KeyPath, one-file-per-component)
- Directory rules (structure, references)
- Feature rules (hierarchy, defaults)
- Property rules (naming, public/private)
- CustomAction rules (sequencing, security)
- Service rules (install, control)
- Registry rules (paths, values)
- File rules (sources, attributes)
- Bundle rules (Burn-specific)
- Best practices

**Solves:**
- Component Rules Complexity
- Preprocessor Variable Errors
- Real-time Validation (via LSP)

---

### 8. Formatter

**Purpose:** Consistent code style.

**Usage:**
```bash
wix-fmt myproject.wxs
wix-fmt --check myproject.wxs
wix-fmt --write myproject.wxs
```

**Options:**
- Indent style (spaces/tabs)
- Indent size
- Attribute sorting
- Attribute per line threshold
- Element ordering
- Blank line rules

---

### 9. Help Browser

**Purpose:** Instant documentation from CLI.

**Usage:**
```bash
wix-help Component
wix-help Component.Guid
wix-help ICE08
wix-help error WIX0242
wix-help property ALLUSERS
wix-help directory ProgramFilesFolder
```

**Output:**
```
COMPONENT
=========

A component is the atomic unit of installation.

Attributes:
  Id       (required)  Unique identifier
  Guid     (optional)  Component GUID, use "*" for auto
  ...

Parents: Directory, DirectoryRef, ComponentGroup, Fragment
Children: File, RegistryKey, RegistryValue, Shortcut, ...

Example:
  <Component Id="MyApp_exe" Guid="*">
    <File Source="MyApp.exe" KeyPath="yes" />
  </Component>

Best Practices:
  • One file per component
  • Always set KeyPath on one child
  • Use Guid="*" for auto-generation

Documentation: https://wixtoolset.org/docs/schema/wxs/component/
```

**Solves:**
- Steep Learning Curve
- Poor/Fragmented Documentation

---

### 10. Error Translator

**Purpose:** Human-readable error explanations.

**Usage:**
```bash
wix-error WIX0242
wix-error ICE08
wix-error 1603
wix-error LGHT0217
```

**Output:**
```
ICE08: Duplicate Component GUIDs
================================

Error: Two components have the same GUID.

Why it matters:
  Windows Installer uses GUIDs to track components.
  Duplicate GUIDs cause unpredictable behavior during
  install, repair, and uninstall.

How to fix:
  1. Find both components with the same GUID
  2. Generate a new GUID for one of them
  3. If using Guid="*", check for duplicate Ids

Common causes:
  • Copy-pasted component without changing GUID
  • Merge module conflicts
  • Incorrectly shared component across features

See also: wix-help Component.Guid
```

**Solves:**
- Cryptic Error Messages
- ICE validation confusion

---

### 11. Migration Tool

**Purpose:** Convert WiX projects between versions.

**Usage:**
```bash
wix-migrate --from v3 --to v5 myproject.wxs
wix-migrate --from v3 --to v5 --dry-run myproject/
wix-migrate --report myproject/
```

**Handles:**
- Namespace changes
- Element renames
- Attribute changes
- Deprecated elements
- Breaking changes

**Output:**
```
Migration Report: myproject/
============================

Files to modify: 5
Breaking changes: 2
Warnings: 7

BREAKING:
  Product.wxs:15 - <Product> element removed in v5, use <Package>
  Product.wxs:23 - ServiceConfig requires new attributes

WARNINGS:
  Files.wxs:42 - <Directory> should use <StandardDirectory>
  ...

Run with --write to apply changes.
```

**Solves:**
- v3→v4→v5→v6 Migration Pain

---

### 12. Code Generator

**Purpose:** Generate WiX code from prompts/wizard.

**Usage:**
```bash
wix-new project
wix-new component
wix-new service
wix-new bundle
```

**Interactive:**
```
$ wix-new project

Project name: MyApp
Manufacturer: My Company
Version: 1.0.0
Install folder: [ProgramFilesFolder]\MyCompany\MyApp

Features:
  [x] Add default directory structure
  [x] Add sample file component
  [ ] Add registry settings
  [ ] Add Windows service
  [ ] Add desktop shortcut

Generated: MyApp.wixproj, Package.wxs, Directories.wxs
```

**Solves:**
- Steep Learning Curve (partially)
- No GUI/WYSIWYG (partially)

---

### 13. Harvester

**Purpose:** Modern Heat replacement.

**Usage:**
```bash
wix-harvest ./bin/Release -o Files.wxs
wix-harvest ./bin/Release --component-group MyFiles
wix-harvest ./bin/Release --stable-guids
```

**Improvements over Heat:**
- Stable GUIDs based on file path (not random)
- Incremental updates (don't regenerate unchanged)
- Better .NET assembly handling
- Exclude patterns
- Transform hooks

**Solves:**
- Heat (Harvesting) Problems

---

### 14. MSI Inspector

**Purpose:** Explore compiled MSI files.

**Usage:**
```bash
wix-msi info myapp.msi
wix-msi tables myapp.msi
wix-msi table myapp.msi Property
wix-msi files myapp.msi
wix-msi extract myapp.msi --output ./extracted/
wix-msi diff v1.msi v2.msi
wix-msi validate myapp.msi
```

**Features:**
- View all tables
- Query specific tables
- Extract files
- Diff two MSIs
- Run ICE validation
- Export to JSON/CSV

**Solves:**
- MSI Table Structure Confusion
- One-click MSI Analysis gap
- Better than Orca

---

### 15. Log Analyzer

**Purpose:** Parse MSI verbose logs, find real errors.

**Usage:**
```bash
wix-log analyze install.log
wix-log errors install.log
wix-log actions install.log
wix-log summary install.log
```

**Output:**
```
Log Analysis: install.log
=========================

Result: FAILED (error 1603)

Timeline:
  00:00:01  Installation started
  00:00:05  CostFinalize completed
  00:00:12  InstallFiles started
  00:00:45  CustomAction "CA_Install" started
  00:00:47  ERROR: Return value 3 (fatal error)
  00:00:48  Rollback started
  00:01:23  Installation failed

Root Cause:
  CustomAction "CA_Install" failed at line 47
  Action type: Type 1 (DLL in Binary table)
  DLL: CustomActions.dll

  Suggested: Check CA_Install implementation
  See: wix-help CustomAction
```

**Solves:**
- Debugging Nightmares
- "Search for return value 3" problem

---

### 16. Security Scanner

**Purpose:** Detect privilege escalation vulnerabilities.

**Usage:**
```bash
wix-security scan myapp.msi
wix-security scan myproject.wxs
wix-security report myapp.msi --format sarif
```

**Checks:**
- Custom actions running as SYSTEM
- Writable paths used by elevated actions
- Repair hijacking vulnerabilities
- DLL search order issues
- Temp folder extraction risks

**Output:**
```
Security Scan: myapp.msi
========================

HIGH: Custom action "CA_RunScript" runs elevated
  Location: CustomAction table, row 3
  Risk: Privilege escalation via repair
  Fix: Add Impersonate="yes" or schedule differently
  CVE: Similar to CVE-2024-38014

MEDIUM: File extracted to %TEMP%
  Location: Binary table, "helper.exe"
  Risk: Binary replacement attack
  Fix: Use secure extraction path

Found: 1 high, 1 medium, 0 low
```

**Solves:**
- MSI Security Vulnerabilities
- Security Vulnerability Scanning gap

---

### 17. Upgrade Validator

**Purpose:** Validate upgrade/patch rules before build.

**Usage:**
```bash
wix-upgrade check v1.wxs v2.wxs
wix-upgrade validate myproject.wxs
wix-upgrade simulate v1.msi v2.msi
```

**Checks:**
- Component GUID changes
- Component rule violations
- Feature tree changes
- ProductCode/UpgradeCode consistency
- Version number format
- Minor vs major upgrade requirements

**Solves:**
- Patching and Upgrade Complexity
- Component Rules Complexity

---

### 18. Component Validator

**Purpose:** Deep component rule checking.

**Usage:**
```bash
wix-component validate myproject.wxs
wix-component check-guids myproject.wxs
wix-component find-duplicates myproject.wxs
```

**Checks:**
- Duplicate GUIDs
- Missing KeyPath
- Multiple files per component
- Invalid GUID format (uppercase)
- Orphaned components
- Cross-feature component sharing

**Solves:**
- Component Rules Complexity
- ICE08 prevention

---

### 19. Bundle Analyzer

**Purpose:** Visualize Burn bootstrapper chains.

**Usage:**
```bash
wix-bundle analyze mybundle.exe
wix-bundle graph mybundle.exe
wix-bundle packages mybundle.exe
```

**Output:**
```
Bundle: mybundle.exe
====================

Chain:
  1. [ExePackage] vcredist_x64.exe (v14.32)
     Condition: NOT VCRedist_Installed

  2. [MsiPackage] prereq.msi (v1.0)
     Condition: NOT Prereq_Installed

  3. [MsiPackage] myapp.msi (v2.0)
     Condition: (always)

Cache: C:\ProgramData\Package Cache\{...}
```

**Solves:**
- Burn Bootstrapper Issues

---

### 20. Property Discoverer

**Purpose:** Extract silent install parameters from MSI.

**Usage:**
```bash
wix-props discover myapp.msi
wix-props document myapp.msi --format markdown
```

**Output:**
```
Silent Install Properties: myapp.msi
====================================

Required:
  INSTALLDIR     Installation directory
                 Default: [ProgramFilesFolder]\MyCompany\MyApp

Optional:
  ADDLOCAL       Features to install (comma-separated)
                 Available: Main,Tools,Docs

  DESKTOP_SHORTCUT  Create desktop shortcut
                    Values: 0, 1 (default: 1)

Example:
  msiexec /i myapp.msi /qn INSTALLDIR="C:\MyApp" ADDLOCAL=Main,Tools
```

**Solves:**
- Silent Install Parameter Discovery gap
- Enterprise deployment documentation

---

### 21-23. CI/CD Integration

**GitHub Action:**
```yaml
- uses: wixcraft/wix-action@v1
  with:
    command: lint
    path: src/installer/
```

**Azure DevOps Task:**
```yaml
- task: WixCraft@1
  inputs:
    command: 'lint'
    path: 'src/installer/'
```

**Pre-commit:**
```yaml
repos:
  - repo: https://github.com/wixcraft/wixcraft
    hooks:
      - id: wix-lint
      - id: wix-fmt
```

**Solves:**
- CI/CD Pipeline Issues
- No native GitHub Actions gap

---

## Tool Dependency Graph

```
                    ┌─────────────────┐
                    │   WiX Plugin    │
                    │     (YAML)      │
                    └────────┬────────┘
                             │
                    ┌────────┴────────┐
                    │     Engine      │
                    │    (Generic)    │
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
   ┌─────────┐         ┌─────────┐         ┌─────────┐
   │   LSP   │         │   CLI   │         │ Analysis│
   │ Server  │         │  Tools  │         │  Tools  │
   └────┬────┘         └────┬────┘         └────┬────┘
        │                   │                   │
   ┌────┴────┐         ┌────┴────┐         ┌────┴────┐
   │ VS Code │         │ Linter  │         │   MSI   │
   │ Sublime │         │Formatter│         │Inspector│
   └─────────┘         │  Help   │         │  Logs   │
                       │ Migrate │         │Security │
                       └─────────┘         └─────────┘
```

---

## Implementation Priority

### Phase 1: Foundation
1. Engine (generic)
2. WiX Plugin (YAML data)
3. Linter (CLI)
4. Formatter (CLI)

### Phase 2: Editor Support
5. LSP Server
6. VS Code Extension
7. TextMate Grammar

### Phase 3: Documentation
8. Help Browser
9. Error Translator
10. Hover docs (LSP)

### Phase 4: Analysis
11. MSI Inspector
12. Log Analyzer
13. Component Validator

### Phase 5: Migration & Generation
14. Migration Tool
15. Code Generator
16. Harvester

### Phase 6: Advanced
17. Security Scanner
18. Upgrade Validator
19. Bundle Analyzer
20. Property Discoverer

### Phase 7: CI/CD
21. GitHub Action
22. Azure DevOps Task
23. Pre-commit hooks

---

## Summary

| Category | Count |
|----------|-------|
| Core/Engine | 3 |
| Editor Extensions | 3 |
| CLI Tools | 7 |
| Analysis Tools | 7 |
| CI/CD | 3 |
| **Total** | **23** |
