# WiX Toolset Developer Pain Points & Tools

## Core Engine

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| XML syntax is verbose and unintuitive | `wixcraft` | Ansible-like YAML-to-MSI engine. Write simple YAML, get WXS/MSI output |
| MSI building requires Windows | `wix-msi` | Cross-platform MSI compiler. Build MSI on Mac/Linux without Windows |
| Excessive complexity / learning curve | `wix-simple` | Generate complete installers from minimal JSON config |

## IDE & Editor Support

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| No LSP/autocomplete outside Visual Studio | `wix-lsp` | Language Server Protocol for WXS files (autocomplete, validation, hover docs, go-to-definition) |
| No VS Code support | `wix-vscode` | VS Code extension powered by wix-lsp |
| No Sublime Text support | `wix-sublime` | Sublime Text package powered by wix-lsp |
| No syntax highlighting | `wix-syntax` | Syntax highlighting for VS Code (.tmLanguage) and Sublime Text (.sublime-syntax) |
| No code snippets | `wix-snippets` | Snippet library for common WXS patterns |
| No AI assistance | `wix-ai` | AI-powered code generation from natural language prompts |

## Code Quality

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| No linter | `winter` | Static analysis to catch errors before build |
| No formatter | `wix-fmt` | Auto-format WXS files with consistent style |
| No schema validation | `schema-loader` | XSD validation and type checking |
| WDAC policy conflicts | `wix-wdac` | WDAC compatibility checker for enterprise deployments |

## Debugging & Testing

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| Cryptic errors / debugging is hard | `wix-doctor` | Error decoder and log analyzer with suggested fixes |
| No installer testing | `wix-test` | Validate install/update/uninstall in sandbox |
| No live preview | `wix-preview` | Preview installer UI and file layout without building |
| No way to compare MSI versions | `wix-diff` | Compare MSI versions, show what changed between builds |
| Custom action testing is hard | `wix-ca-debug` | Enhanced custom action testing framework |

## Authoring Helpers

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| No templates or patterns | `wix-init` | Project scaffolder with common scenarios |
| Heat deprecated / harvesting issues | `wix-harvest` | Modern file harvester with CI/CD support |
| UI customization is hard | `wix-ui` | Visual dialog generator and editor |
| Prerequisites are complex | `wix-prereq` | Prerequisites detection and bundle helper |
| No offline documentation | `wix-docs` | Offline element reference and examples |
| No step-by-step wizard | `wix-easy` | Interactive GUI wizard for creating projects (like InnoSetup) |
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
| Breaking changes between WiX versions | `wix-migrate` | Migration assistant for v3->v4->v5->v6 upgrades |
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
| Intune packaging missing | `wix-intune` | Generate .intunewin packages for Microsoft Intune deployment |

## Analytics & Advanced Features

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| No installation telemetry | `wix-analytics` | Installation success/failure tracking and reporting |
| Self-healing config is complex | `msi-repair` | Self-healing and resiliency configuration helper |
| Silent install params undocumented | `wix-silent` | Silent install parameter generator and validator |
| No MSIX conversion path | `wix-msix` | MSIX conversion helper for modern Windows packaging |
| Dependency management missing | `wix-ext` | Dependency detection and resolution |

## Windows Compatibility (NEW)

| Pain Point | Tool Name | Description |
|------------|-----------|-------------|
| Windows Server 2025 issues | `wix-compat` | Windows version compatibility checker (MSI hangs, VersionNT64) |
| August 2025 UAC changes | `wix-compat` | Detects per-user MSI issues after KB5063878 |
| ARM64 support gaps | `wix-arm64` | ARM64 multi-platform build helper |
| ARM64 driver installation fails | `wix-arm64` | DifxApp deprecation detection and alternatives |

---

## Summary

| Category | Count | Tools |
|----------|-------|-------|
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
| **Total** | **46** | |

---

## New Pain Points (2025 Research)

These pain points were identified through fresh research in January 2025:

### Windows Server 2025 Issues
- MSI installations hang for ~30 minutes on Domain Controllers
- VersionNT64 detection fails with older packaging tools
- Workaround: Disable conflicting services, use build number checks

### Windows 11 24H2 UAC Changes (August 2025)
- Per-user MSI repairs trigger UAC prompts (Error 1730)
- Affects Autodesk, SAP, Firefox and other applications
- Workaround: Use per-machine install or apply KB5070773

### ARM64 Support Gaps
- DifxApp driver installation not supported on ARM64
- ProcessorArchitecture detection bug in emulation
- Custom action DLLs need ARM64 builds

### WDAC Policy Conflicts
- WiX extension DLLs (wixca.dll) are not code-signed
- VBScript/JScript custom actions blocked by WDAC
- Workaround: Add hash rules to WDAC policy

### Intune Deployment
- No native .intunewin package generation
- Manual process to create Win32 app packages

### AI Assistance Gap
- No copilot/AI code generation for WiX XML
- Unlike modern dev tools

---

## Cross-Platform Support Matrix

| Tool | Windows | Mac | Linux | Notes |
|------|---------|-----|-------|-------|
| `wixcraft` | Yes | Yes | Yes | YAML authoring works everywhere |
| `wix-msi` | Yes | Yes | Yes | Native MSI compiler, no Windows needed |
| `wix-lsp` | Yes | Yes | Yes | Editor support everywhere |
| `winter` | Yes | Yes | Yes | Static analysis |
| `wix-fmt` | Yes | Yes | Yes | Formatting |
| `wix-doctor` | Yes | Partial | Partial | Log analysis works, some features need Windows |
| `wix-test` | Yes | No | No | Requires Windows Installer |
| `wix-preview` | Yes | Yes | Yes | Simulated preview |
| `wix-sign` | Yes | No | No | Code signing requires Windows |
| `wix-wdac` | Yes | Yes | Yes | Analysis only |
| `wix-intune` | Yes | Yes | Yes | Package generation |
| `wix-compat` | Yes | Yes | Yes | Compatibility checking |
| `wix-arm64` | Yes | Yes | Yes | Multi-platform config |
| `wix-ai` | Yes | Yes | Yes | Code generation |
| `wix-simple` | Yes | Yes | Yes | Simple installer generation |
