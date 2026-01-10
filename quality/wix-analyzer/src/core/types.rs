//! Core types for WiX analysis

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Position in a file (1-based for LSP compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

impl Position {
    pub fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
}

/// Range in a file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create range from byte offsets in source
    pub fn from_offsets(source: &str, start: usize, end: usize) -> Self {
        let start_pos = offset_to_position(source, start);
        let end_pos = offset_to_position(source, end);
        Self::new(start_pos, end_pos)
    }
}

/// Convert byte offset to Position
fn offset_to_position(source: &str, offset: usize) -> Position {
    let mut line = 1;
    let mut character = 1;

    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 1;
        } else {
            character += 1;
        }
    }

    Position::new(line, character)
}

/// Location in a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub range: Range,
}

impl Location {
    pub fn new(file: PathBuf, range: Range) -> Self {
        Self { file, range }
    }
}

/// Diagnostic severity (SonarQube-inspired)
/// Ordered from lowest to highest priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational, suggestion only
    Info = 1,
    /// Minor issue, fix when convenient
    Low = 2,
    /// Moderate issue, should fix
    Medium = 3,
    /// Serious issue with high impact, fix before release
    High = 4,
    /// Critical issue, must fix immediately - blocks release
    Blocker = 5,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Blocker => "blocker",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Info => "info",
        }
    }

    /// Map to LSP DiagnosticSeverity (1=Error, 2=Warning, 3=Info, 4=Hint)
    pub fn to_lsp_severity(&self) -> u8 {
        match self {
            Severity::Blocker | Severity::High => 1, // Error
            Severity::Medium => 2,                   // Warning
            Severity::Low => 3,                      // Info
            Severity::Info => 4,                     // Hint
        }
    }

    /// Map from legacy severity names for backwards compatibility
    pub fn from_legacy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(Severity::High),
            "warning" => Some(Severity::Medium),
            "info" => Some(Severity::Info),
            "blocker" => Some(Severity::Blocker),
            "high" => Some(Severity::High),
            "medium" => Some(Severity::Medium),
            "low" => Some(Severity::Low),
            _ => None,
        }
    }
}

/// Issue category (SonarQube-inspired)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    /// Code that will break or produce wrong results
    Bug,
    /// Security holes (OWASP, CWE)
    Vulnerability,
    /// Maintainability issues, bad patterns
    CodeSmell,
    /// Code needing manual security review
    SecurityHotspot,
    /// Exposed credentials, API keys, tokens
    Secret,
}

impl IssueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueType::Bug => "bug",
            IssueType::Vulnerability => "vulnerability",
            IssueType::CodeSmell => "code_smell",
            IssueType::SecurityHotspot => "security_hotspot",
            IssueType::Secret => "secret",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            IssueType::Bug => "Bug",
            IssueType::Vulnerability => "Vulnerability",
            IssueType::CodeSmell => "Code Smell",
            IssueType::SecurityHotspot => "Security Hotspot",
            IssueType::Secret => "Secret",
        }
    }
}

/// Diagnostic category (legacy, maps to IssueType)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// Validation errors (references, relationships, attributes) -> Bug
    Validation,
    /// Best practice suggestions -> CodeSmell
    BestPractice,
    /// Security issues -> Vulnerability
    Security,
    /// Dead/unused code -> CodeSmell
    DeadCode,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Validation => "validation",
            Category::BestPractice => "best-practice",
            Category::Security => "security",
            Category::DeadCode => "dead-code",
        }
    }

    /// Convert legacy Category to new IssueType
    pub fn to_issue_type(&self) -> IssueType {
        match self {
            Category::Validation => IssueType::Bug,
            Category::BestPractice => IssueType::CodeSmell,
            Category::Security => IssueType::Vulnerability,
            Category::DeadCode => IssueType::CodeSmell,
        }
    }
}

/// A suggested fix for a diagnostic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fix {
    pub description: String,
    pub action: FixAction,
}

impl Fix {
    pub fn new(description: impl Into<String>, action: FixAction) -> Self {
        Self {
            description: description.into(),
            action,
        }
    }
}

/// Fix action types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FixAction {
    /// Add an attribute to an element
    AddAttribute {
        range: Range,
        name: String,
        value: String,
    },
    /// Remove an attribute from an element
    RemoveAttribute { range: Range, name: String },
    /// Replace an attribute value
    ReplaceAttribute {
        range: Range,
        name: String,
        new_value: String,
    },
    /// Add an element as child
    AddElement {
        parent_range: Range,
        element: String,
        position: InsertPosition,
    },
    /// Remove an element
    RemoveElement { range: Range },
    /// Replace text in range
    ReplaceText { range: Range, new_text: String },
}

/// Position for inserting elements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InsertPosition {
    First,
    Last,
    BeforeElement(String),
    AfterElement(String),
}

/// Related diagnostic information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedInfo {
    pub location: Location,
    pub message: String,
}

impl RelatedInfo {
    pub fn new(location: Location, message: impl Into<String>) -> Self {
        Self {
            location,
            message: message.into(),
        }
    }
}

/// Security standard reference (CWE, OWASP, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityStandard {
    /// CWE identifier (e.g., "CWE-79", "CWE-89")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwe: Option<String>,
    /// OWASP Top 10 category (e.g., "A03:2021-Injection")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owasp: Option<String>,
    /// SANS/CWE Top 25 rank
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sans_top25: Option<u8>,
}

