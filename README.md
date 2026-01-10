# WixCraft

Modern developer tooling for WiX Toolset. 61 tools for linting, formatting, autocomplete, analysis, migration, and more.

## Why WixCraft?

WiX Toolset is powerful but its developer experience is stuck in 2009:
- No VS Code extension
- Cryptic error messages
- Fragmented documentation
- Complex XML syntax
- No cross-platform support

WixCraft fills these gaps with fast, modern CLI tools and editor integrations.

## Quick Start

```bash
# Lint a WiX file
winter Product.wxs

# Format a WiX file
wix-fmt --write Product.wxs

# Get help on any element
wix-help Component

# Explore an MSI file
msi-explorer tables Product.msi

# Migrate WiX v3 to v5
wix-migrate --from v3 --to v5 ./installer/

# Generate installer from simple config
wix-simple generate myapp.json
```

## Installation

### Pre-built Binaries

Download from [Releases](https://github.com/wixcraft/wixcraft/releases).

### Build from Source

Requires Rust 1.70+:

```bash
git clone https://github.com/wixcraft/wixcraft
cd wixcraft

# Build a specific tool
cd quality/winter
cargo build --release

# Binary at target/release/winter
```

## Tools (61 Total)

### Core Engine (5 tools)

| Tool | Description |
|------|-------------|
| **wixcraft** | YAML-to-MSI engine - write simple YAML, get WXS/MSI output |
| **wix-msi** | Cross-platform MSI compiler - build MSI on Mac/Linux |
| **wix-simple** | Generate complete installers from minimal JSON config |
| **schema-loader** | XSD validation and type checking |
| **code-detector** | Code pattern detection for analysis |

### Quality (7 tools)

| Tool | Description |
|------|-------------|
| **winter** | Fast linter with 49+ rules, SARIF output for CI/CD |
| **wix-analyzer** | Unified analyzer: validation, best practices, security |
| **wix-fmt** | XML formatter with WiX-aware element ordering |
| **wix-wdac** | WDAC compatibility checker for enterprise deployments |
| **wix-compat** | Windows Server 2025 / Win11 24H2 compatibility checker |
| **wix-security** | Security vulnerability scanner |
| **wix-upgrade** | Upgrade code and version management |

### Editor Support (10 tools)

| Tool | Description |
|------|-------------|
| **wix-lsp** | Language Server Protocol for any editor |
| **wix-vscode** | VS Code extension |
| **wix-sublime** | Sublime Text package |
| **wix-syntax** | Syntax highlighting (.tmLanguage) |
| **wix-snippets** | Code snippet library |
| **wix-ai** | AI-assisted code generation from natural language |
| **wintellisense** | Context-aware autocomplete engine |
| **wix-hover** | Hover documentation provider |
| **wix-symbols** | Document symbols for outline views |
| **wix-references** | Go to Definition, Find References |

### Debugging & Testing (5 tools)

| Tool | Description |
|------|-------------|
| **wix-doctor** | Error decoder and log analyzer with suggested fixes |
| **wix-test** | Validate install/update/uninstall in sandbox |
| **wix-preview** | Preview installer UI and file layout without building |
| **wix-diff** | Compare MSI versions, show what changed |
| **wix-ca-debug** | Enhanced custom action testing framework |

### Authoring (9 tools)

| Tool | Description |
|------|-------------|
| **wix-init** | Project scaffolder with common scenarios |
| **wix-harvest** | Modern file harvester (Heat replacement) |
| **wix-ui** | Visual dialog generator and editor |
| **wix-prereq** | Prerequisites detection and bundle helper |
| **wix-easy** | Interactive GUI wizard for creating projects |
| **wix-license** | License key/serial validation wizard |
| **wix-env** | Environment variable and PATH configuration helper |
| **wix-patch** | Simplified patch/MSP generation helper |
| **wix-simple** | Generate installers from minimal JSON |

### Build & CI/CD (6 tools)

| Tool | Description |
|------|-------------|
| **wix-build** | Unified build CLI wrapping WiX toolset |
| **wix-bundle** | Wizard and templates for Burn bootstrappers |
| **wix-ci** | Templates for GitHub Actions, Azure DevOps, GitLab CI |
| **wix-intune** | Generate .intunewin packages for Microsoft Intune |
| **wix-arm64** | ARM64 multi-platform build helper |
| **wix-ext** | Dependency detection and resolution |

### MSI Tools (7 tools)

| Tool | Description |
|------|-------------|
| **msi-explorer** | MSI database inspector (CLI + GUI) - better than Orca |
| **msi-repair** | Self-healing and resiliency configuration helper |
| **wix-guid** | GUID generator for installers |
| **wix-silent** | Silent install parameter generator and validator |
| **wix-msix** | MSIX conversion helper for modern packaging |
| **wix-install** | Script runner for installation |
| **wix-uninstall** | Script runner for clean uninstallation |

### Migration (3 tools)

| Tool | Description |
|------|-------------|
| **wix-migrate** | Migration assistant for v3 to v4 to v5 to v6 |
| **wix-import** | Import from NSIS, InnoSetup, vdproj projects |
| **wix-update** | Script runner for updates/upgrades |

### Localization & Signing (2 tools)

| Tool | Description |
|------|-------------|
| **wix-i18n** | Multi-language wizard with translation templates |
| **wix-sign** | Code signing helper (SignTool, Azure Trusted Signing) |

### Documentation (3 tools)

| Tool | Description |
|------|-------------|
| **wix-help** | Interactive help system for elements, attributes, ICE rules |
| **wix-docs** | Offline element reference and examples |
| **wixkb** | WiX knowledge base (SQLite database) |

### Analysis (4 tools)

| Tool | Description |
|------|-------------|
| **wix-analytics** | Installation success/failure tracking and reporting |
| **wix-repl** | Interactive shell for WXS exploration and testing |
| **ice-validator** | ICE rule validation engine |
| **project-map** | Project structure mapping |

## Features

### Linting

```bash
winter Product.wxs
```

Output:
```
Product.wxs:15:5 warning[WIX002] Component 'MainApp' has no KeyPath defined
Product.wxs:23:9 error[WIX001] Component 'Config' is missing Guid attribute

Found 1 error, 1 warning
```

49+ rules across categories:
- Component rules (GUIDs, KeyPath, file counts)
- Directory rules (structure, naming)
- Feature rules (references, nesting)
- Property rules (naming, secure properties)
- Custom action rules (scheduling, impersonation)
- Service rules (dependencies, recovery)
- Registry rules (key paths, hive usage)
- Package rules (version, upgrade codes)
- Bundle rules (bootstrapper configuration)
- File rules (extensions, paths)

### Formatting

```bash
wix-fmt --write Product.wxs
```

Options:
- Configurable indentation (spaces/tabs)
- Attribute sorting (Id first, then alphabetical)
- Element ordering by WiX conventions
- Multi-line attributes when threshold exceeded

### AI Code Generation

```bash
wix-ai generate "create installer for MyApp with start menu shortcut"
wix-ai template basic_installer --var NAME=MyApp
wix-ai suggest "<Component"
```

### MSI Exploration

```bash
msi-explorer tables Product.msi
msi-explorer query Product.msi "SELECT * FROM Property"
msi-explorer extract Product.msi --output ./extracted/
msi-explorer compare v1.msi v2.msi
```

### Windows Compatibility

```bash
# Check for Windows Server 2025 / Win11 24H2 issues
wix-compat analyze Product.wxs

# WDAC enterprise compatibility
wix-wdac analyze Product.wxs
wix-wdac generate-rules Product.wxs

# ARM64 multi-platform builds
wix-arm64 analyze Product.wxs
wix-arm64 generate --platforms x64,arm64
```

### Simplified Authoring

```bash
# Generate example config
wix-simple init > myapp.json

# Generate WiX from config
wix-simple generate myapp.json

# Quick one-liner
wix-simple quick --name MyApp --file app.exe --shortcut
```

### Intune Deployment

```bash
wix-intune generate Product.wxs --msi MyApp.msi --output ./intune/
```

Generates:
- Install/uninstall PowerShell scripts
- Detection script
- Manifest JSON
- IntuneWin packaging instructions

## Output Formats

All CLI tools support multiple formats:

```bash
winter --format text Product.wxs    # Human-readable (default)
winter --format json Product.wxs    # Machine-readable
winter --format sarif Product.wxs   # GitHub Actions / IDE integration
```

## Configuration

Create `.wixlintrc.json` in your project root:

```json
{
  "version": "5",
  "rules": {
    "WIX003": "off",
    "WIX015": "warn"
  },
  "format": {
    "indent_size": 2,
    "indent_style": "spaces"
  }
}
```

## Editor Integration

### VS Code

The `wix-lsp` server provides:
- Syntax highlighting
- IntelliSense autocomplete
- Error diagnostics
- Hover documentation
- Go to Definition
- Find References
- Formatting
- Code snippets

### Sublime Text

LSP integration via `wix-sublime` package.

### Other Editors

Any editor with LSP support can use `wix-lsp`.

## Cross-Platform Support

| Tool | Windows | Mac | Linux |
|------|---------|-----|-------|
| winter, wix-fmt, wix-analyzer | Yes | Yes | Yes |
| wix-lsp, wix-ai, wix-simple | Yes | Yes | Yes |
| msi-explorer (read-only) | Yes | Yes | Yes |
| wix-msi (build MSI) | Yes | Yes | Yes |
| wix-test (install testing) | Yes | No | No |
| wix-sign (code signing) | Yes | No | No |

## Data Layer

WixCraft includes a comprehensive WiX knowledge base:

- 50+ element definitions with attributes and relationships
- 49+ lint rules across 10 categories
- ICE rule documentation with fix suggestions
- Standard directories and properties
- Error code explanations
- Code snippets and templates

The data lives in `common/wixkb` (SQLite) and is used by all tools.

## Project Structure

```
wixcraft/
├── authoring/       # wix-init, wix-harvest, wix-patch, wix-ui, wix-prereq,
│                    # wix-easy, wix-license, wix-env, wix-simple
├── build/           # wix-build, wix-bundle, wix-ci, wix-intune, wix-arm64, wix-ext
├── common/          # wixkb, wix-data, ice-validator
├── core/            # wixcraft, wix-msi, code-detector, project-map, schema-loader
├── debug/           # wix-doctor, wix-diff, wix-test, wix-repl, wix-ca-debug
├── docs/            # Documentation, wix-docs, wix-help
├── editor/          # wix-lsp, wix-vscode, wix-sublime, wix-syntax, wix-snippets,
│                    # wix-ai, wintellisense, wix-hover, wix-symbols, wix-references
├── localization/    # wix-i18n
├── migration/       # wix-import
├── quality/         # winter, wix-analyzer, wix-fmt, wix-wdac, wix-compat,
│                    # wix-security, wix-upgrade
├── signing/         # wix-sign
└── tools/           # msi-explorer, msi-repair, wix-guid, wix-silent, wix-msix,
                     # wix-install, wix-uninstall, wix-update, wix-migrate,
                     # wix-analytics, wix-preview
```

## Building

Each tool is a standalone Rust crate:

```bash
# Build a tool
cd quality/winter
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy

# Build all tools in a category
for dir in quality/*/; do
  (cd "$dir" && cargo build --release)
done
```

## Pain Points Addressed

WixCraft addresses 46 documented WiX developer pain points:

| Category | Pain Points | Tools |
|----------|-------------|-------|
| Core Engine | 3 | wixcraft, wix-msi, wix-simple |
| IDE & Editor | 6 | wix-lsp, wix-vscode, wix-sublime, wix-syntax, wix-snippets, wix-ai |
| Code Quality | 4 | winter, wix-fmt, schema-loader, wix-wdac |
| Debugging & Testing | 5 | wix-doctor, wix-test, wix-preview, wix-diff, wix-ca-debug |
| Authoring Helpers | 8 | wix-init, wix-harvest, wix-ui, wix-prereq, wix-docs, wix-easy, wix-license, wix-env |
| Localization & Signing | 2 | wix-sign, wix-i18n |
| Migration & Import | 2 | wix-migrate, wix-import |
| Patching & Updates | 1 | wix-patch |
| Runtime & Terminal | 4 | wix-repl, wix-install, wix-update, wix-uninstall |
| Build & CI/CD | 4 | wix-bundle, wix-build, wix-ci, wix-intune |
| Analytics & Advanced | 5 | wix-analytics, msi-repair, wix-silent, wix-msix, wix-ext |
| Windows Compatibility | 2 | wix-compat, wix-arm64 |

See [docs/pain-points.md](docs/pain-points.md) for the complete analysis.

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Run `cargo test` and `cargo clippy`
5. Submit a pull request

## License

MIT License. See [LICENSE](LICENSE) for details.

## Links

- [WiX Toolset](https://wixtoolset.org/) - Official WiX documentation
- [Windows Installer](https://docs.microsoft.com/windows/win32/msi/) - Microsoft documentation
- [Pain Points Analysis](docs/pain-points.md) - Problems WixCraft solves
