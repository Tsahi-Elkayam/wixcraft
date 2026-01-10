# WixCraft Architecture

## Design Philosophy

**Build once, use many.**

- **Engine**: Generic, language-agnostic, built once
- **Plugin**: Language-specific data in YAML, built once per language

---

## Language Types Analysis

The engine must support different language categories. Each type has different parsing and tooling requirements.

### Language Categories

| Type | Examples | Parser | AST | Complexity |
|------|----------|--------|-----|------------|
| **Structured Data (XML)** | WiX, Maven, XAML, Spring, XSLT | XML parser | Tree | Medium |
| **Structured Data (YAML)** | Ansible, Docker Compose, K8s, GitHub Actions | YAML parser | Tree | Low |
| **Structured Data (JSON)** | package.json, tsconfig, eslintrc | JSON parser | Tree | Low |
| **Config Files (INI)** | .ini, .conf, .properties, .env | Line parser | Flat | Low |
| **Domain-Specific (DSL)** | Dockerfile, Terraform HCL, Makefile | Custom lexer | Semi-structured | Medium |
| **Scripting Languages** | PowerShell, Bash, Batch, Python | Full parser | Full AST + Scopes | High |
| **Compiled Languages** | Go, Rust, C#, Java | Full parser | Full AST + Types | Very High |

### Complexity Spectrum

```
Simple ◄───────────────────────────────────────────────────────► Complex

  INI       JSON      YAML       XML        DSL      Scripting   Compiled
   │          │         │          │          │           │           │
   ▼          ▼         ▼          ▼          ▼           ▼           ▼
 Lines      Tree      Tree       Tree     Custom      Full AST    Full AST
 Only      Simple    Simple    + Schema    Lexer     + Scopes    + Types
                               + Refs                + Closures  + Generics
```

### Engine Support Decision

| Type | Build in Engine? | Rationale |
|------|------------------|-----------|
| **XML** | YES | WiX is XML - primary focus |
| **YAML** | YES | Plugin format, same tree structure |
| **JSON** | YES | Trivial addition, config files |
| **INI** | YES | Simple, useful for .wixlintrc |
| **DSL** | MAYBE (future) | Requires pluggable lexer architecture |
| **Scripting** | NO - integrate | Mature LSPs exist (PowerShell, Python) |
| **Compiled** | NO - integrate | Mature LSPs exist (gopls, rust-analyzer) |

### Tool Capability by Language Type

| Tool | XML | YAML | JSON | INI | DSL | Script |
|------|-----|------|------|-----|-----|--------|
| Syntax Highlighter | Full | Full | Full | Full | Full | Full |
| Linter | Full | Full | Full | Full | Full | Full |
| Formatter | Full | Full | Full | Basic | Partial | Partial |
| Autocomplete | Full | Full | Full | Basic | Partial | Full |
| Hover Docs | Full | Full | Full | Basic | Partial | Full |
| Go to Definition | Full | Full | Full | N/A | Partial | Full |
| Find References | Full | Full | Full | N/A | Partial | Full |
| Refactor/Rename | Full | Full | Full | N/A | Partial | Full |
| Schema Validation | Full | Full | Full | N/A | N/A | N/A |
| Best Practices | Full | Full | Full | Basic | Full | Full |

### WixCraft Scope

```
┌─────────────────────────────────────────────────────────────┐
│                    Engine Parsers                            │
├─────────────┬─────────────┬─────────────┬───────────────────┤
│     XML     │    YAML     │    JSON     │       INI         │
│  (primary)  │  (plugins)  │  (config)   │    (config)       │
├─────────────┴─────────────┴─────────────┴───────────────────┤
│                     v1.0 Scope                               │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│              Future: Pluggable Lexers                        │
├─────────────┬─────────────┬─────────────────────────────────┤
│ Dockerfile  │  Terraform  │         Other DSLs              │
├─────────────┴─────────────┴─────────────────────────────────┤
│                   v2.0+ Scope (maybe)                        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│              Out of Scope: Use Existing LSPs                 │
├─────────────┬─────────────┬─────────────┬───────────────────┤
│ PowerShell  │   Python    │     Go      │    Rust/C#/etc    │
└─────────────┴─────────────┴─────────────┴───────────────────┘
```