impl SecurityStandard {
    pub fn new() -> Self {
        Self {
            cwe: None,
            owasp: None,
            sans_top25: None,
        }
    }

    pub fn with_cwe(mut self, cwe: impl Into<String>) -> Self {
        self.cwe = Some(cwe.into());
        self
    }

    pub fn with_owasp(mut self, owasp: impl Into<String>) -> Self {
        self.owasp = Some(owasp.into());
        self
    }

    pub fn with_sans_top25(mut self, rank: u8) -> Self {
        self.sans_top25 = Some(rank);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.cwe.is_none() && self.owasp.is_none() && self.sans_top25.is_none()
    }
}

impl Default for SecurityStandard {
    fn default() -> Self {
        Self::new()
    }
}

/// A diagnostic message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Rule identifier (e.g., "SEC-001", "BP-002")
    pub rule_id: String,
    /// Legacy category (for backwards compatibility)
    pub category: Category,
    /// New issue type (SonarQube-style)
    #[serde(default = "default_issue_type")]
    pub issue_type: IssueType,
    /// Severity level
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
    /// Location in source
    pub location: Location,
    /// Help text explaining how to fix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    /// Suggested automatic fix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<Fix>,
    /// Related locations (e.g., where symbol is defined)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub related: Vec<RelatedInfo>,
    /// Estimated effort to fix in minutes (for technical debt)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort_minutes: Option<u32>,
    /// Tags for filtering (e.g., "security", "performance", "convention")
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    /// Security standard references (CWE, OWASP, SANS Top 25)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityStandard>,
    /// Documentation URL for the rule
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_url: Option<String>,
}

fn default_issue_type() -> IssueType {
    IssueType::CodeSmell
}

impl Diagnostic {
    /// Create a new diagnostic with full control over all fields
    pub fn new(
        rule_id: impl Into<String>,
        category: Category,
        severity: Severity,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        let cat = category;
        Self {
            rule_id: rule_id.into(),
            category: cat,
            issue_type: cat.to_issue_type(),
            severity,
            message: message.into(),
            location,
            help: None,
            fix: None,
            related: Vec::new(),
            effort_minutes: None,
            tags: Vec::new(),
            security: None,
            doc_url: None,
        }
    }

    /// Create a diagnostic with explicit issue type
    pub fn with_issue_type(
        rule_id: impl Into<String>,
        issue_type: IssueType,
        severity: Severity,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        // Map issue_type back to legacy category for backwards compat
        let category = match issue_type {
            IssueType::Bug => Category::Validation,
            IssueType::Vulnerability | IssueType::SecurityHotspot | IssueType::Secret => {
                Category::Security
            }
            IssueType::CodeSmell => Category::BestPractice,
        };
        Self {
            rule_id: rule_id.into(),
            category,
            issue_type,
            severity,
            message: message.into(),
            location,
            help: None,
            fix: None,
            related: Vec::new(),
            effort_minutes: None,
            tags: Vec::new(),
            security: None,
            doc_url: None,
        }
    }

    // === Convenience constructors by severity ===

    /// Create a blocker-level diagnostic (must fix immediately)
    pub fn blocker(
        rule_id: impl Into<String>,
        issue_type: IssueType,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self::with_issue_type(rule_id, issue_type, Severity::Blocker, message, location)
    }

    /// Create a high-severity diagnostic (fix before release)
    pub fn high(
        rule_id: impl Into<String>,
        issue_type: IssueType,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self::with_issue_type(rule_id, issue_type, Severity::High, message, location)
    }

    /// Create a medium-severity diagnostic (should fix)
    pub fn medium(
        rule_id: impl Into<String>,
        issue_type: IssueType,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self::with_issue_type(rule_id, issue_type, Severity::Medium, message, location)
    }

    /// Create a low-severity diagnostic (fix when convenient)
    pub fn low(
        rule_id: impl Into<String>,
        issue_type: IssueType,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self::with_issue_type(rule_id, issue_type, Severity::Low, message, location)
    }

    // === Legacy convenience constructors (for backwards compatibility) ===

    /// Legacy: Create error diagnostic (maps to High severity)
    pub fn error(
        rule_id: impl Into<String>,
        category: Category,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self::new(rule_id, category, Severity::High, message, location)
    }

    /// Legacy: Create warning diagnostic (maps to Medium severity)
    pub fn warning(
        rule_id: impl Into<String>,
        category: Category,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self::new(rule_id, category, Severity::Medium, message, location)
    }

    /// Legacy: Create info diagnostic (maps to Info severity)
    pub fn info(
        rule_id: impl Into<String>,
        category: Category,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self::new(rule_id, category, Severity::Info, message, location)
    }

