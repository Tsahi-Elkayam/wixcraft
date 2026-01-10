//! WiX language plugin for the generic analysis engine
//!
//! This plugin provides WiX-specific parsing and rules.

use crate::engine::{
    Attribute, CompareOp, Condition, DataRule, Document, ElementPosition, FixAction, FixTemplate,
    LanguagePlugin, Node, ParseResult, PluginCapabilities, RuleCategory, RuleSeverity,
};
use roxmltree;
use std::path::{Path, PathBuf};

/// WiX language plugin
pub struct WixPlugin {
    /// Data-driven rules
    rules: Vec<DataRule>,
}

impl Default for WixPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WixPlugin {
    pub fn new() -> Self {
        Self {
            rules: Self::build_rules(),
        }
    }

    /// Build all data-driven WiX rules
    fn build_rules() -> Vec<DataRule> {
        let mut rules = Vec::new();

        // BP-IDIOM-001: Missing MajorUpgrade
        rules.push(
            DataRule::new("BP-IDIOM-001", "missing-major-upgrade")
                .with_description(
                    "Package should have a MajorUpgrade element for proper upgrade handling",
                )
                .with_severity(RuleSeverity::Medium)
                .with_category(RuleCategory::BestPractice)
                .with_element("Package")
                .with_condition(Condition::MissingChild {
                    element: "MajorUpgrade".to_string(),
                })
                .with_message("Package is missing MajorUpgrade element")
                .with_help("Add <MajorUpgrade DowngradeErrorMessage=\"A newer version is already installed.\" /> to enable proper upgrade handling")
                .with_fix(FixTemplate {
                    description: "Add MajorUpgrade element".to_string(),
                    action: FixAction::AddElement {
                        element: "MajorUpgrade".to_string(),
                        attributes: vec![
                            ("DowngradeErrorMessage".to_string(), "A newer version is already installed.".to_string()),
                        ],
                        position: ElementPosition::First,
                    },
                }),
        );

        // BP-IDIOM-002: Hardcoded GUID (should use *)
        rules.push(
            DataRule::new("BP-IDIOM-002", "hardcoded-component-guid")
                .with_description("Component should use auto-generated GUID (*) instead of hardcoded value")
                .with_severity(RuleSeverity::Low)
                .with_category(RuleCategory::BestPractice)
                .with_element("Component")
                .with_condition(Condition::All(vec![
                    Condition::AttributeExists { name: "Guid".to_string() },
                    Condition::AttributeNotEquals { name: "Guid".to_string(), value: "*".to_string() },
                    Condition::AttributeMatches {
                        name: "Guid".to_string(),
                        pattern: r"^[{(]?[0-9a-fA-F-]{36}[)}]?$".to_string(),
                    },
                ]))
                .with_message("Component '{{id}}' has hardcoded GUID, consider using Guid=\"*\" for auto-generation")
                .with_fix(FixTemplate {
                    description: "Use auto-generated GUID".to_string(),
                    action: FixAction::ReplaceAttribute {
                        name: "Guid".to_string(),
                        new_value: "*".to_string(),
                    },
                }),
        );

        // BP-IDIOM-004: Missing UpgradeCode
        rules.push(
            DataRule::new("BP-IDIOM-004", "missing-upgrade-code")
                .with_description("Package should have an UpgradeCode for proper upgrade support")
                .with_severity(RuleSeverity::High)
                .with_category(RuleCategory::BestPractice)
                .with_element("Package")
                .with_condition(Condition::AttributeMissing {
                    name: "UpgradeCode".to_string(),
                })
                .with_message("Package is missing UpgradeCode attribute")
                .with_help("Add UpgradeCode=\"{GUID}\" to enable upgrade detection"),
        );

        // BP-PERF-001: Multi-file component
        rules.push(
            DataRule::new("BP-PERF-001", "multi-file-component")
                .with_description("Component should contain only one file for optimal repair behavior")
                .with_severity(RuleSeverity::Low)
                .with_category(RuleCategory::Performance)
                .with_element("Component")
                .with_condition(Condition::ChildCount {
                    element: "File".to_string(),
                    op: CompareOp::Gt,
                    value: 1,
                })
                .with_message("Component '{{id}}' contains multiple files, consider splitting into separate components")
                .with_help("Components with single files have better repair behavior and smaller reinstall footprint"),
        );

        // BP-MAINT-001: Hardcoded absolute path
        rules.push(
            DataRule::new("BP-MAINT-001", "hardcoded-absolute-path")
                .with_description("Avoid hardcoded absolute paths in Source attribute")
                .with_severity(RuleSeverity::Medium)
                .with_category(RuleCategory::Maintainability)
                .with_element("File")
                .with_condition(Condition::AttributeMatches {
                    name: "Source".to_string(),
                    pattern: r"^[A-Za-z]:\\".to_string(),
                })
                .with_message("File has hardcoded absolute path in Source attribute")
                .with_help("Use relative paths or preprocessor variables like $(var.SourceDir)"),
        );

        // SEC-001: LocalSystem service account
        rules.push(
            DataRule::new("SEC-001", "localsystem-service")
                .with_description("Service running as LocalSystem has excessive privileges")
                .with_severity(RuleSeverity::High)
                .with_category(RuleCategory::Security)
                .with_element("ServiceInstall")
                .with_condition(Condition::Any(vec![
                    Condition::AttributeMissing { name: "Account".to_string() },
                    Condition::AttributeEquals {
                        name: "Account".to_string(),
                        value: "LocalSystem".to_string(),
                    },
                ]))
                .with_message("Service '{{id}}' runs as LocalSystem which has excessive privileges")
                .with_help("Consider using LocalService, NetworkService, or a dedicated service account"),
        );

        // SEC-005: Hardcoded sensitive property
        rules.push(
            DataRule::new("SEC-005", "hardcoded-sensitive-property")
                .with_description("Property with sensitive name has hardcoded value")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Security)
                .with_element("Property")
                .with_condition(Condition::All(vec![
                    Condition::AttributeExists { name: "Value".to_string() },
                    Condition::AttributeMatches {
                        name: "Id".to_string(),
                        pattern: r"(?i)(password|secret|key|token|credential)".to_string(),
                    },
                ]))
                .with_message("Property '{{id}}' appears to contain hardcoded sensitive data")
                .with_help("Remove hardcoded values and require them to be provided at install time")
                .with_fix(FixTemplate {
                    description: "Remove hardcoded value".to_string(),
                    action: FixAction::RemoveAttribute { name: "Value".to_string() },
                }),
        );

