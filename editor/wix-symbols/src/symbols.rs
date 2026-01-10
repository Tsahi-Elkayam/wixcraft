//! Symbol extraction from WiX XML files

use crate::types::{Range, Symbol, SymbolKind};
use roxmltree::{Document, Node};

/// Extract symbols from WiX source
pub fn extract_symbols(source: &str) -> Result<Vec<Symbol>, String> {
    let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;

    let mut symbols = Vec::new();
    extract_from_node(doc.root(), source, &mut symbols);

    Ok(symbols)
}

/// Recursively extract symbols from a node
fn extract_from_node(node: Node, source: &str, symbols: &mut Vec<Symbol>) {
    if node.is_element() {
        if let Some(symbol) = create_symbol_for_node(&node, source) {
            let mut sym = symbol;

            // Extract child symbols
            for child in node.children() {
                extract_from_node(child, source, &mut sym.children);
            }

            symbols.push(sym);
            return;
        }
    }

    // Continue traversing even if this node isn't a symbol
    for child in node.children() {
        extract_from_node(child, source, symbols);
    }
}

/// Create a symbol for a node if it represents a known WiX element
fn create_symbol_for_node(node: &Node, source: &str) -> Option<Symbol> {
    let tag_name = node.tag_name().name();

    let (kind, name_attr, detail_attr) = match tag_name {
        // Core elements
        "Wix" => return None, // Skip Wix root, not useful as symbol
        "Package" => (SymbolKind::Module, Some("Name"), Some("Version")),
        "Fragment" => (SymbolKind::Module, Some("Id"), None),
        "Module" => (SymbolKind::Module, Some("Id"), Some("Version")),

        // Components
        "Component" => (SymbolKind::Struct, Some("Id"), None),
        "ComponentGroup" => (SymbolKind::Struct, Some("Id"), None),
        "ComponentRef" => (SymbolKind::Struct, Some("Id"), None),

        // Directories
        "Directory" => (SymbolKind::Namespace, Some("Id"), Some("Name")),
        "DirectoryRef" => (SymbolKind::Namespace, Some("Id"), None),
        "StandardDirectory" => (SymbolKind::Namespace, Some("Id"), None),

        // Features
        "Feature" => (SymbolKind::TypeParameter, Some("Id"), Some("Title")),
        "FeatureRef" => (SymbolKind::TypeParameter, Some("Id"), None),
        "FeatureGroup" => (SymbolKind::TypeParameter, Some("Id"), None),

        // Files
        "File" => {
            // File uses Id or Name
            let id = node.attribute("Id");
            let name = node.attribute("Name").or_else(|| node.attribute("Source"));
            let display_name = id.or(name).unwrap_or("(unnamed)");
            let range = get_node_range(node, source);
            let sel_range = get_attribute_range(node, "Id", source)
                .or_else(|| get_attribute_range(node, "Name", source))
                .unwrap_or(range);

            return Some(
                Symbol::new(display_name.to_string(), SymbolKind::File, range, sel_range)
                    .with_detail(name.unwrap_or_default().to_string()),
            );
        }
        "Binary" => (SymbolKind::File, Some("Id"), Some("SourceFile")),

        // Properties
        "Property" => (SymbolKind::Property, Some("Id"), Some("Value")),
        "SetProperty" => (SymbolKind::Property, Some("Id"), Some("Value")),

        // Custom Actions
        "CustomAction" => (SymbolKind::Function, Some("Id"), Some("Execute")),

        // Registry
        "RegistryKey" => {
            let key = node.attribute("Key").unwrap_or("");
            let root = node.attribute("Root").unwrap_or("HKLM");
            let display = if key.is_empty() {
                root.to_string()
            } else {
                format!("{}\\{}", root, key)
            };
            let range = get_node_range(node, source);
            let sel_range = get_attribute_range(node, "Key", source).unwrap_or(range);

            return Some(Symbol::new(display, SymbolKind::Key, range, sel_range));
        }
        "RegistryValue" => {
            let name = node.attribute("Name").unwrap_or("(Default)");
            let range = get_node_range(node, source);
            let sel_range = get_attribute_range(node, "Name", source).unwrap_or(range);

            return Some(Symbol::new(
                name.to_string(),
                SymbolKind::Key,
                range,
                sel_range,
            ));
        }

        // Services
        "ServiceInstall" => (SymbolKind::Function, Some("Name"), Some("DisplayName")),
        "ServiceControl" => (SymbolKind::Function, Some("Name"), Some("Id")),

        // UI
        "UI" => (SymbolKind::Module, Some("Id"), None),
        "Dialog" => (SymbolKind::Module, Some("Id"), Some("Title")),

        // Bootstrapper/Bundle
        "Bundle" => (SymbolKind::Module, Some("Name"), Some("Version")),
        "Chain" => return None, // Skip Chain, children are more interesting
        "MsiPackage" => (SymbolKind::Module, Some("Id"), Some("SourceFile")),
        "ExePackage" => (SymbolKind::Module, Some("Id"), Some("SourceFile")),
        "MsuPackage" => (SymbolKind::Module, Some("Id"), Some("SourceFile")),

        // Not a symbol-producing element
        _ => return None,
    };

    // Get the name from the preferred attribute
    let name = name_attr
        .and_then(|attr| node.attribute(attr))
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("(unnamed {})", tag_name));

    let detail = detail_attr
        .and_then(|attr| node.attribute(attr))
        .map(|s| s.to_string());

    let range = get_node_range(node, source);
    let sel_range = name_attr
        .and_then(|attr| get_attribute_range(node, attr, source))
        .unwrap_or(range);

    let mut symbol = Symbol::new(name, kind, range, sel_range);
    if let Some(d) = detail {
        if !d.is_empty() {
            symbol = symbol.with_detail(d);
        }
    }

    Some(symbol)
}

