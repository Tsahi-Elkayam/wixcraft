# WixCraft Repository Structure

Monorepo design - all tools under one roof, independently buildable.

## Total: 40 Tools

```
wixcraft/
├── docs/                          # Documentation
│   ├── pain-points.md
│   ├── repo-structure.md
│   └── specs/                     # Tool specifications
│       ├── wixcraft.md
│       ├── wix-lsp.md
│       └── ...
│
├── src/                           # Source code for all tools
│   │
│   ├── core/                      # Shared core libraries
│   │   ├── wix-data/              # WiX element/attribute database (JSON)
│   │   ├── wix-parser/            # WXS/XML parser
│   │   ├── wix-schema/            # XSD schema definitions
│   │   ├── wix-msi-lib/           # MSI file format library (OLE/CFB)
│   │   └── wix-common/            # Shared utilities
│   │
│   ├── engine/                    # Core engine (2 tools)
│   │   ├── wixcraft/              # YAML-to-MSI engine
│   │   └── wix-msi/               # Cross-platform MSI compiler
│   │
│   ├── ide/                       # IDE & Editor support (5 tools)
│   │   ├── wix-lsp/               # Language Server Protocol
│   │   ├── wix-vscode/            # VS Code extension
│   │   ├── wix-sublime/           # Sublime Text package
│   │   ├── wix-syntax/            # Syntax highlighting (VS Code + Sublime)
│   │   └── wix-snippets/          # Code snippets
│   │
│   ├── quality/                   # Code quality tools (3 tools)
│   │   ├── wix-lint/              # Linter
│   │   ├── wix-fmt/               # Formatter
│   │   └── wix-schema/            # Schema validator
│   │
│   ├── debug/                     # Debugging & Testing (5 tools)
│   │   ├── wix-doctor/            # Error decoder & log analyzer
│   │   ├── wix-test/              # Installer testing
│   │   ├── wix-preview/           # Live preview
│   │   ├── wix-diff/              # MSI version comparison
│   │   └── wix-lux/               # Custom action testing
│   │
│   ├── authoring/                 # Authoring helpers (8 tools)
│   │   ├── wix-init/              # Project scaffolder
│   │   ├── wix-harvest/           # File harvester
│   │   ├── wix-ui/                # Dialog generator
│   │   ├── wix-prereq/            # Prerequisites helper
│   │   ├── wix-docs/              # Offline documentation
│   │   ├── wix-wizard/            # Interactive wizard
│   │   ├── wix-license/           # License validation helper
│   │   └── wix-env/               # Environment variable helper
│   │
│   ├── i18n/                      # Localization & Signing (2 tools)
│   │   ├── wix-sign/              # Code signing helper
│   │   └── wix-i18n/              # Multi-language wizard
│   │
│   ├── migration/                 # Migration & Import (2 tools)
│   │   ├── wix-migrate/           # Version migration (v3→v6)
│   │   └── wix-import/            # Import from other tools
│   │
│   ├── patching/                  # Patching & Updates (1 tool)
│   │   └── wix-patch/             # MSP patch generator
│   │
│   ├── runtime/                   # Runtime & Terminal (4 tools)
│   │   ├── wix-repl/              # Interactive shell
│   │   ├── wix-install/           # Install script runner
│   │   ├── wix-update/            # Update script runner
│   │   └── wix-uninstall/         # Uninstall script runner
│   │
│   ├── build/                     # Build & CI/CD (3 tools)
│   │   ├── wix-bundle/            # Bootstrapper wizard
│   │   ├── wix-build/             # Unified build CLI
│   │   └── wix-ci/                # CI/CD templates
│   │
│   └── analytics/                 # Analytics & Advanced (5 tools)
│       ├── wix-analytics/         # Installation telemetry
│       ├── wix-repair/            # Self-healing config
│       ├── wix-silent/            # Silent install generator
│       ├── wix-msix/              # MSIX conversion
│       └── wix-deps/              # Dependency resolver
│
├── packages/                      # Published packages (npm, crates.io)
│   └── ...
│
├── examples/                      # Example projects
│   ├── basic-app/
│   ├── service-installer/
│   ├── bundle-with-prereqs/
│   └── ...
│
├── tests/                         # Integration tests
│   └── ...
│
├── scripts/                       # Build & release scripts
│   └── ...
│
├── .github/                       # GitHub Actions workflows
│   └── workflows/
│
├── Cargo.toml                     # Rust workspace config
├── package.json                   # Root package.json (for JS tools)
├── pnpm-workspace.yaml            # pnpm workspace config
└── README.md
```