        // DEAD-005: Empty feature
        rules.push(
            DataRule::new("DEAD-005", "empty-feature")
                .with_description("Feature has no content")
                .with_severity(RuleSeverity::Low)
                .with_category(RuleCategory::DeadCode)
                .with_element("Feature")
                .with_condition(Condition::All(vec![
                    Condition::ChildCount { element: "ComponentRef".to_string(), op: CompareOp::Eq, value: 0 },
                    Condition::ChildCount { element: "ComponentGroupRef".to_string(), op: CompareOp::Eq, value: 0 },
                    Condition::ChildCount { element: "FeatureRef".to_string(), op: CompareOp::Eq, value: 0 },
                    Condition::ChildCount { element: "Feature".to_string(), op: CompareOp::Eq, value: 0 },
                ]))
                .with_message("Feature '{{id}}' has no content")
                .with_help("Add components or remove the empty feature"),
        );

        // VAL-ATTR-001: Missing required attribute (Component.Guid)
        rules.push(
            DataRule::new("VAL-ATTR-001", "component-missing-guid")
                .with_description("Component requires a Guid attribute")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("Component")
                .with_condition(Condition::AttributeMissing {
                    name: "Guid".to_string(),
                })
                .with_message("Component '{{id}}' is missing required Guid attribute")
                .with_help("Add Guid=\"*\" for auto-generation or specify a GUID")
                .with_fix(FixTemplate {
                    description: "Add auto-generated GUID".to_string(),
                    action: FixAction::AddAttribute {
                        name: "Guid".to_string(),
                        value: "*".to_string(),
                    },
                }),
        );