    // === Builder methods ===

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }

    pub fn with_related(mut self, related: RelatedInfo) -> Self {
        self.related.push(related);
        self
    }

    /// Set estimated effort to fix in minutes (for technical debt calculation)
    pub fn with_effort(mut self, minutes: u32) -> Self {
        self.effort_minutes = Some(minutes);
        self
    }

    /// Add a tag for filtering
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(|t| t.into()));
        self
    }

    /// Set security standard references (CWE, OWASP, SANS Top 25)
    pub fn with_security(mut self, security: SecurityStandard) -> Self {
        self.security = Some(security);
        self
    }

    /// Set CWE identifier (convenience method)
    pub fn with_cwe(mut self, cwe: impl Into<String>) -> Self {
        let sec = self.security.get_or_insert_with(SecurityStandard::new);
        sec.cwe = Some(cwe.into());
        self
    }

    /// Set OWASP category (convenience method)
    pub fn with_owasp(mut self, owasp: impl Into<String>) -> Self {
        let sec = self.security.get_or_insert_with(SecurityStandard::new);
        sec.owasp = Some(owasp.into());
        self
    }

    /// Set documentation URL for the rule
    pub fn with_doc_url(mut self, url: impl Into<String>) -> Self {
        self.doc_url = Some(url.into());
        self
    }

    /// Generate a stable fingerprint for this diagnostic (for SARIF)
    /// Based on rule_id, file path, and code context
    pub fn fingerprint(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.rule_id.hash(&mut hasher);
        self.location.file.to_string_lossy().hash(&mut hasher);
        self.location.range.start.line.hash(&mut hasher);
        // Include message prefix for context (first 50 chars)
        self.message
            .chars()
            .take(50)
            .collect::<String>()
            .hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

/// Result of analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub files: Vec<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
}

impl AnalysisResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, file: PathBuf) {
        if !self.files.contains(&file) {
            self.files.push(file);
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    pub fn merge(&mut self, other: AnalysisResult) {
        for file in other.files {
            self.add_file(file);
        }
        self.diagnostics.extend(other.diagnostics);
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    // === Counts by new severity levels ===

    pub fn blocker_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Blocker)
            .count()
    }

    pub fn high_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::High)
            .count()
    }

    pub fn medium_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Medium)
            .count()
    }

    pub fn low_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Low)
            .count()
    }

    pub fn info_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .count()
    }

    // === Legacy counts (for backwards compatibility) ===

    /// Legacy: count of High + Blocker (was "error")
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity >= Severity::High)
            .count()
    }

    /// Legacy: count of Medium (was "warning")
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Medium)
            .count()
    }

    // === Counts by issue type ===

    pub fn bug_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.issue_type == IssueType::Bug)
            .count()
    }

    pub fn vulnerability_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.issue_type == IssueType::Vulnerability)
            .count()
    }

    pub fn code_smell_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.issue_type == IssueType::CodeSmell)
            .count()
    }

    pub fn security_hotspot_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.issue_type == IssueType::SecurityHotspot)
            .count()
    }

    pub fn secret_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.issue_type == IssueType::Secret)
            .count()
    }

    // === Technical debt ===

    /// Total effort in minutes to fix all issues
    pub fn total_effort_minutes(&self) -> u32 {
        self.diagnostics
            .iter()
            .filter_map(|d| d.effort_minutes)
            .sum()
    }

    /// Total effort formatted as human-readable string
    pub fn total_effort_display(&self) -> String {
        let minutes = self.total_effort_minutes();
        if minutes == 0 {
            return "0min".to_string();
        }
        let hours = minutes / 60;
        let remaining_minutes = minutes % 60;
        let days = hours / 8; // 8-hour workday
        let remaining_hours = hours % 8;

        if days > 0 {
            format!("{}d {}h {}min", days, remaining_hours, remaining_minutes)
        } else if hours > 0 {
            format!("{}h {}min", hours, remaining_minutes)
        } else {
            format!("{}min", minutes)
        }
    }

    // === Filtering ===

    pub fn filter_by_severity(&mut self, min_severity: Severity) {
        self.diagnostics.retain(|d| d.severity >= min_severity);
    }

    pub fn filter_by_category(&mut self, categories: &[Category]) {
        self.diagnostics
            .retain(|d| categories.contains(&d.category));
    }

    pub fn filter_by_issue_type(&mut self, issue_types: &[IssueType]) {
        self.diagnostics
            .retain(|d| issue_types.contains(&d.issue_type));
    }

    pub fn filter_by_tag(&mut self, tag: &str) {
        self.diagnostics.retain(|d| d.tags.iter().any(|t| t == tag));
    }

    pub fn sort(&mut self) {
        self.diagnostics.sort_by(|a, b| {
            a.location
                .file
                .cmp(&b.location.file)
                .then(
                    a.location
                        .range
                        .start
                        .line
                        .cmp(&b.location.range.start.line),
                )
                .then(
                    a.location
                        .range
                        .start
                        .character
                        .cmp(&b.location.range.start.character),
                )
        });
    }
}

/// Reference kind for symbol navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReferenceKind {
    ComponentRef,
    ComponentGroupRef,
    DirectoryRef,
    FeatureRef,
    FeatureGroupRef,
    PropertyRef,
    CustomActionRef,
    BinaryRef,
}

impl ReferenceKind {
    pub fn from_element_name(name: &str) -> Option<Self> {
        match name {
            "ComponentRef" => Some(Self::ComponentRef),
            "ComponentGroupRef" => Some(Self::ComponentGroupRef),
            "DirectoryRef" => Some(Self::DirectoryRef),
            "FeatureRef" => Some(Self::FeatureRef),
            "FeatureGroupRef" => Some(Self::FeatureGroupRef),
            "PropertyRef" => Some(Self::PropertyRef),
            "CustomActionRef" => Some(Self::CustomActionRef),
            "BinaryRef" => Some(Self::BinaryRef),
            _ => None,
        }
    }

