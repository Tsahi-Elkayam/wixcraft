# Getting Started with WixCraft

This guide walks you through installing and using WixCraft tools for WiX development.

## Prerequisites

- Rust 1.70+ (for building from source)
- WiX Toolset v4+ (for building MSI packages)
- A WiX project to work with

## Installation

### Option 1: Build from Source

```bash
# Clone the repository
git clone https://github.com/wixcraft/wixcraft
cd wixcraft

# Build the linter
cd quality/winter
cargo build --release

# The binary is at target/release/winter
# Copy it to your PATH or use directly
```

### Option 2: Pre-built Binaries

Download from the [Releases](https://github.com/wixcraft/wixcraft/releases) page.

## Your First Lint

Create a simple WiX file or use an existing one:

```xml
<!-- Product.wxs -->
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="MyApp" Version="1.0.0" Manufacturer="MyCompany">
    <Feature Id="Main">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>
  </Package>

  <Fragment>
    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="MainExe">
        <File Source="MyApp.exe" />
      </Component>
    </ComponentGroup>
  </Fragment>
</Wix>
```

Run the linter:

```bash
winter Product.wxs
```

Output:
```
Product.wxs:11:7 warning[WIX002] Component 'MainExe' has no KeyPath defined
  --> Add KeyPath="yes" to the primary File element

Product.wxs:11:7 warning[WIX001] Component 'MainExe' is missing Guid attribute
  --> Add Guid="*" for auto-generated GUID

Found 0 errors, 2 warnings
```

## Fixing Issues

Update your file based on the suggestions:

```xml
<Component Id="MainExe" Guid="*">
  <File Source="MyApp.exe" KeyPath="yes" />
</Component>
```

Run again:

```bash
winter Product.wxs
```

Output:
```
No issues found
```

## Formatting

Format your WiX files for consistent style:

```bash
# Check formatting (dry run)
wix-fmt --check Product.wxs

# Format in place
wix-fmt --write Product.wxs

# Format to stdout
wix-fmt Product.wxs
```

Configuration via `.wixfmtrc.json`:

```json
{
  "indent_size": 2,
  "indent_style": "spaces",
  "attribute_sort": true,
  "attribute_per_line": 3
}
```

## Exploring MSI Files

Inspect a compiled MSI:

```bash
# List all tables
msi-explorer tables Product.msi

# View a specific table
msi-explorer table Product.msi Property

# Query with SQL
msi-explorer query Product.msi "SELECT * FROM File WHERE FileName LIKE '%.exe'"

# Extract all files
msi-explorer extract Product.msi --output ./extracted/

# Compare two MSIs
msi-explorer diff v1.msi v2.msi
```

## Getting Help

Look up any WiX element, attribute, or error:

```bash
# Element documentation
wix-help Component
wix-help File

# Attribute documentation
wix-help Component.Guid
wix-help File.KeyPath

# ICE validation errors
wix-help ICE08
wix-help ICE57

# MSI error codes
wix-help error 1603

# Properties
wix-help property ALLUSERS

# Standard directories
wix-help directory ProgramFilesFolder
```

## Project Analysis

Run comprehensive analysis on your project:

```bash
# Full analysis
wix-analyzer ./src/installer/

# Security scan
wix-analyzer --security Product.wxs

# Best practices
wix-analyzer --practices Product.wxs

# Dependency analysis
wix-analyzer deps Product.wxs
```

## Migrating WiX Versions

Convert between WiX versions:

```bash
# Preview changes (dry run)
wix-migrate --from v3 --to v5 --dry-run ./src/installer/

# Generate migration report
wix-migrate --report ./src/installer/

# Apply migration
wix-migrate --from v3 --to v5 --write ./src/installer/
```

## Generating GUIDs

Generate GUIDs for your installers:

```bash
# Random GUID
wix-guid random

# Deterministic GUID from string
wix-guid hash "MyApp.MainComponent"

# Multiple GUIDs
wix-guid random --count 5

# Different formats
wix-guid random --format registry   # {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}
wix-guid random --format plain      # XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

## Editor Integration

### VS Code (Coming Soon)

The `wix-lsp` language server provides:
- Real-time error detection
- Autocomplete for elements and attributes
- Hover documentation
- Go to Definition
- Find References
- Formatting

### Manual LSP Setup

For editors with LSP support:

```bash
# Build the LSP server
cd editor/wix-lsp
cargo build --release

# Point your editor to target/release/wix-lsp
```

## CI/CD Integration

### GitHub Actions

```yaml
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install WixCraft
        run: |
          curl -L https://github.com/wixcraft/wixcraft/releases/download/v0.1.0/winter-linux -o winter
          chmod +x winter
      - name: Lint
        run: ./winter --format sarif src/installer/*.wxs > results.sarif
      - uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: results.sarif
```

### Azure DevOps

```yaml
steps:
  - script: |
      curl -L https://github.com/wixcraft/wixcraft/releases/download/v0.1.0/winter-linux -o winter
      chmod +x winter
      ./winter src/installer/*.wxs
    displayName: 'Lint WiX files'
```

## Configuration Reference

### .wixlintrc.json

```json
{
  "version": "5",
  "rules": {
    "WIX001": "error",
    "WIX002": "warning",
    "WIX003": "off"
  },
  "exclude": [
    "generated/*.wxs",
    "third-party/**"
  ]
}
```

### Rule Severity Levels

- `error` - Fail the build
- `warning` - Report but don't fail
- `info` - Informational only
- `off` - Disable the rule

## Next Steps

- Browse the [full tools list](tools-list.md)
- Read about [WiX pain points](msi-pain-points-and-wix-gaps.md) we address
- Check the [architecture](architecture.md) for understanding the design
- Explore the [data sources](data-sources.md) powering the tools

## Troubleshooting

### "Command not found"

Add the tool to your PATH or use the full path:

```bash
export PATH="$PATH:/path/to/wixcraft/target/release"
```

### "Database not found" errors

Some tools need the wixkb database. Build it first:

```bash
cd common/wixkb
cargo build --release
./target/release/database init
```

### Build errors

Ensure you have Rust 1.70+:

```bash
rustc --version
rustup update stable
```