```
Engine (Rust)  ─────►  Any language tooling
     +
Plugin (YAML)  ─────►  WiX-specific knowledge
     =
WixCraft       ─────►  Complete WiX tooling
```

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                        User Interface                         │
├──────────┬──────────┬──────────┬──────────┬──────────────────┤
│  CLI     │  VS Code │ Sublime  │  Neovim  │  Other Editors   │
└────┬─────┴─────┬────┴─────┬────┴─────┬────┴────────┬─────────┘
     │           │          │          │             │
     │           └──────────┴──────────┴─────────────┘
     │                            │
     │                      LSP Protocol
     │                            │
     ▼                            ▼
┌──────────────────────────────────────────────────────────────┐
│                         TOOLS LAYER                          │
├──────────┬──────────┬──────────┬──────────┬─────────────────┤
│  Linter  │ Formatter│   LSP    │  Help    │  Generator      │
│          │          │  Server  │  Browser │                 │
└────┬─────┴─────┬────┴─────┬────┴─────┬────┴────────┬────────┘
     │           │          │          │             │
     └───────────┴──────────┴──────────┴─────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│                      GENERIC ENGINE                          │
│                     (language-agnostic)                      │
├──────────────────────────────────────────────────────────────┤
│  • File Parser (XML, text, etc.)                             │
│  • AST Builder                                               │
│  • Rule Evaluator                                            │
│  • Symbol Indexer                                            │
│  • Formatter Engine                                          │
│  • Autocomplete Engine                                       │
│  • Reference Resolver                                        │
│  • Output Formatters (text, json, sarif)                     │
│  • LSP Protocol Handler                                      │
│  • Plugin Loader                                             │
└──────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────┐
│                      PLUGIN MANAGER                          │
├──────────────────────────────────────────────────────────────┤
│  • Discovers plugins                                         │
│  • Loads YAML definitions                                    │
│  • Validates plugin schema                                   │
│  • Provides plugin API to engine                             │
└──────────────────────────────────────────────────────────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
              ▼             ▼             ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │   WiX    │  │  Future  │  │  Future  │
        │  Plugin  │  │  Lang A  │  │  Lang B  │
        │  (YAML)  │  │  Plugin  │  │  Plugin  │
        └──────────┘  └──────────┘  └──────────┘
```

---

## Component Responsibilities

### Generic Engine (build once)

| Component | Responsibility |
|-----------|----------------|
| **File Parser** | Parse XML/text into AST |
| **AST Builder** | Create traversable tree structure |
| **Rule Evaluator** | Execute rules from plugin against AST |
| **Symbol Indexer** | Build index of definitions/references |
| **Formatter Engine** | Apply formatting rules from plugin |
| **Autocomplete Engine** | Generate suggestions from plugin data |
| **Reference Resolver** | Find definition/references using plugin |
| **Output Formatters** | Render results as text/json/sarif |
| **LSP Handler** | Implement LSP protocol |
| **Plugin Loader** | Load and validate plugin YAML |

### Plugin (build once per language)

| Component | Responsibility |
|-----------|----------------|
| **plugin.yaml** | Plugin metadata, file extensions |
| **elements/*.yaml** | Syntax elements, attributes, hierarchy |
| **rules/*.yaml** | Lint rules, best practices |
| **snippets/*.yaml** | Code templates |
| **formatting.yaml** | Formatting preferences |
| **docs/*.yaml** | Extended documentation |
| **errors/*.yaml** | Error code explanations |

---

## Plugin Structure (Ansible-style YAML)

```
wix-plugin/
├── plugin.yaml              # Plugin metadata
├── elements/                # Element definitions
│   ├── component.yaml
│   ├── file.yaml
│   ├── directory.yaml
│   └── ...
├── rules/                   # Lint rules
│   ├── component-rules.yaml
│   ├── file-rules.yaml
│   ├── security-rules.yaml
│   └── ...
├── snippets/                # Code snippets
│   └── snippets.yaml
├── formatting/              # Formatter config
│   └── formatting.yaml
├── msi/                     # MSI-specific data
│   ├── properties.yaml
│   ├── directories.yaml
│   ├── tables.yaml
│   └── actions.yaml
├── errors/                  # Error explanations
│   ├── wix-errors.yaml
│   ├── ice-errors.yaml
│   └── msi-errors.yaml
└── docs/                    # Extended docs
    └── best-practices.yaml