    pub fn element_name(&self) -> &'static str {
        match self {
            Self::ComponentRef => "ComponentRef",
            Self::ComponentGroupRef => "ComponentGroupRef",
            Self::DirectoryRef => "DirectoryRef",
            Self::FeatureRef => "FeatureRef",
            Self::FeatureGroupRef => "FeatureGroupRef",
            Self::PropertyRef => "PropertyRef",
            Self::CustomActionRef => "CustomActionRef",
            Self::BinaryRef => "BinaryRef",
        }
    }

    pub fn definition_element(&self) -> &'static str {
        match self {
            Self::ComponentRef | Self::ComponentGroupRef => "Component",
            Self::DirectoryRef => "Directory",
            Self::FeatureRef | Self::FeatureGroupRef => "Feature",
            Self::PropertyRef => "Property",
            Self::CustomActionRef => "CustomAction",
            Self::BinaryRef => "Binary",
        }
    }
}

/// Definition kind for symbol navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefinitionKind {
    Component,
    ComponentGroup,
    Directory,
    StandardDirectory,
    Feature,
    FeatureGroup,
    Property,
    CustomAction,
    Binary,
    Fragment,
    Package,
    Module,
    Bundle,
}

impl DefinitionKind {
    pub fn from_element_name(name: &str) -> Option<Self> {
        match name {
            "Component" => Some(Self::Component),
            "ComponentGroup" => Some(Self::ComponentGroup),
            "Directory" => Some(Self::Directory),
            "StandardDirectory" => Some(Self::StandardDirectory),
            "Feature" => Some(Self::Feature),
            "FeatureGroup" => Some(Self::FeatureGroup),
            "Property" => Some(Self::Property),
            "CustomAction" => Some(Self::CustomAction),
            "Binary" => Some(Self::Binary),
            "Fragment" => Some(Self::Fragment),
            "Package" => Some(Self::Package),
            "Module" => Some(Self::Module),
            "Bundle" => Some(Self::Bundle),
            _ => None,
        }
    }

    pub fn element_name(&self) -> &'static str {
        match self {
            Self::Component => "Component",
            Self::ComponentGroup => "ComponentGroup",
            Self::Directory => "Directory",
            Self::StandardDirectory => "StandardDirectory",
            Self::Feature => "Feature",
            Self::FeatureGroup => "FeatureGroup",
            Self::Property => "Property",
            Self::CustomAction => "CustomAction",
            Self::Binary => "Binary",
            Self::Fragment => "Fragment",
            Self::Package => "Package",
            Self::Module => "Module",
            Self::Bundle => "Bundle",
        }
    }

    pub fn canonical_type(&self) -> &'static str {
        match self {
            Self::Component | Self::ComponentGroup => "Component",
            Self::Directory | Self::StandardDirectory => "Directory",
            Self::Feature | Self::FeatureGroup => "Feature",
            Self::Property => "Property",
            Self::CustomAction => "CustomAction",
            Self::Binary => "Binary",
            Self::Fragment => "Fragment",
            Self::Package | Self::Module | Self::Bundle => "Package",
        }
    }

    pub fn id_attribute(&self) -> &'static str {
        match self {
            Self::Package | Self::Bundle => "Name",
            _ => "Id",
        }
    }
}