        // VAL-ATTR-002: Invalid GUID format
        rules.push(
            DataRule::new("VAL-ATTR-002", "invalid-guid-format")
                .with_description("GUID attribute has invalid format")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("Component")
                .with_condition(Condition::All(vec![
                    Condition::AttributeExists { name: "Guid".to_string() },
                    Condition::AttributeNotEquals { name: "Guid".to_string(), value: "*".to_string() },
                    Condition::AttributeNotMatches {
                        name: "Guid".to_string(),
                        pattern: r"^[{(]?[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}[)}]?$".to_string(),
                    },
                ]))
                .with_message("Component '{{id}}' has invalid GUID format")
                .with_help("Use a valid GUID format or Guid=\"*\" for auto-generation"),
        );

        // BP-IDIOM-003: Deprecated Product element
        rules.push(
            DataRule::new("BP-IDIOM-003", "deprecated-product-element")
                .with_description("Product element is deprecated in WiX v4, use Package instead")
                .with_severity(RuleSeverity::Medium)
                .with_category(RuleCategory::BestPractice)
                .with_element("Product")
                .with_condition(Condition::Always)
                .with_message("Product element is deprecated in WiX v4")
                .with_help("Replace <Product> with <Package> for WiX v4 compatibility"),
        );

        // VAL-ATTR-003: Invalid YesNo value
        rules.push(
            DataRule::new("VAL-ATTR-003", "invalid-yesno-value")
                .with_description("Attribute requires yes/no value")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("Component")
                .with_condition(Condition::All(vec![
                    Condition::AttributeExists { name: "Permanent".to_string() },
                    Condition::AttributeNotIn {
                        name: "Permanent".to_string(),
                        values: vec!["yes".to_string(), "no".to_string()],
                    },
                ]))
                .with_message("Component '{{id}}' has invalid Permanent value, must be 'yes' or 'no'"),
        );

        // =====================================================================
        // VAL-REL-001: Parent-child relationship validation
        // =====================================================================

        // RegistryValue valid parents: Component, RegistryKey
        rules.push(
            DataRule::new("VAL-REL-001-RegistryValue", "invalid-parent-registryvalue")
                .with_description("RegistryValue must be a child of Component or RegistryKey")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("RegistryValue")
                .with_condition(Condition::ParentNotIn {
                    elements: vec![
                        "Component".to_string(),
                        "RegistryKey".to_string(),
                        "RegistryValue".to_string(),
                    ],
                })
                .with_message("RegistryValue cannot be a child of {{parent}}. Valid parents: Component, RegistryKey"),
        );

        // Directory valid parents: Directory, DirectoryRef, Fragment, Package, StandardDirectory, Wix
        rules.push(
            DataRule::new("VAL-REL-001-Directory", "invalid-parent-directory")
                .with_description("Directory must be a child of Directory, DirectoryRef, Fragment, Package, or StandardDirectory")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("Directory")
                .with_condition(Condition::ParentNotIn {
                    elements: vec![
                        "Directory".to_string(),
                        "DirectoryRef".to_string(),
                        "Fragment".to_string(),
                        "Package".to_string(),
                        "StandardDirectory".to_string(),
                        "Wix".to_string(),
                        "root".to_string(), // Engine root node
                    ],
                })
                .with_message("Directory cannot be a child of {{parent}}. Valid parents: Directory, DirectoryRef, Fragment, Package, StandardDirectory"),
        );

        // Feature valid parents: Package, Fragment, Feature, FeatureRef, FeatureGroup, Module
        rules.push(
            DataRule::new("VAL-REL-001-Feature", "invalid-parent-feature")
                .with_description("Feature must be a child of Package, Fragment, Feature, or FeatureGroup")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("Feature")
                .with_condition(Condition::ParentNotIn {
                    elements: vec![
                        "Package".to_string(),
                        "Fragment".to_string(),
                        "Feature".to_string(),
                        "FeatureRef".to_string(),
                        "FeatureGroup".to_string(),
                        "Module".to_string(),
                        "Wix".to_string(),
                        "root".to_string(),
                    ],
                })
                .with_message("Feature cannot be a child of {{parent}}. Valid parents: Package, Fragment, Feature, FeatureGroup"),
        );