```

---

## YAML Schema Examples

### plugin.yaml (metadata)

```yaml
name: wix
display_name: WiX Toolset
description: Windows Installer XML tooling
version: 1.0.0

file_types:
  - extension: .wxs
    type: xml
    description: WiX source file
  - extension: .wxi
    type: xml
    description: WiX include file
  - extension: .wxl
    type: xml
    description: WiX localization file

xml_namespaces:
  - uri: http://wixtoolset.org/schemas/v4/wxs
    prefix: wix
    version: "4"
  - uri: http://schemas.microsoft.com/wix/2006/wi
    prefix: wix
    version: "3"

documentation:
  base_url: https://wixtoolset.org/docs/
  schema_url: https://wixtoolset.org/docs/schema/wxs/
```

### elements/component.yaml

```yaml
name: Component
description: >
  A component is the atomic unit of installation.
  Each component should contain one "key" resource.

category: core
documentation: https://wixtoolset.org/docs/schema/wxs/component/

# Parent-child relationships
parents:
  - Directory
  - DirectoryRef
  - ComponentGroup
  - Fragment

children:
  - File
  - RegistryKey
  - RegistryValue
  - Shortcut
  - ServiceInstall
  - ServiceControl
  - Environment
  - CreateFolder
  - RemoveFile
  - RemoveFolder

# Attribute definitions
attributes:
  - name: Id
    type: identifier
    required: true
    description: Unique identifier for this component

  - name: Guid
    type: guid
    required: false
    default: "*"
    description: >
      Component GUID for Windows Installer tracking.
      Use "*" to auto-generate at build time.

  - name: Directory
    type: reference
    references: Directory
    required: false
    description: >
      Directory to install into. Only needed if Component
      is not nested under a Directory element.

  - name: Win64
    type: yesno
    required: false
    default: "no"
    description: Set to "yes" for 64-bit components

  - name: KeyPath
    type: yesno
    required: false
    description: >
      Deprecated. Set KeyPath on child element instead.
    deprecated: true
    deprecated_message: Use KeyPath attribute on File or RegistryKey child

# MSI mapping (for advanced users)
msi_table: Component
msi_columns:
  Id: Component
  Guid: ComponentId
  Directory: Directory_

# Examples
examples:
  - name: Basic file component
    description: Simple component with one file
    code: |
      <Component Id="MyApp_exe" Guid="*">
        <File Source="MyApp.exe" KeyPath="yes" />
      </Component>

  - name: Component with registry
    description: Component with file and registry key
    code: |
      <Component Id="MyApp_reg" Guid="*">
        <File Source="MyApp.exe" KeyPath="yes" />
        <RegistryKey Root="HKCU" Key="Software\MyCompany\MyApp">
          <RegistryValue Name="Version" Type="string" Value="1.0" />
        </RegistryKey>
      </Component>

# Version info
version:
  introduced: "3.0"
  changed: "4.0"
  change_notes: "Guid='*' now recommended over explicit GUIDs"
  deprecated: null
```

### rules/component-rules.yaml

```yaml
category: component
description: Rules for Component element validation

rules:
  - id: WIX001
    name: component-missing-guid
    severity: error
    message: "Component '{id}' is missing Guid attribute"

    description: >
      Every Component must have a Guid for Windows Installer
      to track installation state. Use Guid="*" to auto-generate.

    applies_to: Component
    condition:
      type: missing_attribute
      attribute: Guid

    fix:
      type: add_attribute
      attribute: Guid
      value: "*"

    documentation: https://wixtoolset.org/docs/schema/wxs/component/

  - id: WIX002
    name: component-no-keypath
    severity: warning
    message: "Component '{id}' has no KeyPath defined"

    description: >
      Components should have a KeyPath to help Windows Installer
      determine if the component is installed. Set KeyPath="yes"
      on one child element.

    applies_to: Component
    condition:
      type: expression
      expr: "not(.//*[@KeyPath='yes'])"

    fix:
      type: suggest
      message: Add KeyPath="yes" to the primary File or RegistryKey

  - id: WIX003
    name: component-multiple-files
    severity: warning
    message: "Component '{id}' contains {file_count} files"

    description: >
      Best practice is one file per component. Multiple files
      in a component share the same install/uninstall state.

    applies_to: Component
    condition:
      type: count_children
      element: File
      operator: ">"
      value: 1

    exceptions:
      - "Satellite assemblies"
      - "Related config files"

  - id: WIX004
    name: component-empty
    severity: error
    message: "Component '{id}' has no child elements"

    applies_to: Component
    condition:
      type: no_children
