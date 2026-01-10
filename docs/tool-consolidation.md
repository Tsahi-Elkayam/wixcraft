# Tool Consolidation Plan

Mapping existing tools in `unsorted/` to final tool structure.

---

## Current Inventory (53 Tools)

### Quality/Linting
| Tool | Description | Status |
|------|-------------|--------|
| wix-analyzer | Static code analysis - validation, best practices, security, auto-fix | Main |
| winter | WiX Linter - fast, real-time linting for IDE | Main |
| wix-fmt | WiX XML formatter | Main |

### Common/Libraries
| Tool | Description | Status |
|------|-------------|--------|
| wix-symbols | Document symbols extractor | Library |
| ice-validator | Cross-platform ICE validator | Library |
| wix-hover | Hover documentation provider | Library |
| wixkb | SQLite knowledge base | Library |
| wix-references | Go to Definition, Find References | Library |

### Editor/IDE
| Tool | Description | Status |
|------|-------------|--------|
| wintellisense | Autocomplete engine | Library |
| wix-vscode | VS Code extension generator | Main |
| wix-syntax | Syntax highlighting definitions | Data |
| wix-sublime | Sublime Text package generator | Main |
| wix-snippets | Snippet library | Data |
| wix-lsp | Language Server Protocol | Main |

### MSI Tools
| Tool | Description | Status |
|------|-------------|--------|
| msi-explorer | MSI database explorer (CLI + GUI) | Main |
| wix-msi | MSI package builder/analyzer | Absorb |

### Core/Engine
| Tool | Description | Status |
|------|-------------|--------|
| wixcraft | Universal tooling framework | Main |
| code-detector | File type detection | Library |
| project-map | Project indexer | Library |
| schema-loader | Language pack loader | Library |

### Documentation
| Tool | Description | Status |
|------|-------------|--------|
| wix-docs | Documentation generator | Main |
| wix-help | CLI help system | Main |

### Authoring
| Tool | Description | Status |
|------|-------------|--------|
| wix-init | Project scaffolder | Main |
| wix-harvest | File harvester | Main |
| wix-patch | Patch/MSP generation | Main |
| wix-env | Environment variable helper | Absorb |
| wix-ui | UI sequence generator | Main |
| wix-prereq | Prerequisites helper | Main |
| wix-wizard | Interactive wizard | Absorb |
| wix-license | License file generator | Absorb |
| wix-guid | GUID generator | Main |

### Build
| Tool | Description | Status |
|------|-------------|--------|
| wix-ci | CI/CD templates | Main |
| wix-build | Unified build CLI | Main |
| wix-bundle | Burn bootstrapper wizard | Main |

### Migration
| Tool | Description | Status |
|------|-------------|--------|
| wix-migrate | v3->v4->v5->v6 migration | Main |
| wix-import | Import from NSIS/InnoSetup | Main |

### Runtime
| Tool | Description | Status |
|------|-------------|--------|
| wix-install | MSI installer runner | Absorb -> wix-init |
| wix-update | MSI update manager | Absorb -> wix-init |
| wix-uninstall | MSI uninstaller | Absorb -> wix-init |
| wix-repl | Interactive REPL | Main |

### Signing
| Tool | Description | Status |
|------|-------------|--------|
| wix-sign | Code signing helper | Main |

### Localization
| Tool | Description | Status |
|------|-------------|--------|
| wix-i18n | Localization helper | Main |

### Analytics
| Tool | Description | Status |
|------|-------------|--------|
| wix-repair | MSI repair tool | Absorb -> wix-init |
| wix-msix | MSIX converter | Main |
| license-detector | License detection | Absorb -> wix-analyzer |
| wix-analytics | MSI analytics | Absorb -> wix-analyzer |
| wix-silent | Silent install config | Absorb -> wix-init |
| wix-deps | Dependency analyzer | Absorb -> wix-analyzer |

### Debug
| Tool | Description | Status |
|------|-------------|--------|
| wix-test | Testing framework for MSI | Main |
| wix-lux | Unit test runner for CAs | Absorb -> wix-test |
| wix-preview | Preview content | Absorb -> wix-build |
| wix-diff | Compare versions | Main |
| wix-doctor | Error decoder, log analyzer, project debugger | Main |