## Design Principles

1. **Independent packages** - Each tool can be built, tested, and published independently
2. **Shared core** - Common parsing, schema, and utilities in `src/core/`
3. **Logical grouping** - Tools grouped by function under `src/`
4. **Cross-platform first** - Rust for CLI tools, TypeScript for editor extensions
5. **No runtime dependencies** - CLI tools compile to single binaries

## Technology Stack

### Hybrid Approach

| Tool Type | Language | Reason |
|-----------|----------|--------|
| Core CLI tools | **Rust** | Single binary, no runtime, fast, cross-platform |
| MSI compiler (`wix-msi`) | **Rust** | Low-level file format handling |
| LSP server (`wix-lsp`) | **Rust** | Performance for IDE responsiveness |
| VS Code extension | **TypeScript** | Required by VS Code API |
| Sublime Text package | **Python** | Required by Sublime Text API |
| CI templates | **YAML/Shell** | Native to CI systems |

### Rust Stack

| Layer | Technology | Reason |
|-------|------------|--------|
| Language | Rust | Cross-platform, single binary |
| XML Parser | quick-xml / roxmltree | Fast, zero-copy XML parsing |
| OLE/CFB | cfb / ole | MSI file format (Compound File Binary) |
| Cabinet | cab | .cab file creation for MSI |
| CLI | clap | Industry standard CLI framework |
| Error handling | miette / anyhow | Beautiful error messages |
| Testing | cargo test | Built-in |

### Editor Extension Stacks

**VS Code (TypeScript)**

| Layer | Technology | Reason |
|-------|------------|--------|
| Language | TypeScript | Required by VS Code |
| LSP Client | vscode-languageclient | Standard LSP client |
| Package Manager | pnpm | Fast, efficient |
| Build | tsup / esbuild | Fast bundling |

**Sublime Text (Python)**

| Layer | Technology | Reason |
|-------|------------|--------|
| Language | Python 3.8+ | Required by Sublime Text |
| LSP Client | LSP package | Standard Sublime LSP |
| Package Format | .sublime-package | Sublime's plugin format |

## Core: wix-data (Single Source of Truth)

All tools consume from this central database:

```
src/core/wix-data/
├── elements/                    # WiX element definitions
│   ├── package.json
│   ├── component.json
│   ├── directory.json
│   ├── file.json
│   ├── feature.json
│   ├── registry.json
│   └── ...
├── attributes/                  # Shared attribute types
│   ├── guid.json
│   ├── identifier.json
│   └── ...
├── keywords/                    # Reserved keywords
│   └── keywords.json
├── rules/                       # Lint rule definitions
│   ├── component-rules.json
│   ├── feature-rules.json
│   └── ...
├── snippets/                    # Snippet templates
│   └── snippets.json
├── versions/                    # WiX version differences (v3, v4, v5, v6)
│   ├── v3.json
│   ├── v4.json
│   └── ...
└── index.json                   # Master index
```

**Consumers:**

| Tool | Consumes |
|------|----------|
| **Core Engine** | |
| `wixcraft` | elements, attributes → YAML-to-WXS translation |
| `wix-msi` | elements → MSI table generation |
| **IDE & Editor** | |
| `wix-syntax` | elements, keywords → syntax highlighting |
| `wix-lsp` | elements, attributes, descriptions → autocomplete, hover docs |
| `wix-snippets` | snippets → code templates |
| **Code Quality** | |
| `wix-lint` | rules, valid values, patterns → validation |
| `wix-fmt` | element/attribute ordering → formatting |
| `wix-schema` | elements, attributes → XSD generation |
| **Debugging & Testing** | |
| `wix-doctor` | error codes, elements → error explanations |
| `wix-preview` | elements → UI preview rendering |
| `wix-diff` | elements → structural comparison |
| **Authoring** | |
| `wix-init` | templates, elements → project scaffolding |
| `wix-harvest` | elements → file harvesting output |
| `wix-ui` | dialog elements → UI generation |
| `wix-wizard` | all elements → interactive wizard |
| `wix-docs` | all → documentation generation |
| **Migration** | |
| `wix-migrate` | versions → v3/v4/v5/v6 migration rules |
| `wix-import` | elements → mapping from NSIS/InnoSetup |

