# wix-data

Single source of truth for all WixCraft tools.

## Structure

```
wix-data/
├── index.json           # Master index and metadata
├── elements/            # WiX element definitions
│   ├── package.json
│   ├── component.json
│   ├── file.json
│   ├── directory.json
│   ├── feature.json
│   └── ...
├── attributes/          # Shared attribute type definitions
│   └── types.json
├── keywords/            # Reserved keywords for syntax highlighting
│   └── keywords.json
├── rules/               # Lint rule definitions
│   └── component-rules.json
├── snippets/            # Code snippet templates
│   └── snippets.json
├── versions/            # Version migration diffs
│   └── v3-v4-diff.json
├── msi-tables/          # MSI database table mappings
│   └── (TODO)
└── errors/              # Error codes and messages
    └── wix-errors.json
```

## Usage

This data is consumed by:

| Tool | Uses |
|------|------|
| wix-syntax | elements, keywords → syntax highlighting |
| wix-lsp | elements, attributes → autocomplete, hover |
| wix-lint | rules → validation |
| wix-fmt | elements → formatting order |
| wix-snippets | snippets → code templates |
| wix-migrate | versions → migration |
| wix-doctor | errors → error explanations |

## Element Schema

Each element definition includes:

```json
{
  "name": "ElementName",
  "namespace": "wix",
  "since": "v3",
  "description": "...",
  "documentation": "https://...",
  "parents": ["AllowedParent1", "AllowedParent2"],
  "children": ["AllowedChild1", "AllowedChild2"],
  "attributes": {
    "AttrName": {
      "type": "string|guid|yesno|integer|enum|identifier|path|filename|version",
      "required": true|false,
      "default": "value",
      "description": "..."
    }
  },
  "msiTables": ["Table1", "Table2"],
  "rules": ["rule-id-1", "rule-id-2"],
  "examples": [...]
}
```

## Contributing

1. Element definitions are extracted from WiX XSD schemas
2. Rules are manually curated based on best practices
3. Error messages are extracted from WiX source and Microsoft docs

## Sources

- [WiX XSD Schema](https://github.com/wixtoolset/web/blob/master/src/xsd4/wix.xsd)
- [MSI Database Tables](https://learn.microsoft.com/en-us/windows/win32/msi/database-tables)
- [ICE Validation](https://learn.microsoft.com/en-us/windows/win32/msi/internal-consistency-evaluators-ices)