---

## Tool Roles: Linter vs Analyzer

**Important distinction:** These tools serve different workflows and remain separate.

| Tool | Purpose | When Used |
|------|---------|-----------|
| **winter** | Real-time linter | While coding in IDE, instant feedback |
| **wix-analyzer** | Static code analysis | Code review, CI/CD, batch analysis |

**winter** provides fast, lightweight checks optimized for IDE integration:
- Sub-100ms response time
- Incremental parsing
- Focus on syntax and common errors

**wix-analyzer** provides deep analysis for code review:
- Best practices validation
- Security vulnerability detection
- Dependency analysis
- Auto-fix suggestions
- Full project analysis

Both tools share the same rule definitions from `wixkb` but apply them differently.

---

## Consolidation Plan

### 1. LSP Integration

**wix-lsp** orchestrates libraries for editor support:

```
wix-lsp
├── uses: wix-symbols (documentSymbol)
├── uses: wix-references (definition, references)
├── uses: wintellisense (completion)
├── uses: wix-hover (hover)
├── uses: winter (real-time diagnostics)
└── uses: wix-fmt (formatting)
```

**Action:** Keep all as libraries, wix-lsp orchestrates.

---

### 2. Knowledge Base Consolidation

**wixkb** is the data layer. Others consume it:

```
wixkb (SQLite database)
├── used by: wix-help (CLI lookup)
├── used by: wix-hover (hover content)
├── used by: wintellisense (completions)
├── used by: winter (lint rules)
└── used by: wix-analyzer (analysis rules)
```

**Action:** Keep wixkb as central data, others as consumers.

---

### 3. Editor Package Consolidation

Keep separate generators, share data:

```
wix-syntax/     (TextMate grammar - data)
wix-snippets/   (snippets - data)
    │
    ├── wix-vscode (generates VS Code extension)
    └── wix-sublime (generates Sublime package)
```

**Action:** Keep structure, ensure data sharing.

---

### 4. Project Init: wix-wizard -> wix-init

**wix-init** becomes the unified project starting point:

| Feature | Source |
|---------|--------|
| Template-based scaffolding | wix-init |
| Interactive wizard mode | wix-wizard |
| Environment check | new |
| IDE setup | new |
| GUID generation | wix-guid |
| License templates | wix-license |

**Action:** Merge wix-wizard, wix-guid, wix-license into wix-init.

```bash
wix-init new MyApp                    # Quick CLI mode
wix-init new MyApp --wizard           # Interactive wizard
wix-init doctor                       # Check WiX installed, IDE ready
wix-init setup                        # Install WiX, configure IDE
wix-init setup --no-ide               # WiX only, skip IDE
wix-init setup --sandbox              # Enable Windows Sandbox for testing
wix-init guid                         # Generate GUID
wix-init sandbox                      # Generate .wsb file for testing MSI
```

Windows Sandbox support (Windows only):
- Enable Windows Sandbox feature if not enabled
- Generate `.wsb` config file for testing MSI in isolation
- Map project folder into sandbox
- Auto-run MSI install in sandbox

---

### 5. MSI Runtime -> wix-init

Runtime tools merge into wix-init (project lifecycle):

| Old Tool | New Command |
|----------|-------------|
| wix-install | `wix-init run` or `wix-init install` |
| wix-uninstall | `wix-init uninstall` |
| wix-update | `wix-init update` |
| wix-repair | `wix-init repair` |
| wix-silent | `wix-init run --silent` |

**Action:** wix-init handles full project lifecycle.

```bash
wix-init new MyApp                    # Create project
wix-init setup                        # Setup environment
wix-init build                        # Build MSI (calls wix build)
wix-init install                      # Install built MSI
wix-init uninstall                    # Uninstall
wix-init sandbox                      # Test in Windows Sandbox
```

---

### 5b. msi-explorer = Orca on Steroids

**msi-explorer** is purely for MSI database inspection (not runtime):