/// A symbol definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDefinition {
    pub id: String,
    pub kind: DefinitionKind,
    pub location: Location,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl SymbolDefinition {
    pub fn new(id: impl Into<String>, kind: DefinitionKind, location: Location) -> Self {
        Self {
            id: id.into(),
            kind,
            location,
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// A symbol reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolReference {
    pub id: String,
    pub kind: ReferenceKind,
    pub location: Location,
}

impl SymbolReference {
    pub fn new(id: impl Into<String>, kind: ReferenceKind, location: Location) -> Self {
        Self {
            id: id.into(),
            kind,
            location,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let pos = Position::new(1, 5);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_range_from_offsets() {
        let source = "line1\nline2\nline3";
        let range = Range::from_offsets(source, 6, 11);
        assert_eq!(range.start.line, 2);
        assert_eq!(range.start.character, 1);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Low);
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Blocker);
    }

    #[test]
    fn test_severity_as_str() {
        assert_eq!(Severity::Blocker.as_str(), "blocker");
        assert_eq!(Severity::High.as_str(), "high");
        assert_eq!(Severity::Medium.as_str(), "medium");
        assert_eq!(Severity::Low.as_str(), "low");
        assert_eq!(Severity::Info.as_str(), "info");
    }

    #[test]
    fn test_severity_to_lsp() {
        assert_eq!(Severity::Blocker.to_lsp_severity(), 1); // Error
        assert_eq!(Severity::High.to_lsp_severity(), 1); // Error
        assert_eq!(Severity::Medium.to_lsp_severity(), 2); // Warning
        assert_eq!(Severity::Low.to_lsp_severity(), 3); // Info
        assert_eq!(Severity::Info.to_lsp_severity(), 4); // Hint
    }

    #[test]
    fn test_severity_from_legacy() {
        assert_eq!(Severity::from_legacy("error"), Some(Severity::High));
        assert_eq!(Severity::from_legacy("warning"), Some(Severity::Medium));
        assert_eq!(Severity::from_legacy("info"), Some(Severity::Info));
        assert_eq!(Severity::from_legacy("blocker"), Some(Severity::Blocker));
        assert_eq!(Severity::from_legacy("unknown"), None);
    }

    #[test]
    fn test_issue_type_as_str() {
        assert_eq!(IssueType::Bug.as_str(), "bug");
        assert_eq!(IssueType::Vulnerability.as_str(), "vulnerability");
        assert_eq!(IssueType::CodeSmell.as_str(), "code_smell");
        assert_eq!(IssueType::SecurityHotspot.as_str(), "security_hotspot");
        assert_eq!(IssueType::Secret.as_str(), "secret");
    }

    #[test]
    fn test_issue_type_display_name() {
        assert_eq!(IssueType::Bug.display_name(), "Bug");
        assert_eq!(IssueType::CodeSmell.display_name(), "Code Smell");
        assert_eq!(
            IssueType::SecurityHotspot.display_name(),
            "Security Hotspot"
        );
    }

    #[test]
    fn test_category_to_issue_type() {
        assert_eq!(Category::Validation.to_issue_type(), IssueType::Bug);
        assert_eq!(Category::BestPractice.to_issue_type(), IssueType::CodeSmell);
        assert_eq!(Category::Security.to_issue_type(), IssueType::Vulnerability);
        assert_eq!(Category::DeadCode.to_issue_type(), IssueType::CodeSmell);
    }

    #[test]
    fn test_category_as_str() {
        assert_eq!(Category::Validation.as_str(), "validation");
        assert_eq!(Category::BestPractice.as_str(), "best-practice");
        assert_eq!(Category::Security.as_str(), "security");
        assert_eq!(Category::DeadCode.as_str(), "dead-code");
    }

    #[test]
    fn test_diagnostic_creation() {
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        let diag = Diagnostic::error("VAL-001", Category::Validation, "Test error", location)
            .with_help("Fix this issue");

        assert_eq!(diag.rule_id, "VAL-001");
        assert_eq!(diag.severity, Severity::High); // error() maps to High
        assert_eq!(diag.issue_type, IssueType::Bug); // Validation maps to Bug
        assert!(diag.help.is_some());
    }

    #[test]
    fn test_diagnostic_with_issue_type() {
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        let diag = Diagnostic::blocker(
            "SEC-001",
            IssueType::Vulnerability,
            "Critical vuln",
            location.clone(),
        );
        assert_eq!(diag.severity, Severity::Blocker);
        assert_eq!(diag.issue_type, IssueType::Vulnerability);

        let diag2 = Diagnostic::high("BUG-001", IssueType::Bug, "Bug found", location.clone());
        assert_eq!(diag2.severity, Severity::High);
        assert_eq!(diag2.issue_type, IssueType::Bug);

        let diag3 = Diagnostic::medium(
            "CS-001",
            IssueType::CodeSmell,
            "Code smell",
            location.clone(),
        );
        assert_eq!(diag3.severity, Severity::Medium);
        assert_eq!(diag3.issue_type, IssueType::CodeSmell);

        let diag4 = Diagnostic::low("CS-002", IssueType::CodeSmell, "Minor smell", location);
        assert_eq!(diag4.severity, Severity::Low);
    }

    #[test]
    fn test_diagnostic_info() {
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        let diag = Diagnostic::info("INFO-001", Category::BestPractice, "Info message", location);
        assert_eq!(diag.severity, Severity::Info);
        assert_eq!(diag.issue_type, IssueType::CodeSmell); // BestPractice maps to CodeSmell
    }

    #[test]
    fn test_diagnostic_with_effort_and_tags() {
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        let diag = Diagnostic::high("SEC-001", IssueType::Vulnerability, "Vuln", location)
            .with_effort(30)
            .with_tag("security")
            .with_tags(["owasp", "cwe-123"]);

        assert_eq!(diag.effort_minutes, Some(30));
        assert_eq!(diag.tags.len(), 3);
        assert!(diag.tags.contains(&"security".to_string()));
        assert!(diag.tags.contains(&"owasp".to_string()));
    }

    #[test]
    fn test_diagnostic_with_related() {
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        let related_loc = Location::new(
            PathBuf::from("other.wxs"),
            Range::new(Position::new(5, 1), Position::new(5, 20)),
        );

        let diag = Diagnostic::error("VAL-001", Category::Validation, "Error", location)
            .with_related(RelatedInfo::new(related_loc, "Related location"));

        assert_eq!(diag.related.len(), 1);
        assert_eq!(diag.related[0].message, "Related location");
    }

    #[test]
    fn test_analysis_result() {
        let mut result = AnalysisResult::new();
        result.add_file(PathBuf::from("test.wxs"));

        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        result.add(Diagnostic::error(
            "VAL-001",
            Category::Validation,
            "Error",
            location.clone(),
        ));
        result.add(Diagnostic::warning(
            "BP-001",
            Category::BestPractice,
            "Warning",
            location,
        ));

        assert_eq!(result.len(), 2);
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 1);
    }

    #[test]
    fn test_analysis_result_info_count() {
        let mut result = AnalysisResult::new();
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        result.add(Diagnostic::info(
            "INFO-001",
            Category::BestPractice,
            "Info",
            location,
        ));
        assert_eq!(result.info_count(), 1);
    }

    #[test]
    fn test_analysis_result_is_empty() {
        let result = AnalysisResult::new();
        assert!(result.is_empty());
    }

    #[test]
    fn test_analysis_result_extend() {
        let mut result = AnalysisResult::new();
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        let diagnostics = vec![
            Diagnostic::error("VAL-001", Category::Validation, "Error 1", location.clone()),
            Diagnostic::error("VAL-002", Category::Validation, "Error 2", location),
        ];
        result.extend(diagnostics);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_analysis_result_add_file_duplicate() {
        let mut result = AnalysisResult::new();
        result.add_file(PathBuf::from("test.wxs"));
        result.add_file(PathBuf::from("test.wxs")); // Duplicate
        assert_eq!(result.files.len(), 1);
    }

    #[test]
    fn test_analysis_result_filter_by_severity() {
        let mut result = AnalysisResult::new();
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        result.add(Diagnostic::info(
            "INFO-001",
            Category::BestPractice,
            "Info",
            location.clone(),
        ));
        result.add(Diagnostic::warning(
            "WARN-001",
            Category::BestPractice,
            "Warn",
            location.clone(),
        ));
        result.add(Diagnostic::error(
            "ERR-001",
            Category::Validation,
            "Error",
            location,
        ));

        result.filter_by_severity(Severity::Medium);
        assert_eq!(result.len(), 2); // Only Medium (warning) and High (error) remain
    }

    #[test]
    fn test_analysis_result_filter_by_issue_type() {
        let mut result = AnalysisResult::new();
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        result.add(Diagnostic::high(
            "BUG-001",
            IssueType::Bug,
            "Bug",
            location.clone(),
        ));
        result.add(Diagnostic::high(
            "SEC-001",
            IssueType::Vulnerability,
            "Vuln",
            location.clone(),
        ));
        result.add(Diagnostic::medium(
            "CS-001",
            IssueType::CodeSmell,
            "Smell",
            location,
        ));

        result.filter_by_issue_type(&[IssueType::Bug, IssueType::Vulnerability]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_analysis_result_counts_by_issue_type() {
        let mut result = AnalysisResult::new();
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        result.add(Diagnostic::high(
            "BUG-001",
            IssueType::Bug,
            "Bug",
            location.clone(),
        ));
        result.add(Diagnostic::high(
            "SEC-001",
            IssueType::Vulnerability,
            "Vuln",
            location.clone(),
        ));
        result.add(Diagnostic::medium(
            "CS-001",
            IssueType::CodeSmell,
            "Smell",
            location,
        ));

        assert_eq!(result.bug_count(), 1);
        assert_eq!(result.vulnerability_count(), 1);
        assert_eq!(result.code_smell_count(), 1);
        assert_eq!(result.security_hotspot_count(), 0);
        assert_eq!(result.secret_count(), 0);
    }

    #[test]
    fn test_analysis_result_effort() {
        let mut result = AnalysisResult::new();
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        result.add(
            Diagnostic::high("BUG-001", IssueType::Bug, "Bug", location.clone()).with_effort(30),
        );
        result.add(
            Diagnostic::high("BUG-002", IssueType::Bug, "Bug2", location.clone()).with_effort(60),
        );
        result.add(Diagnostic::medium(
            "CS-001",
            IssueType::CodeSmell,
            "Smell",
            location,
        )); // No effort

        assert_eq!(result.total_effort_minutes(), 90);
        assert_eq!(result.total_effort_display(), "1h 30min");
    }

    #[test]
    fn test_analysis_result_effort_display_formats() {
        let mut result = AnalysisResult::new();
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        // Test 0 minutes
        assert_eq!(result.total_effort_display(), "0min");

        // Test just minutes
        result.add(Diagnostic::high("B-1", IssueType::Bug, "B", location.clone()).with_effort(45));
        assert_eq!(result.total_effort_display(), "45min");

        // Test hours + minutes (add 75 more = 120 total = 2h)
        result.add(Diagnostic::high("B-2", IssueType::Bug, "B", location.clone()).with_effort(75));
        assert_eq!(result.total_effort_display(), "2h 0min");

        // Test days (add 8h * 60 = 480 = 600 total = 10h = 1d 2h)
        result.add(Diagnostic::high("B-3", IssueType::Bug, "B", location).with_effort(480));
        assert_eq!(result.total_effort_display(), "1d 2h 0min");
    }

    #[test]
    fn test_analysis_result_filter_by_category() {
        let mut result = AnalysisResult::new();
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        result.add(Diagnostic::error(
            "VAL-001",
            Category::Validation,
            "Validation",
            location.clone(),
        ));
        result.add(Diagnostic::warning(
            "SEC-001",
            Category::Security,
            "Security",
            location,
        ));

        result.filter_by_category(&[Category::Security]);
        assert_eq!(result.len(), 1);
        assert_eq!(result.diagnostics[0].category, Category::Security);
    }

    #[test]
    fn test_analysis_result_sort() {
        let mut result = AnalysisResult::new();
        let loc_b = Location::new(
            PathBuf::from("b.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );
        let loc_a = Location::new(
            PathBuf::from("a.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        result.add(Diagnostic::error(
            "VAL-001",
            Category::Validation,
            "Error in b",
            loc_b,
        ));
        result.add(Diagnostic::error(
            "VAL-002",
            Category::Validation,
            "Error in a",
            loc_a,
        ));

        result.sort();
        assert_eq!(
            result.diagnostics[0].location.file.to_str().unwrap(),
            "a.wxs"
        );
    }

    #[test]
    fn test_reference_kind() {
        assert_eq!(
            ReferenceKind::from_element_name("ComponentRef"),
            Some(ReferenceKind::ComponentRef)
        );
        assert_eq!(
            ReferenceKind::ComponentRef.definition_element(),
            "Component"
        );
    }

    #[test]
    fn test_reference_kind_all_variants() {
        // Test all element names
        assert_eq!(
            ReferenceKind::from_element_name("ComponentGroupRef"),
            Some(ReferenceKind::ComponentGroupRef)
        );
        assert_eq!(
            ReferenceKind::from_element_name("DirectoryRef"),
            Some(ReferenceKind::DirectoryRef)
        );
        assert_eq!(
            ReferenceKind::from_element_name("FeatureRef"),
            Some(ReferenceKind::FeatureRef)
        );
        assert_eq!(
            ReferenceKind::from_element_name("FeatureGroupRef"),
            Some(ReferenceKind::FeatureGroupRef)
        );
        assert_eq!(
            ReferenceKind::from_element_name("PropertyRef"),
            Some(ReferenceKind::PropertyRef)
        );
        assert_eq!(
            ReferenceKind::from_element_name("CustomActionRef"),
            Some(ReferenceKind::CustomActionRef)
        );
        assert_eq!(
            ReferenceKind::from_element_name("BinaryRef"),
            Some(ReferenceKind::BinaryRef)
        );
        assert_eq!(ReferenceKind::from_element_name("Unknown"), None);

        // Test element_name for all variants
        assert_eq!(ReferenceKind::ComponentRef.element_name(), "ComponentRef");
        assert_eq!(
            ReferenceKind::ComponentGroupRef.element_name(),
            "ComponentGroupRef"
        );
        assert_eq!(ReferenceKind::DirectoryRef.element_name(), "DirectoryRef");
        assert_eq!(ReferenceKind::FeatureRef.element_name(), "FeatureRef");
        assert_eq!(
            ReferenceKind::FeatureGroupRef.element_name(),
            "FeatureGroupRef"
        );
        assert_eq!(ReferenceKind::PropertyRef.element_name(), "PropertyRef");
        assert_eq!(
            ReferenceKind::CustomActionRef.element_name(),
            "CustomActionRef"
        );
        assert_eq!(ReferenceKind::BinaryRef.element_name(), "BinaryRef");

        // Test definition_element for all variants
        assert_eq!(
            ReferenceKind::ComponentGroupRef.definition_element(),
            "Component"
        );
        assert_eq!(
            ReferenceKind::DirectoryRef.definition_element(),
            "Directory"
        );
        assert_eq!(ReferenceKind::FeatureRef.definition_element(), "Feature");
        assert_eq!(
            ReferenceKind::FeatureGroupRef.definition_element(),
            "Feature"
        );
        assert_eq!(ReferenceKind::PropertyRef.definition_element(), "Property");
        assert_eq!(
            ReferenceKind::CustomActionRef.definition_element(),
            "CustomAction"
        );
        assert_eq!(ReferenceKind::BinaryRef.definition_element(), "Binary");
    }

    #[test]
    fn test_definition_kind() {
        assert_eq!(
            DefinitionKind::from_element_name("Component"),
            Some(DefinitionKind::Component)
        );
        assert_eq!(DefinitionKind::Component.canonical_type(), "Component");
        assert_eq!(DefinitionKind::ComponentGroup.canonical_type(), "Component");
    }

    #[test]
    fn test_definition_kind_all_variants() {
        // Test all from_element_name
        assert_eq!(
            DefinitionKind::from_element_name("ComponentGroup"),
            Some(DefinitionKind::ComponentGroup)
        );
        assert_eq!(
            DefinitionKind::from_element_name("Directory"),
            Some(DefinitionKind::Directory)
        );
        assert_eq!(
            DefinitionKind::from_element_name("StandardDirectory"),
            Some(DefinitionKind::StandardDirectory)
        );
        assert_eq!(
            DefinitionKind::from_element_name("Feature"),
            Some(DefinitionKind::Feature)
        );
        assert_eq!(
            DefinitionKind::from_element_name("FeatureGroup"),
            Some(DefinitionKind::FeatureGroup)
        );
        assert_eq!(
            DefinitionKind::from_element_name("Property"),
            Some(DefinitionKind::Property)
        );
        assert_eq!(
            DefinitionKind::from_element_name("CustomAction"),
            Some(DefinitionKind::CustomAction)
        );
        assert_eq!(
            DefinitionKind::from_element_name("Binary"),
            Some(DefinitionKind::Binary)
        );
        assert_eq!(
            DefinitionKind::from_element_name("Fragment"),
            Some(DefinitionKind::Fragment)
        );
        assert_eq!(
            DefinitionKind::from_element_name("Package"),
            Some(DefinitionKind::Package)
        );
        assert_eq!(
            DefinitionKind::from_element_name("Module"),
            Some(DefinitionKind::Module)
        );
        assert_eq!(
            DefinitionKind::from_element_name("Bundle"),
            Some(DefinitionKind::Bundle)
        );
        assert_eq!(DefinitionKind::from_element_name("Unknown"), None);

        // Test element_name for all variants
        assert_eq!(DefinitionKind::Component.element_name(), "Component");
        assert_eq!(
            DefinitionKind::ComponentGroup.element_name(),
            "ComponentGroup"
        );
        assert_eq!(DefinitionKind::Directory.element_name(), "Directory");
        assert_eq!(
            DefinitionKind::StandardDirectory.element_name(),
            "StandardDirectory"
        );
        assert_eq!(DefinitionKind::Feature.element_name(), "Feature");
        assert_eq!(DefinitionKind::FeatureGroup.element_name(), "FeatureGroup");
        assert_eq!(DefinitionKind::Property.element_name(), "Property");
        assert_eq!(DefinitionKind::CustomAction.element_name(), "CustomAction");
        assert_eq!(DefinitionKind::Binary.element_name(), "Binary");
        assert_eq!(DefinitionKind::Fragment.element_name(), "Fragment");
        assert_eq!(DefinitionKind::Package.element_name(), "Package");
        assert_eq!(DefinitionKind::Module.element_name(), "Module");
        assert_eq!(DefinitionKind::Bundle.element_name(), "Bundle");

        // Test canonical_type for all variants
        assert_eq!(DefinitionKind::Directory.canonical_type(), "Directory");
        assert_eq!(
            DefinitionKind::StandardDirectory.canonical_type(),
            "Directory"
        );
        assert_eq!(DefinitionKind::Feature.canonical_type(), "Feature");
        assert_eq!(DefinitionKind::FeatureGroup.canonical_type(), "Feature");
        assert_eq!(DefinitionKind::Property.canonical_type(), "Property");
        assert_eq!(
            DefinitionKind::CustomAction.canonical_type(),
            "CustomAction"
        );
        assert_eq!(DefinitionKind::Binary.canonical_type(), "Binary");
        assert_eq!(DefinitionKind::Fragment.canonical_type(), "Fragment");
        assert_eq!(DefinitionKind::Package.canonical_type(), "Package");
        assert_eq!(DefinitionKind::Module.canonical_type(), "Package");
        assert_eq!(DefinitionKind::Bundle.canonical_type(), "Package");

        // Test id_attribute
        assert_eq!(DefinitionKind::Component.id_attribute(), "Id");
        assert_eq!(DefinitionKind::Package.id_attribute(), "Name");
        assert_eq!(DefinitionKind::Bundle.id_attribute(), "Name");
    }

    #[test]
    fn test_symbol_definition_with_detail() {
        let location = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );

        let def = SymbolDefinition::new("MyComponent", DefinitionKind::Component, location)
            .with_detail("A test component");

        assert_eq!(def.detail, Some("A test component".to_string()));
    }

    #[test]
    fn test_fix_creation() {
        let range = Range::new(Position::new(1, 1), Position::new(1, 10));
        let fix = Fix::new(
            "Add Id attribute",
            FixAction::AddAttribute {
                range,
                name: "Id".to_string(),
                value: "MyId".to_string(),
            },
        );

        assert_eq!(fix.description, "Add Id attribute");
    }

    #[test]
    fn test_analysis_result_merge_with_files() {
        let mut result1 = AnalysisResult::new();
        result1.add_file(PathBuf::from("test1.wxs"));
        let location1 = Location::new(
            PathBuf::from("test1.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );
        result1.add(Diagnostic::error(
            "VAL-001",
            Category::Validation,
            "Error 1",
            location1,
        ));

        let mut result2 = AnalysisResult::new();
        result2.add_file(PathBuf::from("test2.wxs"));
        let location2 = Location::new(
            PathBuf::from("test2.wxs"),
            Range::new(Position::new(1, 1), Position::new(1, 10)),
        );
        result2.add(Diagnostic::error(
            "VAL-002",
            Category::Validation,
            "Error 2",
            location2,
        ));

        result1.merge(result2);

        assert_eq!(result1.files.len(), 2);
        assert_eq!(result1.len(), 2);
    }

    #[test]
    fn test_analysis_result_sort_by_line() {
        // Test sorting by line when files are the same
        let mut result = AnalysisResult::new();
        let loc_line_10 = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(10, 1), Position::new(10, 10)),
        );
        let loc_line_5 = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(5, 1), Position::new(5, 10)),
        );

        result.add(Diagnostic::error(
            "VAL-001",
            Category::Validation,
            "Error at line 10",
            loc_line_10,
        ));
        result.add(Diagnostic::error(
            "VAL-002",
            Category::Validation,
            "Error at line 5",
            loc_line_5,
        ));

        result.sort();
        assert_eq!(result.diagnostics[0].location.range.start.line, 5);
        assert_eq!(result.diagnostics[1].location.range.start.line, 10);
    }

    #[test]
    fn test_analysis_result_sort_by_character() {
        // Test sorting by character when files and lines are the same
        let mut result = AnalysisResult::new();
        let loc_col_20 = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(5, 20), Position::new(5, 30)),
        );
        let loc_col_5 = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(5, 5), Position::new(5, 15)),
        );

        result.add(Diagnostic::error(
            "VAL-001",
            Category::Validation,
            "Error at col 20",
            loc_col_20,
        ));
        result.add(Diagnostic::error(
            "VAL-002",
            Category::Validation,
            "Error at col 5",
            loc_col_5,
        ));

        result.sort();
        assert_eq!(result.diagnostics[0].location.range.start.character, 5);
        assert_eq!(result.diagnostics[1].location.range.start.character, 20);
    }
}