        // Component valid parents: Directory, DirectoryRef, ComponentGroup, Fragment
        rules.push(
            DataRule::new("VAL-REL-001-Component", "invalid-parent-component")
                .with_description("Component must be a child of Directory, DirectoryRef, ComponentGroup, or Fragment")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("Component")
                .with_condition(Condition::ParentNotIn {
                    elements: vec![
                        "Directory".to_string(),
                        "DirectoryRef".to_string(),
                        "ComponentGroup".to_string(),
                        "Fragment".to_string(),
                        "StandardDirectory".to_string(),
                        "Wix".to_string(),
                        "root".to_string(),
                    ],
                })
                .with_message("Component cannot be a child of {{parent}}. Valid parents: Directory, DirectoryRef, ComponentGroup, Fragment"),
        );

        // File valid parents: Component
        rules.push(
            DataRule::new("VAL-REL-001-File", "invalid-parent-file")
                .with_description("File must be a child of Component")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("File")
                .with_condition(Condition::ParentNotIn {
                    elements: vec!["Component".to_string()],
                })
                .with_message("File cannot be a child of {{parent}}. Valid parent: Component"),
        );

        // =====================================================================
        // VAL-ATTR-001: Required attribute validation for other elements
        // =====================================================================

        // Feature requires Id
        rules.push(
            DataRule::new("VAL-ATTR-001-Feature", "feature-missing-id")
                .with_description("Feature requires an Id attribute")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("Feature")
                .with_condition(Condition::AttributeMissing {
                    name: "Id".to_string(),
                })
                .with_message("Feature is missing required Id attribute"),
        );

        // CustomAction requires Id
        rules.push(
            DataRule::new("VAL-ATTR-001-CustomAction", "customaction-missing-id")
                .with_description("CustomAction requires an Id attribute")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("CustomAction")
                .with_condition(Condition::AttributeMissing {
                    name: "Id".to_string(),
                })
                .with_message("CustomAction is missing required Id attribute"),
        );

        // Property requires Id
        rules.push(
            DataRule::new("VAL-ATTR-001-Property", "property-missing-id")
                .with_description("Property requires an Id attribute")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("Property")
                .with_condition(Condition::AttributeMissing {
                    name: "Id".to_string(),
                })
                .with_message("Property is missing required Id attribute"),
        );

        // RegistryValue requires Type
        rules.push(
            DataRule::new("VAL-ATTR-001-RegistryValue", "registryvalue-missing-type")
                .with_description("RegistryValue requires a Type attribute")
                .with_severity(RuleSeverity::Critical)
                .with_category(RuleCategory::Validation)
                .with_element("RegistryValue")
                .with_condition(Condition::AttributeMissing {
                    name: "Type".to_string(),
                })
                .with_message("RegistryValue is missing required Type attribute"),
        );

        // =====================================================================
        // BP-MAINT-002: Naming convention rules
        // =====================================================================

        // Component naming convention
        rules.push(
            DataRule::new("BP-MAINT-002-Component", "component-naming-convention")
                .with_description("Component Id should follow naming convention")
                .with_severity(RuleSeverity::Info)
                .with_category(RuleCategory::Maintainability)
                .with_element("Component")
                .with_condition(Condition::All(vec![
                    Condition::AttributeExists { name: "Id".to_string() },
                    Condition::AttributeNotMatches {
                        name: "Id".to_string(),
                        pattern: r"^(C_|cmp|Cmp|Component)".to_string(),
                    },
                ]))
                .with_message("Component Id '{{id}}': Consider prefixing with 'C_' or 'cmp'"),
        );

        // Feature naming convention
        rules.push(
            DataRule::new("BP-MAINT-002-Feature", "feature-naming-convention")
                .with_description("Feature Id should follow naming convention")
                .with_severity(RuleSeverity::Info)
                .with_category(RuleCategory::Maintainability)
                .with_element("Feature")
                .with_condition(Condition::All(vec![
                    Condition::AttributeExists { name: "Id".to_string() },
                    Condition::AttributeNotMatches {
                        name: "Id".to_string(),
                        pattern: r"^(F_|feat|Feat|Feature)".to_string(),
                    },
                ]))
                .with_message("Feature Id '{{id}}': Consider prefixing with 'F_' or 'feat'"),
        );

