# WiX Toolset Developer Pain Points & Tools

## Core Engine

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| XML syntax is verbose and unintuitive | `wixcraft` | Ansible-like YAML-to-MSI engine. Write simple YAML, get WXS/MSI output |
| MSI building requires Windows | `wix-msi` | Cross-platform MSI compiler. Build MSI on Mac/Linux without Windows |

## IDE & Editor Support

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| No LSP/autocomplete outside Visual Studio | `wix-lsp` | Language Server Protocol for WXS files (autocomplete, validation, hover docs, go-to-definition) |
| No VS Code support | `wix-vscode` | VS Code extension powered by wix-lsp |
| No Sublime Text support | `wix-sublime` | Sublime Text package powered by wix-lsp |
| No syntax highlighting | `wix-syntax` | Syntax highlighting for VS Code (.tmLanguage) and Sublime Text (.sublime-syntax) |
| No code snippets | `wix-snippets` | Snippet library for common WXS patterns |

## Code Quality

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| No linter | `wix-lint` | Static analysis to catch errors before build |
| No formatter | `wix-fmt` | Auto-format WXS files with consistent style |
| No schema validation | `wix-schema` | XSD validation and type checking |

## Debugging & Testing

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| Cryptic errors / debugging is hard | `wix-doctor` | Error decoder and log analyzer with suggested fixes |
| No installer testing | `wix-test` | Validate install/update/uninstall in sandbox |
| No live preview | `wix-preview` | Preview installer UI and file layout without building |
| No way to compare MSI versions | `wix-diff` | Compare MSI versions, show what changed between builds |
| Custom action testing is hard | `wix-lux` | Enhanced custom action testing framework |

## Authoring Helpers

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| No templates or patterns | `wix-init` | Project scaffolder with common scenarios |
| Heat deprecated / harvesting issues | `wix-harvest` | Modern file harvester with CI/CD support |
| UI customization is hard | `wix-ui` | Visual dialog generator and editor |
| Prerequisites are complex | `wix-prereq` | Prerequisites detection and bundle helper |
| No offline documentation | `wix-docs` | Offline element reference and examples |
| No step-by-step wizard | `wix-wizard` | Interactive GUI wizard for creating projects (like InnoSetup) |
| License validation is manual | `wix-license` | License key/serial validation wizard |
| Environment variables are tricky | `wix-env` | Environment variable and PATH configuration helper |

## Localization & Signing

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| Code signing is manual | `wix-sign` | Code signing helper (SignTool, Azure Trusted Signing integration) |
| Localization is tedious | `wix-i18n` | Multi-language wizard with translation templates |

## Migration & Import

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| Breaking changes between WiX versions | `wix-migrate` | Migration assistant for v3→v4→v5→v6 upgrades |
| No import from other tools | `wix-import` | Import from NSIS, InnoSetup, vdproj projects |

## Patching & Updates

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| MSP patches are complex to author | `wix-patch` | Simplified patch/MSP generation helper |

## Runtime & Terminal

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| No terminal integration | `wix-repl` | Interactive shell for WXS exploration and testing |
| No install script | `wix-install` | Script runner for installation |
| No update script | `wix-update` | Script runner for updates/upgrades |
| No uninstall script | `wix-uninstall` | Script runner for clean uninstallation |

## Build & CI/CD Integration

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| Bootstrapper complexity | `wix-bundle` | Wizard and templates for Burn bootstrappers |
| No unified build CLI | `wix-build` | Unified build CLI wrapping WiX toolset |
| CI/CD setup is painful | `wix-ci` | Templates for GitHub Actions, Azure DevOps, GitLab CI |

## Analytics & Advanced Features

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| No installation telemetry | `wix-analytics` | Installation success/failure tracking and reporting |
| Self-healing config is complex | `wix-repair` | Self-healing and resiliency configuration helper |
| Silent install params undocumented | `wix-silent` | Silent install parameter generator and validator |
| No MSIX conversion path | `wix-msix` | MSIX conversion helper for modern Windows packaging |
| Dependency management missing | `wix-deps` | Dependency detection and resolution |

---

## Summary

| Category | Count | Tools |
|----------|-------|-------|
| Core Engine | 2 | wixcraft, wix-msi |
| IDE & Editor | 5 | wix-lsp, wix-vscode, wix-sublime, wix-syntax, wix-snippets |
| Code Quality | 3 | wix-lint, wix-fmt, wix-schema |
| Debugging & Testing | 5 | wix-doctor, wix-test, wix-preview, wix-diff, wix-lux |
| Authoring Helpers | 8 | wix-init, wix-harvest, wix-ui, wix-prereq, wix-docs, wix-wizard, wix-license, wix-env |
| Localization & Signing | 2 | wix-sign, wix-i18n |
| Migration & Import | 2 | wix-migrate, wix-import |
| Patching & Updates | 1 | wix-patch |
| Runtime & Terminal | 4 | wix-repl, wix-install, wix-update, wix-uninstall |
| Build & CI/CD | 3 | wix-bundle, wix-build, wix-ci |
| Analytics & Advanced | 5 | wix-analytics, wix-repair, wix-silent, wix-msix, wix-deps |
| **Total** | **40** | |

---

## Cross-Platform Support Matrix

| Tool | Windows | Mac | Linux | Notes |
|------|---------|-----|-------|-------|
| `wixcraft` | Yes | Yes | Yes | YAML authoring works everywhere |
| `wix-msi` | Yes | Yes | Yes | Native MSI compiler, no Windows needed |
| `wix-lsp` | Yes | Yes | Yes | Editor support everywhere |
| `wix-lint` | Yes | Yes | Yes | Static analysis |
| `wix-fmt` | Yes | Yes | Yes | Formatting |
| `wix-doctor` | Yes | Partial | Partial | Log analysis works, some features need Windows |
| `wix-test` | Yes | No | No | Requires Windows Installer |
| `wix-preview` | Yes | Yes | Yes | Simulated preview |
| `wix-sign` | Yes | No | No | Code signing requires Windows |