```

### snippets/snippets.yaml

```yaml
snippets:
  - name: component-file
    prefix: comp
    description: Component with single file
    scope: Directory, DirectoryRef
    body: |
      <Component Id="${1:ComponentId}" Guid="*">
        <File Source="${2:SourcePath}" KeyPath="yes" />
      </Component>

  - name: component-service
    prefix: compsvc
    description: Component with Windows service
    scope: Directory, DirectoryRef
    body: |
      <Component Id="${1:ServiceName}_comp" Guid="*">
        <File Source="${2:ServiceExe}" KeyPath="yes" />
        <ServiceInstall
          Id="${1:ServiceName}_svc"
          Name="${1:ServiceName}"
          DisplayName="${3:Display Name}"
          Start="auto"
          Type="ownProcess"
          ErrorControl="normal" />
        <ServiceControl
          Id="${1:ServiceName}_ctrl"
          Name="${1:ServiceName}"
          Start="install"
          Stop="both"
          Remove="uninstall"
          Wait="yes" />
      </Component>

  - name: directory-programfiles
    prefix: dirpf
    description: Standard Program Files directory structure
    scope: Fragment, Product, Package
    body: |
      <StandardDirectory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="${1:CompanyName}">
          <Directory Id="${2:ProductFolder}" Name="${3:ProductName}">
            $0
          </Directory>
        </Directory>
      </StandardDirectory>
```

---

## Configuration Files

### tool_settings.yaml (user config)

```yaml
# User's tool configuration (per-project or global)

plugin: wix
version: "4"  # WiX version to target

linter:
  enabled: true
  severity_threshold: warning  # error, warning, info
  disabled_rules:
    - WIX003  # Allow multiple files per component

formatter:
  indent_style: spaces
  indent_size: 2
  attribute_sort: true
  attribute_per_line_threshold: 3

autocomplete:
  show_deprecated: false
  include_snippets: true

output:
  format: text  # text, json, sarif
  color: auto
```

---

## Engine Interfaces

### Plugin API (what engine expects from plugin)

```rust
trait LanguagePlugin {
    fn name(&self) -> &str;
    fn file_extensions(&self) -> Vec<&str>;
    fn get_element(&self, name: &str) -> Option<Element>;
    fn get_elements(&self) -> Vec<Element>;
    fn get_rules(&self) -> Vec<Rule>;
    fn get_snippets(&self) -> Vec<Snippet>;
    fn get_formatting_rules(&self) -> FormattingRules;
}
```

### Tool API (what tools use from engine)

```rust
trait LintEngine {
    fn lint(&self, file: &Path) -> Vec<Diagnostic>;
    fn lint_with_config(&self, file: &Path, config: &Config) -> Vec<Diagnostic>;
}

trait FormatEngine {
    fn format(&self, content: &str) -> String;
    fn check(&self, content: &str) -> bool;
}

trait CompletionEngine {
    fn complete(&self, file: &Path, position: Position) -> Vec<Completion>;
}

trait HoverEngine {
    fn hover(&self, file: &Path, position: Position) -> Option<HoverInfo>;
}

trait ReferenceEngine {
    fn find_definition(&self, file: &Path, position: Position) -> Option<Location>;
    fn find_references(&self, file: &Path, position: Position) -> Vec<Location>;
}
```

---

## Benefits

| Benefit | Description |
|---------|-------------|
| **Separation of concerns** | Engine logic vs language data |
| **Easy updates** | Update YAML without recompiling |
| **Community contributions** | Non-programmers can add rules/docs |
| **Testable** | Test engine once, test plugin data separately |
| **Extensible** | New language = new plugin folder |
| **Consistent UX** | Same engine = same behavior across languages |
| **Version controlled** | YAML diffs cleanly in git |

---

## Future Languages

Same engine could support:

```
plugins/
├── wix/           # WiX Toolset (our focus)
├── msi/           # Raw MSI editing (future)
├── nsis/          # NSIS scripts (future)
├── innosetup/     # Inno Setup (future)
└── msix/          # MSIX manifests (future)
```

But for WixCraft, **WiX is the only plugin we build**.