        // Directory naming convention
        rules.push(
            DataRule::new("BP-MAINT-002-Directory", "directory-naming-convention")
                .with_description("Directory Id should follow naming convention")
                .with_severity(RuleSeverity::Info)
                .with_category(RuleCategory::Maintainability)
                .with_element("Directory")
                .with_condition(Condition::All(vec![
                    Condition::AttributeExists { name: "Id".to_string() },
                    Condition::AttributeNotMatches {
                        name: "Id".to_string(),
                        // Allow standard directories like TARGETDIR, ProgramFilesFolder, etc.
                        pattern: r"^(D_|dir|Dir|Directory|TARGETDIR|ProgramFilesFolder|ProgramFiles64Folder|CommonFilesFolder|SystemFolder|WindowsFolder|TempFolder|LocalAppDataFolder|AppDataFolder|INSTALLFOLDER|INSTALLDIR)".to_string(),
                    },
                ]))
                .with_message("Directory Id '{{id}}': Consider prefixing with 'D_' or 'dir'"),
        );

        // Property naming convention (uppercase)
        rules.push(
            DataRule::new("BP-MAINT-002-Property", "property-naming-convention")
                .with_description("Public Property Id should be uppercase")
                .with_severity(RuleSeverity::Info)
                .with_category(RuleCategory::Maintainability)
                .with_element("Property")
                .with_condition(Condition::All(vec![
                    Condition::AttributeExists { name: "Id".to_string() },
                    // Property should be uppercase if it's meant to be public
                    Condition::AttributeMatches {
                        name: "Id".to_string(),
                        pattern: r"^[a-z]".to_string(), // Starts with lowercase
                    },
                ]))
                .with_message("Property Id '{{id}}': Public properties should be UPPERCASE"),
        );

        rules
    }
}

impl LanguagePlugin for WixPlugin {
    fn id(&self) -> &str {
        "wix"
    }

    fn name(&self) -> &str {
        "WiX Toolset"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn extensions(&self) -> &[&str] {
        &[".wxs", ".wxi", ".wxl"]
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::new()
            .with_data_rules()
            .with_code_rules()
            .with_auto_fix()
    }

    fn parse(&self, path: &Path, content: &str) -> ParseResult {
        match roxmltree::Document::parse(content) {
            Ok(doc) => {
                let wix_doc = WixEngineDocument::new(content.to_string(), path.to_path_buf(), doc);
                ParseResult::Ok(Box::new(wix_doc))
            }
            Err(e) => ParseResult::Error {
                message: e.to_string(),
                line: Some(e.pos().row as usize),
                column: Some(e.pos().col as usize),
            },
        }
    }

    fn data_rules(&self) -> Vec<DataRule> {
        self.rules.clone()
    }
}

/// WiX document wrapper implementing the engine Document trait
pub struct WixEngineDocument {
    source: String,
    path: PathBuf,
    // We store the parsed doc as a string and re-parse when needed
    // because roxmltree::Document has lifetime constraints
    _marker: std::marker::PhantomData<()>,
}

impl WixEngineDocument {
    fn new(source: String, path: PathBuf, _doc: roxmltree::Document) -> Self {
        Self {
            source,
            path,
            _marker: std::marker::PhantomData,
        }
    }
}

impl Document for WixEngineDocument {
    fn source(&self) -> &str {
        &self.source
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn root(&self) -> &dyn Node {
        // This is tricky because of lifetimes - we need a different approach
        // For now, return a static empty node as a placeholder
        // The real implementation needs an owned node structure
        static EMPTY: WixNode = WixNode::empty();
        &EMPTY
    }

    fn node_at(&self, _line: usize, _column: usize) -> Option<&dyn Node> {
        None
    }
}

/// WiX node wrapper implementing the engine Node trait
pub struct WixNode {
    kind: String,
    text: String,
    range: (usize, usize, usize, usize),
    attributes: Vec<Attribute>,
    children: Vec<WixNode>,
    parent_phantom: Option<PhantomParent>,
}

impl WixNode {
    const fn empty() -> Self {
        Self {
            kind: String::new(),
            text: String::new(),
            range: (0, 0, 0, 0),
            attributes: Vec::new(),
            children: Vec::new(),
            parent_phantom: None,
        }
    }

    /// Build a node tree from roxmltree document
    pub fn from_roxmltree(source: &str, doc: &roxmltree::Document) -> Self {
        Self::from_roxmltree_node(source, doc.root(), None)
    }