## Per-Tool Structure

### Rust Tool Example

```
src/quality/wix-lint/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── rules/               # Lint rules
│   │   ├── mod.rs
│   │   ├── component.rs
│   │   ├── feature.rs
│   │   └── ...
│   └── ...
├── tests/                   # Integration tests
├── Cargo.toml               # Rust package manifest
└── README.md
```

### VS Code Extension (TypeScript)

```
src/ide/wix-vscode/
├── src/
│   ├── extension.ts         # Extension entry point
│   ├── client.ts            # LSP client
│   └── ...
├── syntaxes/                # TextMate grammars
├── package.json             # VS Code extension manifest
├── tsconfig.json
└── README.md
```

### Sublime Text Package (Python)

```
src/ide/wix-sublime/
├── src/
│   ├── plugin.py            # Plugin entry point
│   ├── client.py            # LSP client wrapper
│   └── ...
├── syntaxes/
│   └── WXS.sublime-syntax   # Syntax highlighting
├── snippets/                # Code snippets
├── settings/
│   └── WXS.sublime-settings
└── README.md
```

## Dependency Graph

```
                        ┌─────────────────────────────────────────┐
                        │              src/core/                  │
                        │                                         │
                        │  ┌─────────────────────────────────┐    │
                        │  │  wix-data (JSON database)       │ ◄──┼─── Single Source of Truth
                        │  └───────────────┬─────────────────┘    │
                        │                  │                      │
                        │  ┌───────────────┴─────────────────┐    │
                        │  │  wix-parser (XML/WXS parsing)   │    │
                        │  └───────────────┬─────────────────┘    │
                        │                  │                      │
                        │  ┌───────────────┴─────────────────┐    │
                        │  │  wix-schema (XSD validation)    │    │
                        │  └───────────────┬─────────────────┘    │
                        │                  │                      │
                        │  ┌───────────────┴─────────────────┐    │
                        │  │  wix-msi-lib (OLE/CFB/CAB)      │    │
                        │  └─────────────────────────────────┘    │
                        └──────────────────┬──────────────────────┘
                                           │
     ┌─────────────────────────────────────┼─────────────────────────────────────┐
     │                    │                │                │                    │
     ▼                    ▼                ▼                ▼                    ▼
┌─────────┐       ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       ┌──────────┐
│wix-lint │       │  wix-lsp    │  │  wix-msi    │  │  wix-fmt    │       │wix-syntax│
└─────────┘       └──────┬──────┘  └──────┬──────┘  └─────────────┘       └──────────┘
                         │                │
          ┌──────────────┴──────────────┐ │
          ▼                             ▼ │
    ┌──────────┐                 ┌──────────┐
    │wix-vscode│                 │wix-sublime│
    └──────────┘                 └──────────┘
                                              │
                                              ▼
                                     ┌─────────────────┐
                                     │    wixcraft     │
                                     │ (YAML-to-MSI)   │
                                     └─────────────────┘
```

## Cross-Platform Support

| Component | Windows | Mac | Linux |
|-----------|---------|-----|-------|
| All Rust CLI tools | ✅ | ✅ | ✅ |
| `wix-msi` (MSI compiler) | ✅ | ✅ | ✅ |
| `wix-lsp` | ✅ | ✅ | ✅ |
| `wix-vscode` | ✅ | ✅ | ✅ |
| `wix-sublime` | ✅ | ✅ | ✅ |
| `wix-test` (actual install) | ✅ | ❌ | ❌ |
| `wix-sign` (code signing) | ✅ | ❌ | ❌ |

## Build Commands

### Rust

```bash
# Build all Rust packages
cargo build --release

# Build specific package
cargo build -p wix-lint --release

# Run tests
cargo test

# Run specific tool
cargo run -p wix-lint -- check ./installer.wxs
```

### VS Code Extension (TypeScript)

```bash
# Install dependencies
pnpm install

# Build VS Code extension
pnpm --filter wix-vscode build

# Package extension
pnpm --filter wix-vscode package
```

### Sublime Text Package (Python)

```bash
# No build step needed - Python is interpreted
# Package for distribution
cd src/ide/wix-sublime
zip -r wix-sublime.sublime-package .
```

### Full Build

```bash
# Build everything
./scripts/build-all.sh

# Create release artifacts
./scripts/release.sh
```