- Browse MSI tables (File, Component, Registry, etc.)
- View/search table rows
- Export tables to CSV/JSON
- Compare two MSI files
- Validate MSI structure
- GUI + CLI modes

```bash
msi-explorer open Product.msi         # Open in GUI
msi-explorer tables Product.msi       # List tables
msi-explorer query Product.msi "SELECT * FROM File"
msi-explorer diff v1.msi v2.msi       # Compare MSIs
msi-explorer export Product.msi --format json
```

---

### 6. Testing Tools: wix-test + wix-lux

Different purposes, complementary:

| Tool | Tests What | Purpose |
|------|-----------|---------|
| **wix-test** | Built MSI package | Structure, files, registry, components |
| **wix-lux** | Custom action DLLs | CA code with mock session |

**Action:** Keep both, unify under `wix-test` with subcommands:

```bash
wix-test msi Product.msi             # Test MSI structure (from wix-test)
wix-test ca MyActions.dll            # Test custom actions (from wix-lux)
wix-test suite tests.json            # Run test suite
wix-test sandbox Product.msi         # Test in Windows Sandbox
```

---

### 7. Analytics Merge -> wix-analyzer

All analysis tools merge into wix-analyzer as subcommands:

| Old Tool | New Command | Purpose |
|----------|-------------|---------|
| wix-deps | `wix-analyzer deps` | Binary/runtime dependencies |
| wix-analytics | `wix-analyzer metrics` | Telemetry config generation |
| license-detector | `wix-analyzer licenses` | License compliance scan |

**Action:** wix-analyzer becomes unified analysis hub.

```bash
wix-analyzer check Product.wxs           # Code quality (default)
wix-analyzer deps Product.wxs            # Dependency analysis
wix-analyzer deps --graph                # Dependency graph
wix-analyzer licenses ./src              # License compliance scan
wix-analyzer licenses --notice           # Generate NOTICE file
wix-analyzer metrics --init              # Setup analytics telemetry
```

---

### 8. Build & Debug Tools

**wix-preview** merges into **wix-build** (build lifecycle):

```bash
wix-build preview Product.wxs        # What will this install?
wix-build compile Product.wxs        # Compile to .wixobj
wix-build link *.wixobj              # Link to MSI
wix-build Product.wxs                # Full build (compile + link)
wix-build clean                      # Clean build artifacts
```

**wix-doctor** stays standalone as project debugger:

```bash
wix-doctor error 1603                # Decode MSI error code
wix-doctor log install.log           # Analyze verbose log
wix-doctor search "access denied"    # Search error database
wix-doctor diagnose .                # Diagnose project issues
```

---

### 9. Small Tools Absorption

| Tool | Absorb Into | As |
|------|-------------|-----|
| wix-env | wix-init | Template helper |
| wix-license | wix-init | License template |
| wix-guid | tools/wix-guid | Standalone GUID tool |
| wix-msi | msi-explorer | Subcommand |

---

## Final Tool List (Consolidated)

### Core (5)
| Tool | Type | Description |
|------|------|-------------|
| **wixcraft** | CLI | Main entry point, subcommand router |
| **code-detector** | Library | File type detection |
| **project-map** | Library | Project indexer with symbol graph |
| **schema-loader** | Library | Plugin/language pack loader |
| **wixkb** | Library | SQLite knowledge base |

### Quality (3)
| Tool | Type | Description |
|------|------|-------------|
| **winter** | CLI/Library | Real-time linter for IDE |
| **wix-analyzer** | CLI | Static analysis, deps, licenses, metrics |
| **wix-fmt** | CLI | Formatter |

### Editor (4)
| Tool | Type | Description |
|------|------|-------------|
| **wix-lsp** | Service | Language Server (uses libraries below) |
| **wix-vscode** | Generator | VS Code extension |
| **wix-sublime** | Generator | Sublime Text package |
| **wix-syntax** | Data | Syntax highlighting definitions |

### Editor Libraries (4)
| Tool | Type | Description |
|------|------|-------------|
| **wix-symbols** | Library | Document symbols |
| **wix-references** | Library | Go to def, find refs |
| **wintellisense** | Library | Autocomplete |
| **wix-hover** | Library | Hover documentation |