    fn from_roxmltree_node(source: &str, node: roxmltree::Node, parent_kind: Option<&str>) -> Self {
        let kind = if node.is_element() {
            node.tag_name().name().to_string()
        } else if node.is_text() {
            "text".to_string()
        } else if node.is_comment() {
            "comment".to_string()
        } else {
            "root".to_string()
        };

        let text = node.text().unwrap_or("").to_string();

        let range = {
            let r = node.range();
            let (start_line, start_col) = offset_to_line_col(source, r.start);
            let (end_line, end_col) = offset_to_line_col(source, r.end);
            (start_line, start_col, end_line, end_col)
        };

        let attributes = node
            .attributes()
            .map(|a| Attribute::new(a.name(), a.value()))
            .collect();

        let current_kind = kind.clone();
        let children = node
            .children()
            .filter(|c| c.is_element())
            .map(|c| Self::from_roxmltree_node(source, c, Some(&current_kind)))
            .collect();

        Self {
            kind,
            text,
            range,
            attributes,
            children,
            parent_phantom: parent_kind.map(|s| PhantomParent { kind: s.to_string() }),
        }
    }
}

fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// A phantom parent node that only stores the kind (for parent validation)
pub struct PhantomParent {
    kind: String,
}

impl Node for PhantomParent {
    fn kind(&self) -> &str {
        &self.kind
    }

    fn text(&self) -> &str {
        ""
    }

    fn range(&self) -> (usize, usize, usize, usize) {
        (0, 0, 0, 0)
    }

    fn parent(&self) -> Option<&dyn Node> {
        None
    }

    fn children(&self) -> Vec<&dyn Node> {
        Vec::new()
    }

    fn attribute(&self, _name: &str) -> Option<&str> {
        None
    }

    fn attributes(&self) -> Vec<Attribute> {
        Vec::new()
    }
}

impl Node for WixNode {
    fn kind(&self) -> &str {
        &self.kind
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn range(&self) -> (usize, usize, usize, usize) {
        self.range
    }

    fn parent(&self) -> Option<&dyn Node> {
        self.parent_phantom.as_ref().map(|p| p as &dyn Node)
    }

    fn children(&self) -> Vec<&dyn Node> {
        self.children.iter().map(|c| c as &dyn Node).collect()
    }

    fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value.as_str())
    }

    fn attributes(&self) -> Vec<Attribute> {
        self.attributes.clone()
    }
}

/// An owned WiX document with owned node tree
pub struct OwnedWixDocument {
    source: String,
    path: PathBuf,
    root: WixNode,
}

impl OwnedWixDocument {
    pub fn parse(source: &str, path: &Path) -> Result<Self, String> {
        let doc = roxmltree::Document::parse(source)
            .map_err(|e| format!("XML parse error: {}", e))?;

        let root = WixNode::from_roxmltree(source, &doc);

        Ok(Self {
            source: source.to_string(),
            path: path.to_path_buf(),
            root,
        })
    }
}

impl Document for OwnedWixDocument {
    fn source(&self) -> &str {
        &self.source
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn root(&self) -> &dyn Node {
        &self.root
    }

    fn node_at(&self, line: usize, column: usize) -> Option<&dyn Node> {
        fn find_at<'a>(node: &'a WixNode, line: usize, col: usize) -> Option<&'a dyn Node> {
            let (sl, sc, el, ec) = node.range;
            if line >= sl && line <= el {
                // Check children first (more specific)
                for child in &node.children {
                    if let Some(found) = find_at(child, line, col) {
                        return Some(found);
                    }
                }
                // Check if within this node's range
                if (line > sl || (line == sl && col >= sc)) && (line < el || (line == el && col <= ec)) {
                    return Some(node);
                }
            }
            None
        }
        find_at(&self.root, line, column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::evaluator::RuleEvaluator;

    #[test]
    fn test_wix_plugin_basics() {
        let plugin = WixPlugin::new();
        assert_eq!(plugin.id(), "wix");
        assert_eq!(plugin.name(), "WiX Toolset");
        assert!(plugin.extensions().contains(&".wxs"));
    }

    #[test]
    fn test_wix_plugin_can_handle() {
        let plugin = WixPlugin::new();
        assert!(plugin.can_handle(Path::new("test.wxs")));
        assert!(plugin.can_handle(Path::new("include.wxi")));
        assert!(plugin.can_handle(Path::new("strings.wxl")));
        assert!(!plugin.can_handle(Path::new("test.xml")));
    }

    #[test]
    fn test_wix_plugin_has_rules() {
        let plugin = WixPlugin::new();
        let rules = plugin.data_rules();
        assert!(!rules.is_empty());

        // Check specific rules exist
        assert!(rules.iter().any(|r| r.id == "BP-IDIOM-001"));
        assert!(rules.iter().any(|r| r.id == "SEC-001"));
    }

    #[test]
    fn test_owned_wix_document_parse() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        assert_eq!(doc.source(), source);
        assert_eq!(doc.path().file_name().unwrap().to_str().unwrap(), "test.wxs");
    }