/// Get the full range of a node
fn get_node_range(node: &Node, source: &str) -> Range {
    let start = node.range().start;
    let end = node.range().end;
    Range::from_text_pos(source, start, end)
}

/// Get the range of a specific attribute value
fn get_attribute_range(node: &Node, attr_name: &str, source: &str) -> Option<Range> {
    // Find the attribute in the source text
    let node_start = node.range().start;
    let node_text = &source[node.range()];

    // Search for attr="value" or attr='value'
    let patterns = [
        format!("{}=\"", attr_name),
        format!("{}='", attr_name),
        format!("{} =\"", attr_name),
        format!("{} ='", attr_name),
    ];

    for pattern in &patterns {
        if let Some(attr_pos) = node_text.find(pattern.as_str()) {
            let quote_char = if pattern.ends_with('"') { '"' } else { '\'' };
            let value_start = attr_pos + pattern.len();

            if let Some(value_end) = node_text[value_start..].find(quote_char) {
                let abs_start = node_start + value_start;
                let abs_end = node_start + value_start + value_end;
                return Some(Range::from_text_pos(source, abs_start, abs_end));
            }
        }
    }

    None
}

/// Filter symbols by query string (for workspace symbol search)
pub fn filter_symbols<'a>(symbols: &'a [Symbol], query: &str) -> Vec<&'a Symbol> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for symbol in symbols {
        collect_matching_symbols(symbol, &query_lower, &mut results);
    }

    results
}

fn collect_matching_symbols<'a>(symbol: &'a Symbol, query: &str, results: &mut Vec<&'a Symbol>) {
    if symbol.name.to_lowercase().contains(query) {
        results.push(symbol);
    }

    for child in &symbol.children {
        collect_matching_symbols(child, query, results);
    }
}