### Documentation (2)
| Tool | Type | Description |
|------|------|-------------|
| **wix-help** | CLI | PowerShell-like help browser |
| **wix-docs** | CLI | Project documentation generator |

### MSI Tools (1)
| Tool | Type | Description |
|------|------|-------------|
| **msi-explorer** | CLI/GUI | Orca on steroids - MSI database inspector |

### Authoring (6)
| Tool | Type | Description |
|------|------|-------------|
| **wix-init** | CLI | Project lifecycle (new, setup, run, uninstall, sandbox) |
| **wix-harvest** | CLI | File harvester |
| **wix-patch** | CLI | Patch generation |
| **wix-ui** | CLI | UI sequence generator |
| **wix-prereq** | CLI | Prerequisites helper |
| **wix-bundle** | CLI | Burn bootstrapper wizard |

### Build (2)
| Tool | Type | Description |
|------|------|-------------|
| **wix-build** | CLI | Unified build + preview |
| **wix-ci** | Generator | CI/CD templates |

### Migration (2)
| Tool | Type | Description |
|------|------|-------------|
| **wix-migrate** | CLI | Version migration (v3->v4->v5->v6) |
| **wix-import** | CLI | Import from NSIS/InnoSetup |

### Debug (4)
| Tool | Type | Description |
|------|------|-------------|
| **wix-doctor** | CLI | Project debugger, error decoder, log analyzer |
| **wix-diff** | CLI | Compare WiX/MSI versions |
| **wix-test** | CLI | MSI testing + CA unit tests + sandbox |
| **wix-repl** | CLI | Interactive REPL |

### Tools (4)
| Tool | Type | Description |
|------|------|-------------|
| **wix-guid** | CLI | GUID generator |
| **wix-sign** | CLI | Code signing |
| **wix-i18n** | CLI | Localization |
| **wix-msix** | CLI | MSIX conversion |

---

## Summary

| Category | Before | After | Reduction |
|----------|--------|-------|-----------|
| Quality | 3 | 3 | 0 |
| Editor | 6 | 4 + 4 libs | 0 |
| Docs | 2 | 2 | 0 |
| MSI | 2 | 1 | -1 |
| Core | 4 | 5 | +1 |
| Authoring | 9 | 6 | -3 |
| Build | 3 | 2 | -1 |
| Migration | 2 | 2 | 0 |
| Runtime | 4 | 0 (merged) | -4 |
| Analytics | 6 | 0 (merged) | -6 |
| Debug | 5 | 4 | -1 |
| Tools | 3 | 4 | +1 |
| **Total** | **53** | **37** | **-16** |

---

## Merge Actions

1. ~~wix-wizard~~ -> **wix-init** (interactive mode)
2. ~~wix-license~~ -> **wix-init** (license templates)
3. ~~wix-env~~ -> **wix-init** (environment helper)
4. ~~wix-install, wix-uninstall, wix-update, wix-repair, wix-silent~~ -> **wix-init** (project lifecycle)
5. ~~wix-lux~~ -> **wix-test** (CA unit testing)
6. ~~wix-msi~~ -> **msi-explorer** (MSI building features)
7. ~~wix-deps, wix-analytics, license-detector~~ -> **wix-analyzer**
8. ~~wix-preview~~ -> **wix-build** (build preview)

---

## Dependency Graph

```
                         ┌──────────────┐
                         │    wixkb     │
                         │  (database)  │
                         └──────┬───────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌──────────────┐       ┌──────────────┐       ┌──────────────┐
│    winter    │       │   wix-lsp    │       │   wix-help   │
│  (rt linter) │       │   (editor)   │       │    (cli)     │
└──────────────┘       └──────┬───────┘       └──────────────┘
                              │
                              │
┌──────────────┐              │
│ wix-analyzer │              │
│  (static)    │              │
└──────────────┘              │
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
       ┌────────────┐  ┌────────────┐  ┌────────────┐
       │wix-symbols │  │wix-referenc│  │wintellisens│
       └────────────┘  └────────────┘  └────────────┘
```