    #[test]
    fn test_owned_wix_document_root() {
        let source = r#"<Wix><Package Name="Test" /></Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let root = doc.root();
        assert_eq!(root.kind(), "root");

        let children = root.children();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].kind(), "Wix");
    }

    #[test]
    fn test_owned_wix_document_attributes() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let wix = &doc.root().children()[0];
        let pkg = &wix.children()[0];

        assert_eq!(pkg.kind(), "Package");
        assert_eq!(pkg.attribute("Name"), Some("Test"));
        assert_eq!(pkg.attribute("Version"), Some("1.0"));
    }

    #[test]
    fn test_wix_node_descendants() {
        let source = r#"<Wix>
            <Package Name="Test">
                <Component Id="C1" Guid="*">
                    <File Id="F1" Source="app.exe" />
                </Component>
            </Package>
        </Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let root = doc.root();
        let descendants = root.descendants();

        // Should have Wix, Package, Component, File
        assert!(descendants.len() >= 4);
        assert!(descendants.iter().any(|n| n.kind() == "Component"));
        assert!(descendants.iter().any(|n| n.kind() == "File"));
    }

    #[test]
    fn test_missing_major_upgrade_rule() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let plugin = WixPlugin::new();
        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(std::sync::Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&doc);

        assert!(diagnostics.iter().any(|d| d.rule_id == "BP-IDIOM-001"));
    }

    #[test]
    fn test_missing_major_upgrade_rule_passes() {
        let source = r#"<Wix><Package Name="Test" Version="1.0"><MajorUpgrade /></Package></Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let plugin = WixPlugin::new();
        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(std::sync::Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&doc);

        assert!(!diagnostics.iter().any(|d| d.rule_id == "BP-IDIOM-001"));
    }

    #[test]
    fn test_component_missing_guid_rule() {
        let source = r#"<Wix><Component Id="C1" /></Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let plugin = WixPlugin::new();
        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(std::sync::Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&doc);

        assert!(diagnostics.iter().any(|d| d.rule_id == "VAL-ATTR-001"));
    }

    #[test]
    fn test_localsystem_service_rule() {
        let source = r#"<Wix><ServiceInstall Id="Svc1" Name="MyService" /></Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let plugin = WixPlugin::new();
        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(std::sync::Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&doc);

        // Missing Account defaults to LocalSystem
        assert!(diagnostics.iter().any(|d| d.rule_id == "SEC-001"));
    }

    #[test]
    fn test_hardcoded_sensitive_property() {
        let source = r#"<Wix><Property Id="DB_PASSWORD" Value="secret123" /></Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let plugin = WixPlugin::new();
        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(std::sync::Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&doc);

        assert!(diagnostics.iter().any(|d| d.rule_id == "SEC-005"));
    }

    #[test]
    fn test_multi_file_component() {
        let source = r#"<Wix>
            <Component Id="C1" Guid="*">
                <File Id="F1" Source="a.exe" />
                <File Id="F2" Source="b.dll" />
            </Component>
        </Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let plugin = WixPlugin::new();
        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(std::sync::Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&doc);

        assert!(diagnostics.iter().any(|d| d.rule_id == "BP-PERF-001"));
    }

    #[test]
    fn test_empty_feature() {
        let source = r#"<Wix><Feature Id="MainFeature" /></Wix>"#;
        let doc = OwnedWixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let plugin = WixPlugin::new();
        let mut evaluator = RuleEvaluator::new();
        evaluator.register_plugin(std::sync::Arc::new(plugin));

        let diagnostics = evaluator.evaluate(&doc);

        assert!(diagnostics.iter().any(|d| d.rule_id == "DEAD-005"));
    }
}