/// Flatten all symbols to a list
pub fn flatten_symbols(symbols: &[Symbol]) -> Vec<&Symbol> {
    let mut result = Vec::new();
    for symbol in symbols {
        result.extend(symbol.flatten());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_component() {
        let source = r#"<Wix><Component Id="MainComp" Guid="*" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MainComp");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);
    }

    #[test]
    fn test_extract_directory_with_name() {
        let source = r#"<Wix><Directory Id="INSTALLFOLDER" Name="MyApp" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "INSTALLFOLDER");
        assert_eq!(symbols[0].detail, Some("MyApp".to_string()));
    }

    #[test]
    fn test_extract_feature() {
        let source = r#"<Wix><Feature Id="MainFeature" Title="Main Application" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MainFeature");
        assert_eq!(symbols[0].kind, SymbolKind::TypeParameter);
        assert_eq!(symbols[0].detail, Some("Main Application".to_string()));
    }

    #[test]
    fn test_extract_file_with_id() {
        let source = r#"<Wix><File Id="MainExe" Source="app.exe" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MainExe");
        assert_eq!(symbols[0].kind, SymbolKind::File);
    }

    #[test]
    fn test_extract_file_without_id() {
        let source = r#"<Wix><File Name="readme.txt" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "readme.txt");
    }

    #[test]
    fn test_extract_property() {
        let source = r#"<Wix><Property Id="INSTALLDIR" Value="C:\Program Files\MyApp" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "INSTALLDIR");
        assert_eq!(symbols[0].kind, SymbolKind::Property);
    }

    #[test]
    fn test_extract_custom_action() {
        let source = r#"<Wix><CustomAction Id="SetInstallDir" Execute="immediate" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "SetInstallDir");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
    }

    #[test]
    fn test_extract_registry_key() {
        let source = r#"<Wix><RegistryKey Root="HKLM" Key="Software\MyApp" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "HKLM\\Software\\MyApp");
        assert_eq!(symbols[0].kind, SymbolKind::Key);
    }

    #[test]
    fn test_extract_nested_hierarchy() {
        let source = r#"
<Wix>
    <Directory Id="TARGETDIR">
        <Directory Id="ProgramFilesFolder">
            <Directory Id="INSTALLFOLDER" Name="MyApp">
                <Component Id="MainComp">
                    <File Id="MainExe" />
                </Component>
            </Directory>
        </Directory>
    </Directory>
</Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        // Top level: TARGETDIR
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "TARGETDIR");

        // Child: ProgramFilesFolder
        assert_eq!(symbols[0].children.len(), 1);
        assert_eq!(symbols[0].children[0].name, "ProgramFilesFolder");

        // Grandchild: INSTALLFOLDER
        assert_eq!(symbols[0].children[0].children.len(), 1);
        assert_eq!(symbols[0].children[0].children[0].name, "INSTALLFOLDER");

        // Great-grandchild: Component
        let installfolder = &symbols[0].children[0].children[0];
        assert_eq!(installfolder.children.len(), 1);
        assert_eq!(installfolder.children[0].name, "MainComp");

        // Component's child: File
        assert_eq!(installfolder.children[0].children.len(), 1);
        assert_eq!(installfolder.children[0].children[0].name, "MainExe");
    }

    #[test]
    fn test_extract_fragment() {
        let source = r#"<Wix><Fragment Id="UIFragment" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "UIFragment");
        assert_eq!(symbols[0].kind, SymbolKind::Module);
    }

    #[test]
    fn test_extract_package() {
        let source = r#"<Wix><Package Name="MyApp" Version="1.0.0" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MyApp");
        assert_eq!(symbols[0].detail, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_filter_symbols() {
        let source = r#"
<Wix>
    <Component Id="MainComponent" />
    <Component Id="SecondaryComponent" />
    <Feature Id="MainFeature" />
</Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        let filtered = filter_symbols(&symbols, "main");
        assert_eq!(filtered.len(), 2);

        let filtered_comp = filter_symbols(&symbols, "component");
        assert_eq!(filtered_comp.len(), 2);
    }

    #[test]
    fn test_flatten_symbols() {
        let source = r#"
<Wix>
    <Directory Id="TARGETDIR">
        <Component Id="Comp1" />
        <Component Id="Comp2" />
    </Directory>
</Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        let flat = flatten_symbols(&symbols);
        assert_eq!(flat.len(), 3); // Directory + 2 Components
    }

    #[test]
    fn test_empty_source() {
        let source = "<Wix />";
        let symbols = extract_symbols(source).unwrap();
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_invalid_xml() {
        let source = "<Wix><Invalid";
        let result = extract_symbols(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_range() {
        let source = "<Wix><Component Id=\"Test\" /></Wix>";
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols[0].range.start.line, 1);
        assert!(symbols[0].range.start.character > 1);
    }

    #[test]
    fn test_service_install() {
        let source =
            r#"<Wix><ServiceInstall Name="MyService" DisplayName="My Service" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MyService");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert_eq!(symbols[0].detail, Some("My Service".to_string()));
    }

    #[test]
    fn test_component_ref() {
        let source = r#"<Wix><ComponentRef Id="RefComp" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "RefComp");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);
    }

    #[test]
    fn test_bundle() {
        let source = r#"<Wix><Bundle Name="MyInstaller" Version="1.0" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MyInstaller");
        assert_eq!(symbols[0].detail, Some("1.0".to_string()));
    }

    #[test]
    fn test_msi_package() {
        let source = r#"<Wix><MsiPackage Id="MainMsi" SourceFile="main.msi" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MainMsi");
        assert_eq!(symbols[0].detail, Some("main.msi".to_string()));
    }

    #[test]
    fn test_registry_value() {
        let source = r#"<Wix><RegistryValue Name="Version" Value="1.0" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Version");
        assert_eq!(symbols[0].kind, SymbolKind::Key);
    }

    #[test]
    fn test_dialog() {
        let source = r#"<Wix><Dialog Id="WelcomeDlg" Title="Welcome" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "WelcomeDlg");
        assert_eq!(symbols[0].detail, Some("Welcome".to_string()));
    }

    #[test]
    fn test_unnamed_component() {
        let source = r#"<Wix><Component Guid="*" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert!(symbols[0].name.contains("unnamed"));
    }

    #[test]
    fn test_standard_directory() {
        let source = r#"<Wix><StandardDirectory Id="ProgramFilesFolder" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "ProgramFilesFolder");
        assert_eq!(symbols[0].kind, SymbolKind::Namespace);
    }

    #[test]
    fn test_exe_package() {
        let source = r#"<Wix><ExePackage Id="MyExeSetup" SourceFile="setup.exe" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MyExeSetup");
        assert_eq!(symbols[0].detail, Some("setup.exe".to_string()));
    }

    #[test]
    fn test_msu_package() {
        let source = r#"<Wix><MsuPackage Id="MyUpdate" SourceFile="update.msu" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MyUpdate");
        assert_eq!(symbols[0].detail, Some("update.msu".to_string()));
    }

    #[test]
    fn test_registry_key_empty_key() {
        // Test RegistryKey with no Key attribute (only Root)
        let source = r#"<Wix><RegistryKey Root="HKCU" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "HKCU"); // Just the Root when Key is empty
        assert_eq!(symbols[0].kind, SymbolKind::Key);
    }

    #[test]
    fn test_unknown_element() {
        // Unknown element should not produce symbols
        let source = r#"<Wix><SomeUnknownElement Id="Test" /></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        assert!(symbols.is_empty());
    }

    #[test]
    fn test_chain_is_skipped() {
        // Chain element should be skipped, only children matter
        let source = r#"<Wix><Chain><MsiPackage Id="Main" /></Chain></Wix>"#;
        let symbols = extract_symbols(source).unwrap();

        // Should have MsiPackage but not Chain
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Main");
    }
}
