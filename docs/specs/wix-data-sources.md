# wix-data: Data Sources Specification

## Sources Overview

| Source | Type | URL | What It Contains |
|--------|------|-----|------------------|
| WiX XSD Schema (v4/v5/v6) | XML Schema | [GitHub](https://github.com/wixtoolset/web/blob/master/src/xsd4/wix.xsd) | Element names, attributes, types, hierarchy |
| WiX v3 XSD | XML Schema | [GitHub](https://github.com/wixtoolset/wix3/blob/develop/src/tools/wix/Xsd/wix.xsd) | Legacy v3 elements |
| WiX Extension XSDs | XML Schema | Various | Extension elements (Util, NetFx, UI, etc.) |
| MSI Database Tables | Microsoft Docs | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/msi/database-tables) | MSI table structure, columns, types |
| ICE Validation Reference | Microsoft Docs | [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/msi/internal-consistency-evaluators-ices) | ICE01-ICE99+ validation rules |
| WiX Documentation | Website | [wixtoolset.org](https://wixtoolset.org/docs/) | Element descriptions, examples |
| FireGiant Docs | Website | [docs.firegiant.com](https://docs.firegiant.com/) | Tutorials, best practices |
| WiX Source Code | C#/Rust | [GitHub](https://github.com/wixtoolset/wix) | Actual implementation, validation logic |

---

## What Data We Need From Each Source

### 1. WiX XSD Schema (PRIMARY SOURCE)

```
Extracted Data:
├── Elements
│   ├── Name (e.g., "Component", "File", "Directory")
│   ├── Documentation/Description
│   ├── Parent elements allowed
│   ├── Child elements allowed
│   └── Namespace (Wix, Util, NetFx, etc.)
│
├── Attributes
│   ├── Name (e.g., "Id", "Guid", "Source")
│   ├── Type (string, guid, yesno, integer, etc.)
│   ├── Required (yes/no)
│   ├── Default value
│   ├── Documentation/Description
│   └── Valid values (enums)
│
└── Types
    ├── Simple types (Guid, YesNoType, etc.)
    ├── Patterns/Regex
    └── Restrictions
```

**Extraction method:** Parse XSD with XML parser, extract elements/attributes/types

### 2. MSI Database Tables (SECONDARY SOURCE)

```
Extracted Data:
├── Tables
│   ├── Name (Component, Feature, File, Directory, etc.)
│   ├── Columns
│   │   ├── Name
│   │   ├── Type (s0-s255, i2, i4, S0-S255, etc.)
│   │   ├── Nullable
│   │   └── Key (primary/foreign)
│   └── Relationships (foreign keys)
│
└── Mapping
    └── WiX Element → MSI Table(s)
```

**Why needed:** Understand what WiX elements map to in the final MSI

### 3. ICE Validation Reference

```
Extracted Data:
├── ICE Rules
│   ├── ICE Number (ICE01, ICE02, ... ICE99+)
│   ├── Description
│   ├── Tables affected
│   ├── Error/Warning messages
│   └── Severity (error, warning, info)
│
└── Common Errors
    ├── Error code
    ├── Description
    └── Resolution steps
```

**Why needed:** Power `wix-lint` rules and `wix-doctor` error explanations

### 4. WiX Documentation

```
Extracted Data:
├── Element Descriptions (human-readable)
├── Usage Examples (code snippets)
├── Best Practices
├── Common Patterns
└── Version-specific Notes (v3 vs v4 vs v5 vs v6)
```

**Why needed:** Provide hover docs in LSP, documentation in `wix-docs`

### 5. WiX Source Code

```
Extracted Data:
├── Validation Logic (what the compiler actually checks)
├── Default Values (runtime defaults)
├── Error Messages (actual error strings)
└── Version Differences (what changed between versions)
```

**Why needed:** Ensure our lint rules match actual WiX behavior

---

## What Data Is Useful

| Data | Used By | Priority |
|------|---------|----------|
| Element names | wix-syntax, wix-lsp, wix-lint | Critical |
| Attribute names + types | wix-lsp, wix-lint, wix-schema | Critical |
| Required attributes | wix-lint | Critical |
| Element hierarchy (parent/child) | wix-lsp, wix-lint | Critical |
| Descriptions | wix-lsp (hover), wix-docs | High |
| Valid enum values | wix-lsp (autocomplete), wix-lint | High |
| Default values | wix-lsp, wix-fmt | Medium |
| Examples | wix-docs, wix-snippets | Medium |
| ICE rules | wix-lint, wix-doctor | High |
| MSI table mappings | wix-msi, wix-doctor | Medium |
| Version differences | wix-migrate | High |
| Error messages | wix-doctor | High |

---

## What Is Missing (Not in Official Sources)

| Missing Data | Why Missing | How To Get |
|--------------|-------------|------------|
| Best practices | Not in schema | Manual curation, community input |
| Common mistakes | Not documented | Analyze StackOverflow, GitHub issues |
| Performance tips | Tribal knowledge | Manual curation |
| Real-world examples | Schema is abstract | Collect from open source projects |
| Lint rule severity | Our judgment | Manual curation |
| Formatting preferences | Subjective | Define our own standard |
| YAML mappings (for wixcraft) | Doesn't exist | Create our own |

---

## Update Strategy

### Do We Need To Update?

| Scenario | Update Needed? | Frequency |
|----------|----------------|-----------|
| New WiX version (v5→v6) | Yes | ~1-2 years |
| New WiX extension | Yes | Occasional |
| Bug fix in our data | Yes | As needed |
| New lint rules | Yes | Ongoing |
| Community contributions | Yes | Ongoing |

### How To Update

```
Option 1: Manual Update (Simple)
├── Download new XSD from WiX GitHub
├── Run parser script to extract JSON
├── Diff against existing data
├── Review changes manually
└── Commit updates

Option 2: Automated Pipeline (Better)
├── GitHub Action monitors WiX releases
├── Auto-downloads new XSD on release
├── Runs extraction script
├── Creates PR with changes
├── Human reviews and merges
└── Publishes new wix-data version

Option 3: Semi-Automated (Recommended)
├── CLI command: `wix-data update`
├── Fetches latest XSD from WiX GitHub
├── Parses and generates JSON
├── Shows diff of changes
├── User confirms update
└── Commits to local wix-data
```

### Update Triggers

| Trigger | Action |
|---------|--------|
| WiX releases new version | Run extraction, update version data |
| Community reports missing element | Add manually, submit upstream if applicable |
| New lint rule needed | Add to rules/ directory |
| Error message improvement | Update descriptions |

---

## Extraction Scripts Needed

| Script | Input | Output |
|--------|-------|--------|
| `extract-xsd.rs` | wix.xsd | elements/*.json, attributes/*.json |
| `extract-msi-tables.rs` | Microsoft docs (manual) | msi-tables/*.json |
| `extract-ice-rules.rs` | Microsoft docs (manual) | rules/ice-*.json |
| `merge-docs.rs` | WiX docs site | Adds descriptions to elements |
| `diff-versions.rs` | v3.xsd, v4.xsd, v5.xsd | versions/*.json |

---

## Data Freshness

| Data Type | Source Stability | Update Cadence |
|-----------|------------------|----------------|
| Core WiX elements | Stable (rarely changes) | Per WiX major version |
| Extension elements | Moderate | Per extension release |
| MSI tables | Very stable (Windows API) | Rarely |
| ICE rules | Very stable | Rarely |
| Lint rules | We control | Ongoing |
| Descriptions | We can improve | Ongoing |

---

## Recommended Approach

1. **Initial Population**
   - Parse WiX v4/v5/v6 XSD → generate element/attribute JSON
   - Manually add MSI table mappings from Microsoft docs
   - Manually add ICE rules from Microsoft docs
   - Add descriptions from WiX documentation

2. **Ongoing Maintenance**
   - Monitor WiX releases for schema changes
   - Accept community PRs for improvements
   - Add new lint rules as patterns emerge

3. **Version Support**
   - Support v3, v4, v5, v6 simultaneously
   - Store version differences in `versions/` directory
   - Tools can target specific version

---

## Sources

- [WiX v4/v5 XSD Schema](https://github.com/wixtoolset/web/blob/master/src/xsd4/wix.xsd)
- [WiX v3 XSD Schema](https://github.com/wixtoolset/wix3/blob/develop/src/tools/wix/Xsd/wix.xsd)
- [WiX Schema Reference](https://wixtoolset.org/docs/v3/xsd/)
- [MSI Database Tables](https://learn.microsoft.com/en-us/windows/win32/msi/database-tables)
- [ICE Validation Reference](https://learn.microsoft.com/en-us/windows/win32/msi/internal-consistency-evaluators-ices)
- [FireGiant Docs](https://docs.firegiant.com/)
