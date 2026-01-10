# WixCraft - Project Scope

## Mission

Make WiX development as pleasant as modern language tooling (Rust Analyzer, TypeScript, etc.)

## In Scope

### WiX Authoring Tools (Primary Focus)
- **Language Server (LSP)** - Powers all editors
- **VS Code Extension** - First-class editor support
- **Sublime Text Extension** - Secondary editor support
- **Linter** - Catch errors before compile
- **Formatter** - Consistent code style
- **Autocomplete** - Elements, attributes, values, variables
- **Hover Documentation** - Instant help on any element
- **Go to Definition / Find References** - Navigate large projects
- **Migration Tool** - v3 to v4 to v5 to v6 conversion
- **Preprocessor Validator** - Catch malformed variables early

### MSI Analysis Tools (Secondary Focus)
- **MSI Inspector** - Explore compiled .msi files (better than Orca)
- **Log Analyzer** - Parse verbose MSI logs, highlight actual errors
- **Security Scanner** - Detect privilege escalation vulnerabilities
- **Upgrade Validator** - Verify component rules before deployment

### Data Layer (Foundation)
- Complete WiX schema database (all versions)
- ICE rule documentation with fix suggestions
- Error code explanations
- Standard directory/property reference

## Out of Scope

- NSIS support
- Inno Setup support
- InstallShield compatibility
- MSIX authoring (maybe future)
- GUI/WYSIWYG editor (not our strength)
- Build system (WiX already does this)

## Target Users

1. **Solo developers** forced to create MSI packages
2. **Small teams** without budget for commercial tools
3. **Enterprise developers** using VS Code instead of Visual Studio
4. **CI/CD pipelines** needing validation and analysis
5. **Anyone migrating** between WiX versions

## Success Metrics

- VS Code extension with 1000+ installs
- Sub-100ms response time for all LSP operations
- Catch 90% of common errors before compile
- Documentation always one hover away
- Works offline, no telemetry, fully open source

## Technology Choices

- **Rust** - CLI tools, LSP server, fast and cross-platform
- **TypeScript** - VS Code extension
- **JSON** - WiX data layer (elements, rules, snippets)
- **SQLite** - Optional caching for large projects

## Non-Goals

- Replace WiX Toolset itself
- Compete with Advanced Installer/InstallShield on features
- Support every edge case of Windows Installer
- Build a GUI - we're CLI/editor-first
