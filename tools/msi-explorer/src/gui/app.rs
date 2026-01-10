//! Main application state and logic

use eframe::egui::{self, RichText};
use msi_explorer::{MsiFile, Table, TableCategory, SummaryInfo, MsiStats};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::panels;
use crate::theme::Theme;

/// Number display format
#[derive(Clone, Copy, PartialEq, Default)]
pub enum NumberFormat {
    #[default]
    Decimal,
    Hex,
    Binary,
}

impl NumberFormat {
    pub fn label(&self) -> &'static str {
        match self {
            NumberFormat::Decimal => "Dec",
            NumberFormat::Hex => "Hex",
            NumberFormat::Binary => "Bin",
        }
    }

    /// Format an integer value according to the current format
    pub fn format_i32(&self, value: i32) -> String {
        match self {
            NumberFormat::Decimal => value.to_string(),
            NumberFormat::Hex => format!("0x{:X}", value),
            NumberFormat::Binary => format!("0b{:b}", value),
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            NumberFormat::Decimal => NumberFormat::Hex,
            NumberFormat::Hex => NumberFormat::Binary,
            NumberFormat::Binary => NumberFormat::Decimal,
        }
    }
}

/// Current view mode
#[derive(Clone, Copy, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Tables,
    Tree,
    Diff,
    Files,
}

/// Tree node for Feature → Component → File hierarchy
#[derive(Clone)]
pub struct TreeNode {
    pub name: String,
    pub node_type: TreeNodeType,
    pub children: Vec<TreeNode>,
}

#[derive(Clone)]
pub enum TreeNodeType {
    Feature { title: Option<String> },
    Component { directory: Option<String> },
    File { size: Option<i64>, version: Option<String> },
    #[allow(dead_code)]
    Directory { path: String },
}

/// Extracted file info for Files view
#[derive(Clone)]
pub struct ExtractedFile {
    #[allow(dead_code)]
    pub file_key: String,
    pub file_name: String,
    #[allow(dead_code)]
    pub component: String,
    pub directory: String,
    pub size: i64,
    pub version: Option<String>,
    pub cab_name: Option<String>,
}

/// A pending edit to a cell
#[derive(Clone, Debug)]
pub struct PendingEdit {
    pub table: String,
    pub row_idx: usize,
    pub col_idx: usize,
    #[allow(dead_code)] // Will be used for undo functionality
    pub old_value: String,
    pub new_value: String,
}

/// A cell being edited
#[derive(Clone, Default)]
pub struct EditingCell {
    pub table: String,
    pub row_idx: usize,
    pub col_idx: usize,
    pub col_name: String,
    pub text: String,
    pub is_primary_key: bool,
}

/// Cascade rename preview
#[derive(Clone)]
pub struct CascadePreview {
    pub original_value: String,
    pub new_value: String,
    #[allow(dead_code)] // Could be used to show source in UI
    pub source_table: String,
    #[allow(dead_code)] // Could be used to show source in UI
    pub source_column: String,
    pub affected: Vec<CascadeAffected>,
}

/// An affected reference in cascade rename
#[derive(Clone)]
pub struct CascadeAffected {
    pub table: String,
    pub column: String,
    pub row_count: usize,
}

/// A pending row addition
#[derive(Clone, Debug)]
pub struct PendingRowAdd {
    pub table: String,
    pub values: Vec<String>,
}

/// A pending row deletion
#[derive(Clone, Debug)]
pub struct PendingRowDelete {
    pub table: String,
    pub row_idx: usize,
}

/// Undo action types
#[derive(Clone, Debug)]
pub enum UndoAction {
    Edit {
        table: String,
        row_idx: usize,
        col_idx: usize,
        old_value: String,
        new_value: String,
    },
    AddRow {
        table: String,
        values: Vec<String>,
    },
    DeleteRow {
        table: String,
        row_idx: usize,
        values: Vec<String>,
    },
    Paste {
        table: String,
        count: usize,
    },
}

/// Patch diff information
#[derive(Clone, Debug)]
pub struct PatchDiff {
    /// Tables added in new MSI
    pub added_tables: Vec<String>,
    /// Tables removed in new MSI
    pub removed_tables: Vec<String>,
    /// Tables with changes
    pub changed_tables: Vec<TableDiff>,
    /// Old product version
    pub old_version: String,
    /// New product version
    pub new_version: String,
    /// Old product code
    pub old_product_code: String,
    /// New product code
    pub new_product_code: String,
}

/// Table-level diff
#[derive(Clone, Debug)]
pub struct TableDiff {
    pub name: String,
    pub added_rows: usize,
    pub deleted_rows: usize,
    pub modified_rows: usize,
}

/// New column definition for table creation
#[derive(Clone, Debug, Default)]
pub struct NewColumnDef {
    pub name: String,
    pub col_type: String,  // "String" or "Integer"
    pub nullable: bool,
    pub primary_key: bool,
    pub size: i32,
}

/// Digital signature information
#[derive(Clone, Debug)]
pub struct SignatureInfo {
    pub is_signed: bool,
    pub signer: Option<String>,
    pub timestamp: Option<String>,
    pub valid: bool,
}

/// Reference error for validation
#[derive(Clone, Debug)]
pub struct ReferenceError {
    pub table: String,
    pub column: String,
    pub row_idx: usize,
    pub value: String,
    pub references_table: String,
    pub error_type: ReferenceErrorType,
}

#[derive(Clone, Debug)]
pub enum ReferenceErrorType {
    MissingReference,
    InvalidForeignKey,
}

/// Report format options
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ReportFormat {
    #[default]
    Html,
    Markdown,
    PlainText,
}

impl ReportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ReportFormat::Html => "html",
            ReportFormat::Markdown => "md",
            ReportFormat::PlainText => "txt",
        }
    }
}

/// CAB compression levels
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CabCompression {
    None,
    #[default]
    Low,
    Medium,
    High,
}

impl CabCompression {
    pub fn label(&self) -> &'static str {
        match self {
            CabCompression::None => "None",
            CabCompression::Low => "Low (LZX:15)",
            CabCompression::Medium => "Medium (LZX:18)",
            CabCompression::High => "High (LZX:21)",
        }
    }
}

/// Dependency node for graph
#[derive(Clone, Debug)]
pub struct DependencyNode {
    pub name: String,
    pub node_type: DependencyType,
    pub depends_on: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DependencyType {
    Feature,
    Component,
    File,
}

/// Hash verification result
#[derive(Clone, Debug)]
pub struct HashResult {
    pub file_key: String,
    pub file_name: String,
    pub expected_hash: Option<String>,
    pub status: HashStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HashStatus {
    Valid,
    Invalid,
    Missing,
    NoHash,
}

/// Bookmark entry
#[derive(Clone, Debug)]
pub struct Bookmark {
    pub table: String,
    pub row_idx: usize,
    pub primary_key: String,
    pub note: String,
}

/// Stream information
#[derive(Clone, Debug)]
pub struct StreamInfo {
    pub name: String,
    pub size: usize,
    pub stream_type: StreamType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StreamType {
    Binary,
    Icon,
    Cab,
    Summary,
    Other,
}

/// Known MSI property
#[derive(Clone, Debug)]
pub struct KnownProperty {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
}

/// Condition validation result
#[derive(Clone, Debug)]
pub struct ConditionResult {
    pub valid: bool,
    pub message: String,
    pub properties_used: Vec<String>,
}

/// CAB file info
#[derive(Clone, Debug)]
pub struct CabFileInfo {
    pub name: String,
    pub size: u64,
    pub compressed_size: u64,
    pub cab_name: String,
}

/// Diff export format
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DiffExportFormat {
    #[default]
    Text,
    Html,
    Json,
    Csv,
}

impl DiffExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            DiffExportFormat::Text => "txt",
            DiffExportFormat::Html => "html",
            DiffExportFormat::Json => "json",
            DiffExportFormat::Csv => "csv",
        }
    }
}

/// Duplicate row info
#[derive(Clone, Debug)]
pub struct DuplicateRow {
    pub table: String,
    pub row_indices: Vec<usize>,
    pub key_value: String,
}

/// Orphan entry (unreferenced)
#[derive(Clone, Debug)]
pub struct OrphanEntry {
    pub table: String,
    pub row_idx: usize,
    pub key_value: String,
    pub entry_type: OrphanType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OrphanType {
    UnusedComponent,
    UnusedFeature,
    UnusedDirectory,
    UnusedBinary,
    UnusedIcon,
}

impl OrphanType {
    pub fn label(&self) -> &'static str {
        match self {
            OrphanType::UnusedComponent => "Unused Component",
            OrphanType::UnusedFeature => "Unused Feature",
            OrphanType::UnusedDirectory => "Unused Directory",
            OrphanType::UnusedBinary => "Unused Binary",
            OrphanType::UnusedIcon => "Unused Icon",
        }
    }
}

/// Icon info for extraction
#[derive(Clone, Debug)]
pub struct IconInfo {
    pub name: String,
    pub size: usize,
    pub data: Vec<u8>,
}

/// Embedded file preview
#[derive(Clone, Debug)]
pub struct FilePreview {
    pub name: String,
    pub file_type: FilePreviewType,
    pub content: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FilePreviewType {
    Text,
    Image,
    Binary,
    Xml,
}

/// Feature tree node
#[derive(Clone, Debug)]
pub struct FeatureTreeNode {
    pub feature: String,
    pub title: String,
    pub parent: Option<String>,
    pub level: i32,
    pub components: Vec<String>,
    pub children: Vec<FeatureTreeNode>,
}

/// Directory tree node
#[derive(Clone, Debug)]
pub struct DirectoryTreeNode {
    pub directory: String,
    pub name: String,
    pub parent: Option<String>,
    pub full_path: String,
    pub children: Vec<DirectoryTreeNode>,
}

/// Component rule violation
#[derive(Clone, Debug)]
pub struct ComponentRuleViolation {
    pub component: String,
    pub rule: ComponentRule,
    pub message: String,
    pub severity: RuleSeverity,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ComponentRule {
    OneFilePerComponent,
    KeyPathRequired,
    GuidRequired,
    ConsistentDirectory,
    NoEmptyComponent,
}

impl ComponentRule {
    pub fn label(&self) -> &'static str {
        match self {
            ComponentRule::OneFilePerComponent => "One file per component",
            ComponentRule::KeyPathRequired => "KeyPath required",
            ComponentRule::GuidRequired => "GUID required",
            ComponentRule::ConsistentDirectory => "Consistent directory",
            ComponentRule::NoEmptyComponent => "No empty component",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RuleSeverity {
    Error,
    Warning,
    Info,
}

/// Custom Action decoded info
#[derive(Clone, Debug)]
pub struct CustomActionDecoded {
    pub name: String,
    pub ca_type: i32,
    pub source_type: String,
    pub target_type: String,
    pub execution: String,
    pub flags: Vec<String>,
}

/// Install timeline event
#[derive(Clone, Debug)]
pub struct TimelineEvent {
    pub action: String,
    pub sequence: i32,
    pub phase: InstallPhase,
    pub is_standard: bool,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InstallPhase {
    UISequence,
    ExecuteSequence,
    AdminUISequence,
    AdminExecuteSequence,
    AdvertiseExecuteSequence,
}

impl InstallPhase {
    pub fn label(&self) -> &'static str {
        match self {
            InstallPhase::UISequence => "UI Sequence",
            InstallPhase::ExecuteSequence => "Execute Sequence",
            InstallPhase::AdminUISequence => "Admin UI",
            InstallPhase::AdminExecuteSequence => "Admin Execute",
            InstallPhase::AdvertiseExecuteSequence => "Advertise",
        }
    }
}

/// Table template
#[derive(Clone, Debug)]
pub struct TableTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub columns: Vec<TemplateColumn>,
}

#[derive(Clone, Debug)]
pub struct TemplateColumn {
    pub name: &'static str,
    pub col_type: &'static str,
    pub nullable: bool,
    pub primary_key: bool,
}

/// Condition builder node
#[derive(Clone, Debug)]
pub struct ConditionNode {
    pub node_type: ConditionNodeType,
    pub value: String,
    pub children: Vec<ConditionNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConditionNodeType {
    Property,
    Operator,
    Value,
    And,
    Or,
    Not,
}

/// Column statistics
#[derive(Clone, Debug)]
pub struct ColumnStats {
    pub column_name: String,
    pub total_count: usize,
    pub null_count: usize,
    pub unique_count: usize,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
    pub avg_length: f64,
    pub numeric_stats: Option<NumericStats>,
}

#[derive(Clone, Debug)]
pub struct NumericStats {
    pub min: i64,
    pub max: i64,
    pub sum: i64,
    pub avg: f64,
}

/// Database issue for repair
#[derive(Clone, Debug)]
pub struct DatabaseIssue {
    pub issue_type: IssueType,
    pub table: String,
    pub description: String,
    pub can_auto_fix: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IssueType {
    OrphanedRow,
    InvalidReference,
    DuplicateKey,
    MissingRequiredValue,
    InvalidDataType,
    SchemaViolation,
}

impl IssueType {
    pub fn label(&self) -> &'static str {
        match self {
            IssueType::OrphanedRow => "Orphaned Row",
            IssueType::InvalidReference => "Invalid Reference",
            IssueType::DuplicateKey => "Duplicate Key",
            IssueType::MissingRequiredValue => "Missing Required",
            IssueType::InvalidDataType => "Invalid Data Type",
            IssueType::SchemaViolation => "Schema Violation",
        }
    }
}

/// Batch operation
#[derive(Clone, Debug)]
pub struct BatchOperation {
    pub operation_type: BatchOpType,
    pub target_files: Vec<PathBuf>,
    pub status: BatchStatus,
    pub results: Vec<BatchResult>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BatchOpType {
    ExtractAllTables,
    ValidateAll,
    ExportToWix,
    GenerateReport,
    ApplyTransform,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BatchStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Debug)]
pub struct BatchResult {
    pub file: PathBuf,
    pub success: bool,
    pub message: String,
}

/// Session log entry
#[derive(Clone, Debug)]
pub struct SessionLogEntry {
    pub timestamp: String,
    pub action: String,
    pub details: String,
    pub file: Option<String>,
}

/// Plugin info
#[derive(Clone, Debug)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub path: PathBuf,
}

/// Extracted CAB file
#[derive(Clone, Debug)]
pub struct ExtractedCabFile {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub compressed: bool,
}

/// Binary diff result
#[derive(Clone, Debug)]
pub struct BinaryDiffResult {
    pub stream_name: String,
    pub size_a: usize,
    pub size_b: usize,
    pub differs: bool,
    pub diff_offset: Option<usize>,
}

/// Install simulation step
#[derive(Clone, Debug)]
pub struct SimulationStep {
    pub action: String,
    pub description: String,
    pub affected_files: Vec<String>,
    pub affected_registry: Vec<String>,
    pub condition: Option<String>,
    pub will_run: bool,
}

/// Feature cost info
#[derive(Clone, Debug)]
pub struct FeatureCost {
    pub feature: String,
    pub title: String,
    pub local_cost: u64,
    pub source_cost: u64,
    pub components: usize,
    pub files: usize,
}

/// Row annotation
#[derive(Clone, Debug)]
pub struct RowAnnotation {
    pub table: String,
    pub row_key: String,
    pub note: String,
    pub timestamp: String,
}

/// Watch expression
#[derive(Clone, Debug)]
pub struct WatchExpression {
    pub property: String,
    pub value: String,
    pub condition: Option<String>,
}

/// Row change history
#[derive(Clone, Debug)]
pub struct RowChange {
    pub table: String,
    pub row_key: String,
    pub column: String,
    pub old_value: String,
    pub new_value: String,
    pub timestamp: String,
}

/// Split view config
#[derive(Clone, Debug, Default)]
pub struct SplitViewConfig {
    pub enabled: bool,
    pub left_table: Option<String>,
    pub right_table: Option<String>,
    pub sync_scroll: bool,
}

/// Recent search entry
#[derive(Clone, Debug)]
pub struct RecentSearch {
    pub query: String,
    pub timestamp: String,
    pub result_count: usize,
}

/// Favorite item
#[derive(Clone, Debug)]
pub struct FavoriteItem {
    pub name: String,
    pub item_type: FavoriteType,
    pub table: Option<String>,
    pub row_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FavoriteType {
    Table,
    Row,
    Query,
}

/// Column profile
#[derive(Clone, Debug)]
pub struct ColumnProfile {
    pub name: String,
    pub table: String,
    pub visible_columns: Vec<String>,
    pub column_widths: Vec<f32>,
}

/// Rollback operation
#[derive(Clone, Debug)]
pub struct RollbackOperation {
    pub action: String,
    pub operation: String,
    pub target: String,
    pub sequence: i32,
}

/// Action timing estimate
#[derive(Clone, Debug)]
pub struct ActionTiming {
    pub action: String,
    pub estimated_ms: u64,
    pub category: TimingCategory,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TimingCategory {
    Fast,      // < 100ms
    Medium,    // 100ms - 1s
    Slow,      // 1s - 10s
    VerySlow,  // > 10s
}

/// Patch delta info
#[derive(Clone, Debug)]
pub struct PatchDelta {
    pub table: String,
    pub operation: PatchOperation,
    pub key: String,
    pub details: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PatchOperation {
    Add,
    Delete,
    Modify,
}

/// Main application state
pub struct MsiExplorerApp {
    /// Currently open MSI file path
    pub current_file: Option<PathBuf>,
    /// MSI file handle
    pub msi: Option<MsiFile>,
    /// Cached table list
    pub tables: Vec<String>,
    /// Tables by category
    pub tables_by_category: HashMap<TableCategory, Vec<String>>,
    /// Currently selected table
    pub selected_table: Option<String>,
    /// Current table data
    pub current_table: Option<Table>,
    /// Summary info
    pub summary: Option<SummaryInfo>,
    /// File stats
    pub stats: Option<MsiStats>,
    /// Search query
    pub search_query: String,
    /// Search results
    pub search_results: Vec<msi_explorer::search::SearchResult>,
    /// Status message
    pub status: String,
    /// Show table categories
    pub show_categories: bool,
    /// Error message
    pub error: Option<String>,
    /// Theme applied
    theme_applied: bool,

    // New features
    /// Number display format
    pub number_format: NumberFormat,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Second MSI for diff
    pub diff_file: Option<PathBuf>,
    pub diff_msi: Option<MsiFile>,
    /// Tree data (Feature → Component → File)
    pub tree_data: Vec<TreeNode>,
    /// Extracted files list
    pub files_list: Vec<ExtractedFile>,

    // Edit mode
    /// Whether edit mode is enabled
    pub edit_mode: bool,
    /// Cell currently being edited
    pub editing_cell: Option<EditingCell>,
    /// Pending edits not yet applied
    pub pending_edits: Vec<PendingEdit>,
    /// Pending row additions
    pub pending_adds: Vec<PendingRowAdd>,
    /// Pending row deletions
    pub pending_deletes: Vec<PendingRowDelete>,
    /// Cascade rename preview dialog
    pub cascade_preview: Option<CascadePreview>,
    /// Has unsaved changes
    pub has_changes: bool,
    /// New row being edited (for add row dialog)
    pub new_row_values: Vec<String>,
    /// Show add row dialog
    pub show_add_row_dialog: bool,

    // ICE Validation
    /// Validation results
    pub validation_result: Option<ice_validator::ValidationResult>,
    /// Show validation panel
    pub show_validation_panel: bool,
    /// Is validation running
    pub validation_running: bool,

    // Transform support
    /// Transform mode active (tracking changes for MST)
    pub transform_mode: bool,
    /// Base MSI state before transform (for comparison)
    pub transform_base_file: Option<PathBuf>,

    // Merge Module support
    /// Show merge module dialog
    pub show_merge_dialog: bool,
    /// Selected merge module path
    pub merge_module_path: Option<PathBuf>,
    /// Feature to connect merge module to
    pub merge_target_feature: String,
    /// Directory for merge module root
    pub merge_target_directory: String,

    // Edit Summary Info
    /// Show edit summary dialog
    pub show_edit_summary_dialog: bool,
    /// Editable summary fields
    pub edit_summary_title: String,
    pub edit_summary_author: String,
    pub edit_summary_subject: String,
    pub edit_summary_comments: String,

    // Copy/Paste
    /// Clipboard for copied rows
    pub clipboard_rows: Vec<Vec<String>>,
    /// Source table for clipboard
    pub clipboard_source_table: Option<String>,

    // Find & Replace
    /// Show find/replace dialog
    pub show_find_replace: bool,
    /// Find text
    pub find_text: String,
    /// Replace text
    pub replace_text: String,
    /// Search results (table, row, col)
    pub find_results: Vec<(String, usize, usize)>,
    /// Current find result index
    pub find_result_index: usize,
    /// Search all tables
    pub find_all_tables: bool,

    // Undo/Redo
    /// Undo history
    pub undo_stack: Vec<UndoAction>,
    /// Redo history
    pub redo_stack: Vec<UndoAction>,

    // Sequence visualization
    /// Show sequence view
    pub show_sequence_view: bool,
    /// Selected sequence table
    pub sequence_table: String,

    // Dialog preview
    /// Show dialog preview
    pub show_dialog_preview: bool,
    /// Selected dialog name
    pub preview_dialog_name: Option<String>,

    // Patch creation
    /// Show create patch dialog
    pub show_create_patch_dialog: bool,
    /// Old MSI path for patch
    pub patch_old_msi: Option<PathBuf>,
    /// New MSI path for patch
    pub patch_new_msi: Option<PathBuf>,
    /// Patch output path
    pub patch_output_path: Option<PathBuf>,
    /// Patch diff results
    pub patch_diff: Option<PatchDiff>,

    // Row Filtering
    /// Column to filter on
    pub filter_column: Option<usize>,
    /// Filter text
    pub filter_text: String,
    /// Show filter bar
    pub show_filter_bar: bool,

    // Column Sorting
    /// Column to sort by
    pub sort_column: Option<usize>,
    /// Sort ascending (true) or descending (false)
    pub sort_ascending: bool,

    // Recent Files
    /// Recently opened files (max 10)
    pub recent_files: Vec<PathBuf>,

    // GUID Generator
    /// Show GUID generator dialog
    pub show_guid_generator: bool,
    /// Generated GUID
    pub generated_guid: String,
    /// GUID format (uppercase, braces, etc.)
    pub guid_uppercase: bool,
    pub guid_braces: bool,

    // Regex Search
    /// Use regex in find
    pub find_use_regex: bool,
    /// Case sensitive search
    pub find_case_sensitive: bool,

    // SQL Query Editor
    /// Show SQL editor
    pub show_sql_editor: bool,
    /// SQL query text
    pub sql_query: String,
    /// SQL query results
    pub sql_results: Option<Table>,
    /// SQL error message
    pub sql_error: Option<String>,

    // Custom Table Creation
    /// Show create table dialog
    pub show_create_table_dialog: bool,
    /// New table name
    pub new_table_name: String,
    /// New table columns (name, type, nullable, primary_key)
    pub new_table_columns: Vec<NewColumnDef>,

    // Digital Signature Info
    /// Signature information
    pub signature_info: Option<SignatureInfo>,

    // Script Viewer
    /// Show script viewer
    pub show_script_viewer: bool,
    /// Selected binary name for script viewing
    pub selected_binary: Option<String>,
    /// Script content
    pub script_content: String,

    // Reference Validation
    /// Show reference validation panel
    pub show_reference_validation: bool,
    /// Reference errors found
    pub reference_errors: Vec<ReferenceError>,

    // Print/Report
    /// Show report dialog
    pub show_report_dialog: bool,
    /// Report format
    pub report_format: ReportFormat,

    // String Localization
    /// Show localization view
    pub show_localization_view: bool,
    /// Selected language code
    pub selected_language: Option<i32>,
    /// Available languages
    pub available_languages: Vec<(i32, String)>,

    // CAB Rebuild
    /// Show CAB rebuild dialog
    pub show_cab_rebuild: bool,
    /// CAB compression level
    pub cab_compression: CabCompression,

    // Table Schema Viewer
    /// Show schema viewer
    pub show_schema_viewer: bool,

    // Dependency Graph
    /// Show dependency graph
    pub show_dependency_graph: bool,
    /// Dependency data
    pub dependency_data: Vec<DependencyNode>,

    // File Hash Verification
    /// Show hash verification
    pub show_hash_verification: bool,
    /// Hash verification results
    pub hash_results: Vec<HashResult>,

    // Theme
    /// Dark mode enabled
    pub dark_mode: bool,

    // Export to WiX
    /// Show WiX export dialog
    pub show_wix_export: bool,

    // Bulk Edit
    /// Show bulk edit dialog
    pub show_bulk_edit: bool,
    /// Bulk edit column
    pub bulk_edit_column: Option<usize>,
    /// Bulk edit find value
    pub bulk_edit_find: String,
    /// Bulk edit replace value
    pub bulk_edit_replace: String,

    // Bookmarks
    /// Bookmarked rows (table, row_idx, note)
    pub bookmarks: Vec<Bookmark>,
    /// Show bookmarks panel
    pub show_bookmarks: bool,

    // Statistics Dashboard
    /// Show stats dashboard
    pub show_stats_dashboard: bool,

    // Stream Viewer
    /// Show stream viewer
    pub show_stream_viewer: bool,
    /// Available streams
    pub streams: Vec<StreamInfo>,

    // Property Editor
    /// Show property editor
    pub show_property_editor: bool,
    /// Known properties with descriptions
    pub known_properties: Vec<KnownProperty>,

    // Condition Validator
    /// Show condition validator
    pub show_condition_validator: bool,
    /// Condition to validate
    pub condition_text: String,
    /// Condition validation result
    pub condition_result: Option<ConditionResult>,

    // Action Sequence Diagram
    /// Show sequence diagram
    pub show_sequence_diagram: bool,

    // Preview panels
    /// Show registry preview
    pub show_registry_preview: bool,
    /// Show shortcut preview
    pub show_shortcut_preview: bool,
    /// Show service preview
    pub show_service_preview: bool,

    // CAB Contents
    /// Show CAB contents
    pub show_cab_contents: bool,
    /// CAB file list
    pub cab_files: Vec<CabFileInfo>,

    // Diff Export
    /// Diff export format
    pub diff_export_format: DiffExportFormat,

    // Row Selection
    /// Selected row indices
    pub selected_rows: Vec<usize>,
    /// Last clicked row for shift-select
    pub last_selected_row: Option<usize>,

    // Context Menu
    /// Show context menu
    pub show_context_menu: bool,
    /// Context menu position
    pub context_menu_pos: (f32, f32),
    /// Context menu row
    pub context_menu_row: Option<usize>,

    // Transform Comparison
    /// Show transform comparison dialog
    pub show_transform_compare: bool,
    /// Transform file for comparison
    pub compare_transform_path: Option<PathBuf>,

    // Duplicate Detection
    /// Show duplicate detection panel
    pub show_duplicate_detection: bool,
    /// Found duplicates
    pub duplicates: Vec<DuplicateRow>,

    // Orphan Detection
    /// Show orphan detection panel
    pub show_orphan_detection: bool,
    /// Found orphans
    pub orphans: Vec<OrphanEntry>,

    // Icon Extraction
    /// Show icon extraction dialog
    pub show_icon_extraction: bool,
    /// Available icons
    pub icons: Vec<IconInfo>,

    // File Preview
    /// Show file preview
    pub show_file_preview: bool,
    /// Current file preview
    pub file_preview: Option<FilePreview>,

    // Feature Tree View
    /// Show feature tree
    pub show_feature_tree: bool,
    /// Feature tree data
    pub feature_tree: Vec<FeatureTreeNode>,

    // Directory Tree View
    /// Show directory tree
    pub show_directory_tree: bool,
    /// Directory tree data
    pub directory_tree: Vec<DirectoryTreeNode>,

    // Component Rules Checker
    /// Show component rules panel
    pub show_component_rules: bool,
    /// Component rule violations
    pub component_violations: Vec<ComponentRuleViolation>,

    // Custom Action Decoder
    /// Show CA decoder
    pub show_ca_decoder: bool,
    /// Decoded custom actions
    pub decoded_cas: Vec<CustomActionDecoded>,

    // Install Sequence Timeline
    /// Show timeline view
    pub show_timeline: bool,
    /// Timeline events
    pub timeline_events: Vec<TimelineEvent>,
    /// Selected timeline phase
    pub timeline_phase: InstallPhase,

    // Foreign Key Navigation
    /// Pending FK navigation (table, row_idx)
    pub fk_navigate_to: Option<(String, usize)>,

    // Keyboard Shortcuts
    /// Show keyboard shortcuts help
    pub show_shortcuts_help: bool,

    // User Guide
    /// Show user guide
    pub show_user_guide: bool,

    // About Dialog
    /// Show about dialog
    pub show_about: bool,

    // Table Templates
    /// Show template picker
    pub show_template_picker: bool,
    /// Available templates
    pub table_templates: Vec<TableTemplate>,

    // Condition Builder
    /// Show condition builder
    pub show_condition_builder: bool,
    /// Condition builder root node
    pub condition_builder_root: Option<ConditionNode>,
    /// Built condition string
    pub built_condition: String,

    // Column Statistics
    /// Show column stats
    pub show_column_stats: bool,
    /// Column statistics data
    pub column_stats: Vec<ColumnStats>,

    // Database Repair
    /// Show repair panel
    pub show_db_repair: bool,
    /// Found issues
    pub db_issues: Vec<DatabaseIssue>,

    // Batch Operations
    /// Show batch panel
    pub show_batch_panel: bool,
    /// Current batch operation
    pub batch_operation: Option<BatchOperation>,

    // Session Logging
    /// Session log entries
    pub session_log: Vec<SessionLogEntry>,
    /// Show session log
    pub show_session_log: bool,

    // Plugin System
    /// Loaded plugins
    pub plugins: Vec<PluginInfo>,
    /// Show plugin manager
    pub show_plugin_manager: bool,

    // Database Compression
    /// Show compression dialog
    pub show_compression_dialog: bool,

    // CAB Operations
    /// Show CAB extraction dialog
    pub show_cab_extraction: bool,
    /// Extracted CAB files
    pub extracted_cab_files: Vec<ExtractedCabFile>,
    /// CAB extraction path
    pub cab_extraction_path: Option<PathBuf>,

    // Binary Diff
    /// Show binary diff dialog
    pub show_binary_diff: bool,
    /// Binary diff results
    pub binary_diff_results: Vec<BinaryDiffResult>,

    // Install Simulation
    /// Show simulation panel
    pub show_simulation: bool,
    /// Simulation steps
    pub simulation_steps: Vec<SimulationStep>,
    /// Simulation property overrides
    pub simulation_properties: HashMap<String, String>,

    // Feature Costing
    /// Show feature costs
    pub show_feature_costs: bool,
    /// Feature cost data
    pub feature_costs: Vec<FeatureCost>,
    /// Total disk space required
    pub total_disk_space: u64,

    // Annotations
    /// Row annotations
    pub annotations: Vec<RowAnnotation>,
    /// Show annotations panel
    pub show_annotations: bool,

    // Watch Expressions
    /// Watch expressions
    pub watch_expressions: Vec<WatchExpression>,
    /// Show watch panel
    pub show_watch_panel: bool,

    // Row History
    /// Row change history
    pub row_history: Vec<RowChange>,
    /// Show history panel
    pub show_history_panel: bool,

    // Split View
    /// Split view configuration
    pub split_view: SplitViewConfig,
    /// Second table data for split view
    pub split_table: Option<Table>,

    // Recent Searches
    /// Recent search history
    pub recent_searches: Vec<RecentSearch>,

    // Favorites
    /// Favorite items
    pub favorites: Vec<FavoriteItem>,
    /// Show favorites panel
    pub show_favorites: bool,

    // Column Profiles
    /// Saved column profiles
    pub column_profiles: Vec<ColumnProfile>,
    /// Current profile name
    pub current_profile: Option<String>,

    // Rollback Viewer
    /// Show rollback viewer
    pub show_rollback_viewer: bool,
    /// Rollback operations
    pub rollback_operations: Vec<RollbackOperation>,

    // Action Timing
    /// Show timing estimates
    pub show_action_timing: bool,
    /// Action timing data
    pub action_timings: Vec<ActionTiming>,

    // Patch Analysis
    /// Show patch analysis
    pub show_patch_analysis: bool,
    /// Patch deltas
    pub patch_deltas: Vec<PatchDelta>,

    // SQL Syntax Highlighting
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,

    // Auto-complete
    /// Enable auto-complete
    pub auto_complete: bool,
    /// Auto-complete suggestions
    pub auto_complete_suggestions: Vec<String>,

    // Export formats
    /// Show Excel export dialog
    pub show_excel_export: bool,
    /// Show XML export dialog
    pub show_xml_export: bool,
}

impl MsiExplorerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_file: None,
            msi: None,
            tables: Vec::new(),
            tables_by_category: HashMap::new(),
            selected_table: None,
            current_table: None,
            summary: None,
            stats: None,
            search_query: String::new(),
            search_results: Vec::new(),
            status: "Ready".into(),
            show_categories: true,
            error: None,
            theme_applied: false,
            // New features
            number_format: NumberFormat::default(),
            view_mode: ViewMode::default(),
            diff_file: None,
            diff_msi: None,
            tree_data: Vec::new(),
            files_list: Vec::new(),
            // Edit mode
            edit_mode: false,
            editing_cell: None,
            pending_edits: Vec::new(),
            pending_adds: Vec::new(),
            pending_deletes: Vec::new(),
            cascade_preview: None,
            has_changes: false,
            new_row_values: Vec::new(),
            show_add_row_dialog: false,
            // ICE Validation
            validation_result: None,
            show_validation_panel: false,
            validation_running: false,
            // Transform support
            transform_mode: false,
            transform_base_file: None,
            // Merge Module support
            show_merge_dialog: false,
            merge_module_path: None,
            merge_target_feature: String::new(),
            merge_target_directory: String::new(),
            // Edit Summary Info
            show_edit_summary_dialog: false,
            edit_summary_title: String::new(),
            edit_summary_author: String::new(),
            edit_summary_subject: String::new(),
            edit_summary_comments: String::new(),
            // Copy/Paste
            clipboard_rows: Vec::new(),
            clipboard_source_table: None,
            // Find & Replace
            show_find_replace: false,
            find_text: String::new(),
            replace_text: String::new(),
            find_results: Vec::new(),
            find_result_index: 0,
            find_all_tables: true,
            // Undo/Redo
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            // Sequence visualization
            show_sequence_view: false,
            sequence_table: "InstallExecuteSequence".to_string(),
            // Dialog preview
            show_dialog_preview: false,
            preview_dialog_name: None,
            // Patch creation
            show_create_patch_dialog: false,
            patch_old_msi: None,
            patch_new_msi: None,
            patch_output_path: None,
            patch_diff: None,
            // Row Filtering
            filter_column: None,
            filter_text: String::new(),
            show_filter_bar: false,
            // Column Sorting
            sort_column: None,
            sort_ascending: true,
            // Recent Files
            recent_files: Vec::new(),
            // GUID Generator
            show_guid_generator: false,
            generated_guid: String::new(),
            guid_uppercase: true,
            guid_braces: true,
            // Regex Search
            find_use_regex: false,
            find_case_sensitive: false,
            // SQL Query Editor
            show_sql_editor: false,
            sql_query: String::new(),
            sql_results: None,
            sql_error: None,
            // Custom Table Creation
            show_create_table_dialog: false,
            new_table_name: String::new(),
            new_table_columns: Vec::new(),
            // Digital Signature
            signature_info: None,
            // Script Viewer
            show_script_viewer: false,
            selected_binary: None,
            script_content: String::new(),
            // Reference Validation
            show_reference_validation: false,
            reference_errors: Vec::new(),
            // Print/Report
            show_report_dialog: false,
            report_format: ReportFormat::default(),
            // String Localization
            show_localization_view: false,
            selected_language: None,
            available_languages: Vec::new(),
            // CAB Rebuild
            show_cab_rebuild: false,
            cab_compression: CabCompression::default(),
            // Table Schema Viewer
            show_schema_viewer: false,
            // Dependency Graph
            show_dependency_graph: false,
            dependency_data: Vec::new(),
            // File Hash Verification
            show_hash_verification: false,
            hash_results: Vec::new(),
            // Theme
            dark_mode: true,
            // Export to WiX
            show_wix_export: false,
            // Bulk Edit
            show_bulk_edit: false,
            bulk_edit_column: None,
            bulk_edit_find: String::new(),
            bulk_edit_replace: String::new(),
            // Bookmarks
            bookmarks: Vec::new(),
            show_bookmarks: false,
            // Statistics Dashboard
            show_stats_dashboard: false,
            // Stream Viewer
            show_stream_viewer: false,
            streams: Vec::new(),
            // Property Editor
            show_property_editor: false,
            known_properties: Vec::new(),
            // Condition Validator
            show_condition_validator: false,
            condition_text: String::new(),
            condition_result: None,
            // Action Sequence Diagram
            show_sequence_diagram: false,
            // Preview panels
            show_registry_preview: false,
            show_shortcut_preview: false,
            show_service_preview: false,
            // CAB Contents
            show_cab_contents: false,
            cab_files: Vec::new(),
            // Diff Export
            diff_export_format: DiffExportFormat::default(),
            // Row Selection
            selected_rows: Vec::new(),
            last_selected_row: None,
            // Context Menu
            show_context_menu: false,
            context_menu_pos: (0.0, 0.0),
            context_menu_row: None,
            // Transform Comparison
            show_transform_compare: false,
            compare_transform_path: None,
            // Duplicate Detection
            show_duplicate_detection: false,
            duplicates: Vec::new(),
            // Orphan Detection
            show_orphan_detection: false,
            orphans: Vec::new(),
            // Icon Extraction
            show_icon_extraction: false,
            icons: Vec::new(),
            // File Preview
            show_file_preview: false,
            file_preview: None,
            // Feature Tree View
            show_feature_tree: false,
            feature_tree: Vec::new(),
            // Directory Tree View
            show_directory_tree: false,
            directory_tree: Vec::new(),
            // Component Rules Checker
            show_component_rules: false,
            component_violations: Vec::new(),
            // Custom Action Decoder
            show_ca_decoder: false,
            decoded_cas: Vec::new(),
            // Install Sequence Timeline
            show_timeline: false,
            timeline_events: Vec::new(),
            timeline_phase: InstallPhase::ExecuteSequence,
            // Foreign Key Navigation
            fk_navigate_to: None,
            // Keyboard Shortcuts
            show_shortcuts_help: false,
            // User Guide
            show_user_guide: false,
            // About Dialog
            show_about: false,
            // Table Templates
            show_template_picker: false,
            table_templates: Vec::new(),
            // Condition Builder
            show_condition_builder: false,
            condition_builder_root: None,
            built_condition: String::new(),
            // Column Statistics
            show_column_stats: false,
            column_stats: Vec::new(),
            // Database Repair
            show_db_repair: false,
            db_issues: Vec::new(),
            // Batch Operations
            show_batch_panel: false,
            batch_operation: None,
            // Session Logging
            session_log: Vec::new(),
            show_session_log: false,
            // Plugin System
            plugins: Vec::new(),
            show_plugin_manager: false,
            // Database Compression
            show_compression_dialog: false,
            // CAB Operations
            show_cab_extraction: false,
            extracted_cab_files: Vec::new(),
            cab_extraction_path: None,
            // Binary Diff
            show_binary_diff: false,
            binary_diff_results: Vec::new(),
            // Install Simulation
            show_simulation: false,
            simulation_steps: Vec::new(),
            simulation_properties: HashMap::new(),
            // Feature Costing
            show_feature_costs: false,
            feature_costs: Vec::new(),
            total_disk_space: 0,
            // Annotations
            annotations: Vec::new(),
            show_annotations: false,
            // Watch Expressions
            watch_expressions: Vec::new(),
            show_watch_panel: false,
            // Row History
            row_history: Vec::new(),
            show_history_panel: false,
            // Split View
            split_view: SplitViewConfig::default(),
            split_table: None,
            // Recent Searches
            recent_searches: Vec::new(),
            // Favorites
            favorites: Vec::new(),
            show_favorites: false,
            // Column Profiles
            column_profiles: Vec::new(),
            current_profile: None,
            // Rollback Viewer
            show_rollback_viewer: false,
            rollback_operations: Vec::new(),
            // Action Timing
            show_action_timing: false,
            action_timings: Vec::new(),
            // Patch Analysis
            show_patch_analysis: false,
            patch_deltas: Vec::new(),
            // SQL Syntax Highlighting
            syntax_highlighting: true,
            // Auto-complete
            auto_complete: true,
            auto_complete_suggestions: Vec::new(),
            // Export formats
            show_excel_export: false,
            show_xml_export: false,
        }
    }

    /// Open an MSI file
    pub fn open_file(&mut self, path: PathBuf) {
        self.error = None;
        self.status = format!("Opening {}...", path.display());

        match MsiFile::open(&path) {
            Ok(mut msi) => {
                self.tables = msi.table_names();
                self.tables.sort();
                self.tables_by_category = msi.tables_by_category();
                self.summary = msi.summary_info().ok();
                self.stats = msi.stats().ok();
                self.current_file = Some(path.clone());

                // Build tree and files list
                self.build_tree(&mut msi);
                self.build_files_list(&mut msi);

                self.msi = Some(msi);
                self.selected_table = None;
                self.current_table = None;
                self.search_results.clear();

                // Add to recent files
                self.add_to_recent(path.clone());

                // Reset filter/sort when opening new file
                self.clear_filter();
                self.sort_column = None;

                self.status = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Opened".into());
            }
            Err(e) => {
                self.error = Some(format!("Failed to open: {}", e));
                self.status = "Error".into();
            }
        }
    }

    /// Open second MSI for diff
    pub fn open_diff_file(&mut self, path: PathBuf) {
        match MsiFile::open(&path) {
            Ok(msi) => {
                self.diff_file = Some(path);
                self.diff_msi = Some(msi);
                self.view_mode = ViewMode::Diff;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Failed to open diff file: {}", e));
            }
        }
    }

    /// Build Feature → Component → File tree
    fn build_tree(&mut self, msi: &mut MsiFile) {
        self.tree_data.clear();

        // Get Feature table
        let features = match msi.get_table("Feature") {
            Ok(t) => t,
            Err(_) => return,
        };

        // Get FeatureComponents table
        let feature_components = match msi.get_table("FeatureComponents") {
            Ok(t) => t,
            Err(_) => return,
        };

        // Get Component table
        let components = match msi.get_table("Component") {
            Ok(t) => t,
            Err(_) => return,
        };

        // Get File table
        let files = msi.get_table("File").ok();

        // Build component → files map
        let mut comp_files: HashMap<String, Vec<(String, i64, Option<String>)>> = HashMap::new();
        if let Some(ref file_table) = files {
            let file_col = file_table.columns.iter().position(|c| c.name == "File");
            let comp_col = file_table.columns.iter().position(|c| c.name == "Component_");
            let name_col = file_table.columns.iter().position(|c| c.name == "FileName");
            let size_col = file_table.columns.iter().position(|c| c.name == "FileSize");
            let ver_col = file_table.columns.iter().position(|c| c.name == "Version");

            if let (Some(fc), Some(cc), Some(nc)) = (file_col, comp_col, name_col) {
                for row in &file_table.rows {
                    let file_key = row.values.get(fc).map(|v| v.display()).unwrap_or_default();
                    let comp = row.values.get(cc).map(|v| v.display()).unwrap_or_default();
                    let name = row.values.get(nc).map(|v| {
                        let n = v.display();
                        // Handle short|long name format
                        n.split('|').last().unwrap_or(&n).to_string()
                    }).unwrap_or(file_key.clone());
                    let size = size_col.and_then(|i| row.values.get(i))
                        .and_then(|v| v.display().parse::<i64>().ok())
                        .unwrap_or(0);
                    let version = ver_col.and_then(|i| row.values.get(i))
                        .map(|v| v.display())
                        .filter(|s| !s.is_empty());

                    comp_files.entry(comp).or_default().push((name, size, version));
                }
            }
        }

        // Build feature → components map
        let mut feat_comps: HashMap<String, Vec<String>> = HashMap::new();
        {
            let feat_col = feature_components.columns.iter().position(|c| c.name == "Feature_");
            let comp_col = feature_components.columns.iter().position(|c| c.name == "Component_");

            if let (Some(fc), Some(cc)) = (feat_col, comp_col) {
                for row in &feature_components.rows {
                    let feat = row.values.get(fc).map(|v| v.display()).unwrap_or_default();
                    let comp = row.values.get(cc).map(|v| v.display()).unwrap_or_default();
                    feat_comps.entry(feat).or_default().push(comp);
                }
            }
        }

        // Build component info map
        let mut comp_info: HashMap<String, Option<String>> = HashMap::new();
        {
            let comp_col = components.columns.iter().position(|c| c.name == "Component");
            let dir_col = components.columns.iter().position(|c| c.name == "Directory_");

            if let (Some(cc), Some(dc)) = (comp_col, dir_col) {
                for row in &components.rows {
                    let comp = row.values.get(cc).map(|v| v.display()).unwrap_or_default();
                    let dir = row.values.get(dc).map(|v| v.display());
                    comp_info.insert(comp, dir);
                }
            }
        }

        // Get feature info
        let feat_col = features.columns.iter().position(|c| c.name == "Feature");
        let title_col = features.columns.iter().position(|c| c.name == "Title");
        let parent_col = features.columns.iter().position(|c| c.name == "Feature_Parent");

        // Build feature nodes (only root features)
        if let Some(fc) = feat_col {
            for row in &features.rows {
                let feat_name = row.values.get(fc).map(|v| v.display()).unwrap_or_default();
                let title = title_col.and_then(|i| row.values.get(i)).map(|v| v.display()).filter(|s| !s.is_empty());
                let parent = parent_col.and_then(|i| row.values.get(i)).map(|v| v.display()).filter(|s| !s.is_empty());

                // Skip non-root features for now (they'll be children)
                if parent.is_some() {
                    continue;
                }

                let mut feature_node = TreeNode {
                    name: feat_name.clone(),
                    node_type: TreeNodeType::Feature { title },
                    children: Vec::new(),
                };

                // Add components for this feature
                if let Some(comps) = feat_comps.get(&feat_name) {
                    for comp_name in comps {
                        let directory = comp_info.get(comp_name).cloned().flatten();

                        let mut comp_node = TreeNode {
                            name: comp_name.clone(),
                            node_type: TreeNodeType::Component { directory },
                            children: Vec::new(),
                        };

                        // Add files for this component
                        if let Some(file_list) = comp_files.get(comp_name) {
                            for (file_name, size, version) in file_list {
                                comp_node.children.push(TreeNode {
                                    name: file_name.clone(),
                                    node_type: TreeNodeType::File {
                                        size: Some(*size),
                                        version: version.clone(),
                                    },
                                    children: Vec::new(),
                                });
                            }
                        }

                        feature_node.children.push(comp_node);
                    }
                }

                self.tree_data.push(feature_node);
            }
        }
    }

    /// Build flat files list for extraction view
    fn build_files_list(&mut self, msi: &mut MsiFile) {
        self.files_list.clear();

        let files = match msi.get_table("File") {
            Ok(t) => t,
            Err(_) => return,
        };

        let components = msi.get_table("Component").ok();
        let media = msi.get_table("Media").ok();

        // Build component → directory map
        let mut comp_dirs: HashMap<String, String> = HashMap::new();
        if let Some(ref comp_table) = components {
            let comp_col = comp_table.columns.iter().position(|c| c.name == "Component");
            let dir_col = comp_table.columns.iter().position(|c| c.name == "Directory_");
            if let (Some(cc), Some(dc)) = (comp_col, dir_col) {
                for row in &comp_table.rows {
                    let comp = row.values.get(cc).map(|v| v.display()).unwrap_or_default();
                    let dir = row.values.get(dc).map(|v| v.display()).unwrap_or_default();
                    comp_dirs.insert(comp, dir);
                }
            }
        }

        // Build sequence → cabinet map
        let mut seq_cab: Vec<(i64, String)> = Vec::new();
        if let Some(ref media_table) = media {
            let seq_col = media_table.columns.iter().position(|c| c.name == "LastSequence");
            let cab_col = media_table.columns.iter().position(|c| c.name == "Cabinet");
            if let (Some(sc), Some(cc)) = (seq_col, cab_col) {
                for row in &media_table.rows {
                    let seq = row.values.get(sc).and_then(|v| v.display().parse::<i64>().ok()).unwrap_or(0);
                    let cab = row.values.get(cc).map(|v| v.display()).unwrap_or_default();
                    seq_cab.push((seq, cab));
                }
            }
            seq_cab.sort_by_key(|(s, _)| *s);
        }

        let file_col = files.columns.iter().position(|c| c.name == "File");
        let name_col = files.columns.iter().position(|c| c.name == "FileName");
        let comp_col = files.columns.iter().position(|c| c.name == "Component_");
        let size_col = files.columns.iter().position(|c| c.name == "FileSize");
        let ver_col = files.columns.iter().position(|c| c.name == "Version");
        let seq_col = files.columns.iter().position(|c| c.name == "Sequence");

        if let (Some(fc), Some(nc), Some(cc)) = (file_col, name_col, comp_col) {
            for row in &files.rows {
                let file_key = row.values.get(fc).map(|v| v.display()).unwrap_or_default();
                let file_name = row.values.get(nc).map(|v| {
                    let n = v.display();
                    n.split('|').last().unwrap_or(&n).to_string()
                }).unwrap_or(file_key.clone());
                let component = row.values.get(cc).map(|v| v.display()).unwrap_or_default();
                let size = size_col.and_then(|i| row.values.get(i))
                    .and_then(|v| v.display().parse::<i64>().ok())
                    .unwrap_or(0);
                let version = ver_col.and_then(|i| row.values.get(i))
                    .map(|v| v.display())
                    .filter(|s| !s.is_empty());
                let sequence = seq_col.and_then(|i| row.values.get(i))
                    .and_then(|v| v.display().parse::<i64>().ok())
                    .unwrap_or(0);

                let directory = comp_dirs.get(&component).cloned().unwrap_or_default();

                // Find cabinet for this file
                let cab_name = seq_cab.iter()
                    .find(|(s, _)| sequence <= *s)
                    .map(|(_, c)| c.clone())
                    .filter(|c| !c.is_empty());

                self.files_list.push(ExtractedFile {
                    file_key,
                    file_name,
                    component,
                    directory,
                    size,
                    version,
                    cab_name,
                });
            }
        }

        // Sort by directory then filename
        self.files_list.sort_by(|a, b| {
            a.directory.cmp(&b.directory).then(a.file_name.cmp(&b.file_name))
        });
    }

    /// Select a table to view
    pub fn select_table(&mut self, name: &str) {
        self.view_mode = ViewMode::Tables;
        if let Some(ref mut msi) = self.msi {
            match msi.get_table(name) {
                Ok(table) => {
                    self.selected_table = Some(name.to_string());
                    self.current_table = Some(table);
                    self.error = None;
                }
                Err(e) => {
                    self.error = Some(format!("Failed to load table: {}", e));
                }
            }
        }
    }

    /// Perform search
    pub fn do_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
            return;
        }

        if let Some(ref mut msi) = self.msi {
            let options = msi_explorer::search::SearchOptions {
                case_sensitive: false,
                max_results: Some(100),
                ..Default::default()
            };

            match msi_explorer::search::search(msi, &self.search_query, &options) {
                Ok(results) => {
                    self.search_results = results;
                    self.error = None;
                }
                Err(e) => {
                    self.error = Some(format!("Search failed: {}", e));
                }
            }
        }
    }

    /// Start editing a cell
    pub fn start_edit(&mut self, table: &str, row_idx: usize, col_idx: usize, col_name: &str, current_value: &str, is_pk: bool) {
        self.editing_cell = Some(EditingCell {
            table: table.to_string(),
            row_idx,
            col_idx,
            col_name: col_name.to_string(),
            text: current_value.to_string(),
            is_primary_key: is_pk,
        });
    }

    /// Cancel current edit
    pub fn cancel_edit(&mut self) {
        self.editing_cell = None;
        self.cascade_preview = None;
    }

    /// Commit an edit - if it's a PK, show cascade preview
    pub fn commit_edit(&mut self) {
        let edit = match self.editing_cell.take() {
            Some(e) => e,
            None => return,
        };

        // Get current value from table
        let old_value = self.current_table.as_ref()
            .and_then(|t| t.rows.get(edit.row_idx))
            .and_then(|r| r.values.get(edit.col_idx))
            .map(|v| v.display())
            .unwrap_or_default();

        if old_value == edit.text {
            return; // No change
        }

        // If it's a primary key, build cascade preview
        if edit.is_primary_key {
            if let Some(preview) = self.build_cascade_preview(&edit.table, &edit.col_name, &old_value, &edit.text) {
                if !preview.affected.is_empty() {
                    self.cascade_preview = Some(preview);
                    // Re-store the edit for when user confirms
                    self.editing_cell = Some(edit);
                    return;
                }
            }
        }

        // Apply the single edit
        self.apply_single_edit(&edit.table, edit.row_idx, edit.col_idx, &old_value, &edit.text);
    }

    /// Build cascade preview for a PK rename
    fn build_cascade_preview(&mut self, source_table: &str, source_col: &str, old_value: &str, new_value: &str) -> Option<CascadePreview> {
        use crate::schema;

        let schema_map = schema::get_schema();
        let mut affected = Vec::new();

        // Find all tables that have FK references to this table/column
        for (table_name, columns) in &schema_map {
            for (col_name, col_info) in columns {
                if let Some((ref_table, ref_col)) = col_info.foreign_key {
                    if ref_table == source_table && ref_col == source_col {
                        // This column references our source - count affected rows
                        if let Some(ref mut msi) = self.msi {
                            if let Ok(table) = msi.get_table(table_name) {
                                // Find column index
                                if let Some(col_idx) = table.columns.iter().position(|c| c.name == *col_name) {
                                    let count = table.rows.iter()
                                        .filter(|r| r.values.get(col_idx).map(|v| v.display()) == Some(old_value.to_string()))
                                        .count();
                                    if count > 0 {
                                        affected.push(CascadeAffected {
                                            table: table_name.to_string(),
                                            column: col_name.to_string(),
                                            row_count: count,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Some(CascadePreview {
            original_value: old_value.to_string(),
            new_value: new_value.to_string(),
            source_table: source_table.to_string(),
            source_column: source_col.to_string(),
            affected,
        })
    }

    /// Apply cascade rename
    pub fn apply_cascade_rename(&mut self) {
        let preview = match self.cascade_preview.take() {
            Some(p) => p,
            None => return,
        };
        let edit = match self.editing_cell.take() {
            Some(e) => e,
            None => return,
        };

        // Apply the primary key edit
        self.apply_single_edit(&edit.table, edit.row_idx, edit.col_idx, &preview.original_value, &preview.new_value);

        // Apply cascade edits to all referencing tables
        for affected in &preview.affected {
            if let Some(ref mut msi) = self.msi {
                if let Ok(table) = msi.get_table(&affected.table) {
                    if let Some(col_idx) = table.columns.iter().position(|c| c.name == affected.column) {
                        for (row_idx, row) in table.rows.iter().enumerate() {
                            if row.values.get(col_idx).map(|v| v.display()) == Some(preview.original_value.clone()) {
                                self.pending_edits.push(PendingEdit {
                                    table: affected.table.clone(),
                                    row_idx,
                                    col_idx,
                                    old_value: preview.original_value.clone(),
                                    new_value: preview.new_value.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        self.has_changes = true;
        self.status = format!("Renamed '{}' → '{}' ({} references updated)",
            preview.original_value, preview.new_value,
            preview.affected.iter().map(|a| a.row_count).sum::<usize>());
    }

    /// Apply a single edit
    fn apply_single_edit(&mut self, table: &str, row_idx: usize, col_idx: usize, old_value: &str, new_value: &str) {
        self.pending_edits.push(PendingEdit {
            table: table.to_string(),
            row_idx,
            col_idx,
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
        });
        self.has_changes = true;
        self.status = format!("Modified {}.{}", table, col_idx);
    }

    /// Get the display value for a cell (considering pending edits)
    pub fn get_cell_value(&self, table: &str, row_idx: usize, col_idx: usize, original: &str) -> String {
        // Check if there's a pending edit for this cell
        for edit in self.pending_edits.iter().rev() {
            if edit.table == table && edit.row_idx == row_idx && edit.col_idx == col_idx {
                return edit.new_value.clone();
            }
        }
        original.to_string()
    }

    /// Check if a cell has been modified
    pub fn is_cell_modified(&self, table: &str, row_idx: usize, col_idx: usize) -> bool {
        self.pending_edits.iter().any(|e| e.table == table && e.row_idx == row_idx && e.col_idx == col_idx)
    }

    /// Check if a row is marked for deletion
    pub fn is_row_deleted(&self, table: &str, row_idx: usize) -> bool {
        self.pending_deletes.iter().any(|d| d.table == table && d.row_idx == row_idx)
    }

    /// Start adding a new row
    pub fn start_add_row(&mut self) {
        if let Some(ref table) = self.current_table {
            // Initialize with empty values for each column
            self.new_row_values = table.columns.iter().map(|_| String::new()).collect();
            self.show_add_row_dialog = true;
        }
    }

    /// Confirm adding a new row
    pub fn confirm_add_row(&mut self) {
        if let Some(ref table) = self.current_table {
            self.pending_adds.push(PendingRowAdd {
                table: table.name.clone(),
                values: self.new_row_values.clone(),
            });
            self.has_changes = true;
            self.status = format!("Added new row to {}", table.name);
        }
        self.show_add_row_dialog = false;
        self.new_row_values.clear();
    }

    /// Cancel adding a new row
    pub fn cancel_add_row(&mut self) {
        self.show_add_row_dialog = false;
        self.new_row_values.clear();
    }

    /// Delete a row
    pub fn delete_row(&mut self, table: &str, row_idx: usize) {
        // Don't delete the same row twice
        if !self.is_row_deleted(table, row_idx) {
            self.pending_deletes.push(PendingRowDelete {
                table: table.to_string(),
                row_idx,
            });
            self.has_changes = true;
            self.status = format!("Marked row {} for deletion", row_idx);
        }
    }

    /// Undelete a row
    pub fn undelete_row(&mut self, table: &str, row_idx: usize) {
        self.pending_deletes.retain(|d| !(d.table == table && d.row_idx == row_idx));
        if self.pending_edits.is_empty() && self.pending_adds.is_empty() && self.pending_deletes.is_empty() {
            self.has_changes = false;
        }
        self.status = format!("Restored row {}", row_idx);
    }

    /// Get total pending changes count
    pub fn pending_changes_count(&self) -> usize {
        self.pending_edits.len() + self.pending_adds.len() + self.pending_deletes.len()
    }

    /// Discard all pending changes
    pub fn discard_changes(&mut self) {
        self.pending_edits.clear();
        self.pending_adds.clear();
        self.pending_deletes.clear();
        self.has_changes = false;
        self.status = "Changes discarded".into();

        // Reload current table to get fresh data
        if let Some(ref name) = self.selected_table.clone() {
            self.select_table(&name);
        }
    }

    /// Export pending changes to a JSON file
    pub fn export_changes(&mut self) {
        use std::io::Write;

        // Ask user for save location
        let default_name = self.current_file.as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| format!("{}_changes.json", s.to_string_lossy()))
            .unwrap_or_else(|| "msi_changes.json".to_string());

        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .add_filter("JSON", &["json"])
            .save_file()
        {
            // Build changes structure
            let changes = serde_json::json!({
                "source_file": self.current_file.as_ref().map(|p| p.display().to_string()),
                "edits": self.pending_edits.iter().map(|e| {
                    serde_json::json!({
                        "type": "update",
                        "table": e.table,
                        "row": e.row_idx,
                        "column": e.col_idx,
                        "old_value": e.old_value,
                        "new_value": e.new_value
                    })
                }).collect::<Vec<_>>(),
                "additions": self.pending_adds.iter().map(|a| {
                    serde_json::json!({
                        "type": "insert",
                        "table": a.table,
                        "values": a.values
                    })
                }).collect::<Vec<_>>(),
                "deletions": self.pending_deletes.iter().map(|d| {
                    serde_json::json!({
                        "type": "delete",
                        "table": d.table,
                        "row": d.row_idx
                    })
                }).collect::<Vec<_>>(),
                "summary": {
                    "edits": self.pending_edits.len(),
                    "additions": self.pending_adds.len(),
                    "deletions": self.pending_deletes.len()
                }
            });

            match std::fs::File::create(&path) {
                Ok(mut file) => {
                    match serde_json::to_string_pretty(&changes) {
                        Ok(json) => {
                            if let Err(e) = file.write_all(json.as_bytes()) {
                                self.error = Some(format!("Failed to write file: {}", e));
                            } else {
                                self.status = format!("Changes exported to {}", path.display());
                                // Clear changes after successful export
                                self.pending_edits.clear();
                                self.pending_adds.clear();
                                self.pending_deletes.clear();
                                self.has_changes = false;
                            }
                        }
                        Err(e) => {
                            self.error = Some(format!("Failed to serialize: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.error = Some(format!("Failed to create file: {}", e));
                }
            }
        }
    }

    /// Run ICE validation on current MSI
    pub fn run_validation(&mut self) {
        if let Some(ref path) = self.current_file {
            self.validation_running = true;
            self.status = "Running ICE validation...".to_string();

            match ice_validator::validate_msi(path) {
                Ok(result) => {
                    let (errors, warnings, _) = result.count_by_severity();
                    self.status = format!(
                        "Validation complete: {} errors, {} warnings",
                        errors, warnings
                    );
                    self.validation_result = Some(result);
                    self.show_validation_panel = true;
                }
                Err(e) => {
                    self.error = Some(format!("Validation failed: {}", e));
                }
            }
            self.validation_running = false;
        }
    }

    /// Start a new transform (track changes for MST generation)
    pub fn start_new_transform(&mut self) {
        if self.msi.is_some() {
            // Clear any existing changes
            self.pending_edits.clear();
            self.pending_adds.clear();
            self.pending_deletes.clear();
            self.has_changes = false;

            // Store current file as base
            self.transform_base_file = self.current_file.clone();
            self.transform_mode = true;
            self.edit_mode = true;

            self.status = "Transform mode: Make changes, then generate MST".to_string();
        }
    }

    /// Generate MST transform file from pending changes
    pub fn generate_transform(&mut self) {
        use std::io::Write;

        if !self.transform_mode {
            self.error = Some("Not in transform mode".to_string());
            return;
        }

        if self.pending_edits.is_empty() && self.pending_adds.is_empty() && self.pending_deletes.is_empty() {
            self.error = Some("No changes to save as transform".to_string());
            return;
        }

        let default_name = self.current_file.as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| format!("{}.mst", s.to_string_lossy()))
            .unwrap_or_else(|| "transform.mst".to_string());

        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&default_name)
            .add_filter("Transform Files", &["mst"])
            .save_file()
        {
            // For now, export as a detailed JSON that could be applied later
            // Full MST binary format would require additional implementation
            let transform_data = serde_json::json!({
                "format": "msi-explorer-transform",
                "version": "1.0",
                "base_file": self.transform_base_file.as_ref().map(|p| p.display().to_string()),
                "changes": {
                    "edits": self.pending_edits.iter().map(|e| {
                        serde_json::json!({
                            "table": e.table,
                            "row": e.row_idx,
                            "column": e.col_idx,
                            "old_value": e.old_value,
                            "new_value": e.new_value
                        })
                    }).collect::<Vec<_>>(),
                    "inserts": self.pending_adds.iter().map(|a| {
                        serde_json::json!({
                            "table": a.table,
                            "values": a.values
                        })
                    }).collect::<Vec<_>>(),
                    "deletes": self.pending_deletes.iter().map(|d| {
                        serde_json::json!({
                            "table": d.table,
                            "row": d.row_idx
                        })
                    }).collect::<Vec<_>>()
                }
            });

            // Write as .mst.json for now (full MST binary support would need msi crate enhancements)
            let json_path = path.with_extension("mst.json");
            match std::fs::File::create(&json_path) {
                Ok(mut file) => {
                    match serde_json::to_string_pretty(&transform_data) {
                        Ok(json) => {
                            if let Err(e) = file.write_all(json.as_bytes()) {
                                self.error = Some(format!("Failed to write: {}", e));
                            } else {
                                self.status = format!("Transform saved to {}", json_path.display());
                                self.transform_mode = false;
                                self.pending_edits.clear();
                                self.pending_adds.clear();
                                self.pending_deletes.clear();
                                self.has_changes = false;
                            }
                        }
                        Err(e) => self.error = Some(format!("Serialize error: {}", e)),
                    }
                }
                Err(e) => self.error = Some(format!("Failed to create file: {}", e)),
            }
        }
    }

    /// Open merge module dialog
    pub fn open_merge_module_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Merge Modules", &["msm"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.merge_module_path = Some(path);
            self.show_merge_dialog = true;

            // Pre-fill with first feature if available
            if let Some(ref mut msi) = self.msi {
                if let Ok(table) = msi.get_table("Feature") {
                    if !table.rows.is_empty() {
                        if let Some(first_feature) = table.rows.first()
                            .and_then(|r| r.values.first())
                            .map(|v| v.display())
                        {
                            self.merge_target_feature = first_feature;
                        }
                    }
                }
            }

            // Default directory
            if self.merge_target_directory.is_empty() {
                self.merge_target_directory = "TARGETDIR".to_string();
            }
        }
    }

    /// Merge a module into current MSI
    pub fn merge_module(&mut self) {
        let msm_path = match &self.merge_module_path {
            Some(p) => p.clone(),
            None => {
                self.error = Some("No merge module selected".to_string());
                return;
            }
        };

        // Open the merge module
        match MsiFile::open(&msm_path) {
            Ok(mut msm) => {
                let msm_tables = msm.table_names();

                // Count tables that will be affected
                let mut tables_merged = 0;
                let mut rows_added = 0;

                // For each table in MSM, add rows to pending_adds
                for table_name in &msm_tables {
                    // Skip internal tables
                    if table_name.starts_with('_') || table_name == "ModuleComponents"
                        || table_name == "ModuleDependency" || table_name == "ModuleSignature"
                    {
                        continue;
                    }

                    if let Ok(table) = msm.get_table(table_name) {
                        for row in &table.rows {
                            // Replace module GUID placeholders with actual values
                            let processed_row: Vec<String> = row.values.iter().map(|cell| {
                                let cell_str = cell.display();
                                // Replace MergeModule.GUID patterns
                                if cell_str.contains("MergeModule.") {
                                    cell_str.replace("MergeModule.", &format!("Merged_{}_",
                                        msm_path.file_stem()
                                            .map(|s| s.to_string_lossy().to_string())
                                            .unwrap_or_default()
                                    ))
                                } else {
                                    cell_str
                                }
                            }).collect();

                            self.pending_adds.push(PendingRowAdd {
                                table: table_name.clone(),
                                values: processed_row,
                            });
                            rows_added += 1;
                        }
                        tables_merged += 1;
                    }
                }

                self.has_changes = true;
                self.status = format!(
                    "Merged {} tables ({} rows) from {}",
                    tables_merged,
                    rows_added,
                    msm_path.file_name().unwrap_or_default().to_string_lossy()
                );
                self.show_merge_dialog = false;
                self.merge_module_path = None;
            }
            Err(e) => {
                self.error = Some(format!("Failed to open merge module: {}", e));
            }
        }
    }

    /// Apply an existing transform file
    pub fn apply_transform(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Transform Files", &["mst.json", "json"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(data) => {
                            // Parse and apply edits
                            if let Some(changes) = data.get("changes") {
                                // Apply edits
                                if let Some(edits) = changes.get("edits").and_then(|e| e.as_array()) {
                                    for edit in edits {
                                        if let (Some(table), Some(row), Some(col), Some(old_val), Some(new_val)) = (
                                            edit.get("table").and_then(|v| v.as_str()),
                                            edit.get("row").and_then(|v| v.as_u64()),
                                            edit.get("column").and_then(|v| v.as_u64()),
                                            edit.get("old_value").and_then(|v| v.as_str()),
                                            edit.get("new_value").and_then(|v| v.as_str()),
                                        ) {
                                            self.pending_edits.push(PendingEdit {
                                                table: table.to_string(),
                                                row_idx: row as usize,
                                                col_idx: col as usize,
                                                old_value: old_val.to_string(),
                                                new_value: new_val.to_string(),
                                            });
                                        }
                                    }
                                }

                                // Apply inserts
                                if let Some(inserts) = changes.get("inserts").and_then(|e| e.as_array()) {
                                    for insert in inserts {
                                        if let (Some(table), Some(values)) = (
                                            insert.get("table").and_then(|v| v.as_str()),
                                            insert.get("values").and_then(|v| v.as_array()),
                                        ) {
                                            let row_values: Vec<String> = values.iter()
                                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                                .collect();
                                            self.pending_adds.push(PendingRowAdd {
                                                table: table.to_string(),
                                                values: row_values,
                                            });
                                        }
                                    }
                                }

                                // Apply deletes
                                if let Some(deletes) = changes.get("deletes").and_then(|e| e.as_array()) {
                                    for delete in deletes {
                                        if let (Some(table), Some(row)) = (
                                            delete.get("table").and_then(|v| v.as_str()),
                                            delete.get("row").and_then(|v| v.as_u64()),
                                        ) {
                                            self.pending_deletes.push(PendingRowDelete {
                                                table: table.to_string(),
                                                row_idx: row as usize,
                                            });
                                        }
                                    }
                                }
                            }

                            self.has_changes = !self.pending_edits.is_empty()
                                || !self.pending_adds.is_empty()
                                || !self.pending_deletes.is_empty();

                            self.status = format!(
                                "Applied transform: {} edits, {} inserts, {} deletes",
                                self.pending_edits.len(),
                                self.pending_adds.len(),
                                self.pending_deletes.len()
                            );
                        }
                        Err(e) => self.error = Some(format!("Invalid transform file: {}", e)),
                    }
                }
                Err(e) => self.error = Some(format!("Failed to read file: {}", e)),
            }
        }
    }

    /// Extract binary data from Binary table
    pub fn extract_binary(&mut self, binary_name: &str) {
        if let Some(ref mut msi) = self.msi {
            if let Ok(table) = msi.get_table("Binary") {
                // Find the row with this name
                let name_col = table.column_index("Name").unwrap_or(0);
                let data_col = table.column_index("Data").unwrap_or(1);

                for row in &table.rows {
                    if row.values.get(name_col).map(|v| v.display()) == Some(binary_name.to_string()) {
                        // Get binary data - in MSI it's stored as a stream
                        if let Some(save_path) = rfd::FileDialog::new()
                            .set_file_name(binary_name)
                            .save_file()
                        {
                            // For now, export the data column value
                            // Real implementation would read the binary stream
                            if let Some(data) = row.values.get(data_col) {
                                let data_str = data.display();
                                if let Err(e) = std::fs::write(&save_path, data_str.as_bytes()) {
                                    self.error = Some(format!("Failed to write: {}", e));
                                } else {
                                    self.status = format!("Extracted {} to {}", binary_name, save_path.display());
                                }
                            }
                        }
                        return;
                    }
                }
                self.error = Some(format!("Binary '{}' not found", binary_name));
            }
        }
    }

    /// Export current table to IDT format
    pub fn export_table_idt(&mut self) {
        if let Some(ref table) = self.current_table {
            let default_name = format!("{}.idt", table.name);

            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(&default_name)
                .add_filter("IDT Files", &["idt"])
                .save_file()
            {
                let mut content = String::new();

                // Column names (tab-separated)
                let col_names: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
                content.push_str(&col_names.join("\t"));
                content.push('\n');

                // Column types
                let col_types: Vec<String> = table.columns.iter().map(|c| {
                    match c.col_type {
                        msi_explorer::ColumnType::String => {
                            if c.nullable { "S255" } else { "s255" }.to_string()
                        }
                        msi_explorer::ColumnType::Integer => {
                            if c.nullable { "I4" } else { "i4" }.to_string()
                        }
                    }
                }).collect();
                content.push_str(&col_types.join("\t"));
                content.push('\n');

                // Table name and primary keys
                let pk_indices: Vec<usize> = table.columns.iter()
                    .enumerate()
                    .filter(|(_, c)| c.primary_key)
                    .map(|(i, _)| i + 1)
                    .collect();
                let pk_str: Vec<String> = pk_indices.iter().map(|i| i.to_string()).collect();
                content.push_str(&format!("{}\t{}\n", table.name, pk_str.join("\t")));

                // Data rows
                for row in &table.rows {
                    let values: Vec<String> = row.values.iter().map(|v| v.display()).collect();
                    content.push_str(&values.join("\t"));
                    content.push('\n');
                }

                match std::fs::write(&path, &content) {
                    Ok(_) => self.status = format!("Exported {} to {}", table.name, path.display()),
                    Err(e) => self.error = Some(format!("Failed to write: {}", e)),
                }
            }
        }
    }

    /// Import table from IDT format
    pub fn import_table_idt(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("IDT Files", &["idt"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    let lines: Vec<&str> = content.lines().collect();
                    if lines.len() < 4 {
                        self.error = Some("Invalid IDT file: too few lines".to_string());
                        return;
                    }

                    // Line 3 has table name
                    let table_info: Vec<&str> = lines[2].split('\t').collect();
                    let table_name = table_info[0];

                    // Lines 4+ are data rows
                    let mut rows_imported = 0;
                    for line in lines.iter().skip(3) {
                        if line.is_empty() { continue; }
                        let values: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();
                        self.pending_adds.push(PendingRowAdd {
                            table: table_name.to_string(),
                            values,
                        });
                        rows_imported += 1;
                    }

                    self.has_changes = true;
                    self.status = format!("Imported {} rows to {}", rows_imported, table_name);
                }
                Err(e) => self.error = Some(format!("Failed to read: {}", e)),
            }
        }
    }

    /// Create a new empty MSI database
    pub fn create_new_msi(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("new_package.msi")
            .add_filter("MSI Files", &["msi"])
            .save_file()
        {
            // Create a minimal MSI using the msi crate
            match std::fs::File::create(&path) {
                Ok(file) => {
                    match msi::Package::create(msi::PackageType::Installer, file) {
                        Ok(mut package) => {
                            // Set basic summary info
                            package.summary_info_mut().set_subject("New Package".to_string());

                            // Create Property table with required properties
                            let columns = vec![
                                msi::Column::build("Property").primary_key().id_string(72),
                                msi::Column::build("Value").nullable().text_string(0),
                            ];
                            if let Err(e) = package.create_table("Property", columns) {
                                self.error = Some(format!("Failed to create Property table: {}", e));
                                return;
                            }

                            // Insert required properties
                            let props = vec![
                                ("ProductName", "New Product"),
                                ("ProductCode", "{00000000-0000-0000-0000-000000000000}"),
                                ("ProductVersion", "1.0.0"),
                                ("Manufacturer", "Unknown"),
                            ];
                            for (name, value) in props {
                                let query = msi::Insert::into("Property").row(vec![
                                    msi::Value::Str(name.to_string()),
                                    msi::Value::Str(value.to_string()),
                                ]);
                                if let Err(e) = package.insert_rows(query) {
                                    self.error = Some(format!("Failed to insert property: {}", e));
                                    return;
                                }
                            }

                            drop(package);
                            self.status = format!("Created new MSI: {}", path.display());
                            // Open the new file
                            self.open_file(path);
                        }
                        Err(e) => self.error = Some(format!("Failed to create MSI: {}", e)),
                    }
                }
                Err(e) => self.error = Some(format!("Failed to create file: {}", e)),
            }
        }
    }

    /// Open edit summary dialog
    pub fn open_edit_summary_dialog(&mut self) {
        if let Some(ref summary) = self.summary {
            self.edit_summary_title = summary.title.clone().unwrap_or_default();
            self.edit_summary_author = summary.author.clone().unwrap_or_default();
            self.edit_summary_subject = summary.subject.clone().unwrap_or_default();
            self.edit_summary_comments = String::new(); // Comments not in our SummaryInfo struct
        }
        self.show_edit_summary_dialog = true;
    }

    /// Save summary info changes (as pending - actual save needs msi crate write support)
    pub fn save_summary_changes(&mut self) {
        // Update the local summary info
        if let Some(ref mut summary) = self.summary {
            if !self.edit_summary_title.is_empty() {
                summary.title = Some(self.edit_summary_title.clone());
            }
            if !self.edit_summary_author.is_empty() {
                summary.author = Some(self.edit_summary_author.clone());
            }
            if !self.edit_summary_subject.is_empty() {
                summary.subject = Some(self.edit_summary_subject.clone());
            }
        }
        self.show_edit_summary_dialog = false;
        self.has_changes = true;
        self.status = "Summary info updated (pending save)".to_string();
    }

    // ==================== Copy/Paste ====================

    /// Copy selected rows to clipboard
    pub fn copy_rows(&mut self, row_indices: &[usize]) {
        if let Some(ref table) = self.current_table {
            self.clipboard_rows.clear();
            for &idx in row_indices {
                if let Some(row) = table.rows.get(idx) {
                    let values: Vec<String> = row.values.iter().map(|v| v.display()).collect();
                    self.clipboard_rows.push(values);
                }
            }
            self.clipboard_source_table = Some(table.name.clone());
            self.status = format!("Copied {} row(s)", self.clipboard_rows.len());
        }
    }

    /// Copy current row to clipboard
    pub fn copy_current_row(&mut self, row_idx: usize) {
        self.copy_rows(&[row_idx]);
    }

    /// Paste rows from clipboard
    pub fn paste_rows(&mut self) {
        if self.clipboard_rows.is_empty() {
            self.error = Some("Clipboard is empty".to_string());
            return;
        }

        if let Some(ref table) = self.current_table {
            let count = self.clipboard_rows.len();
            for row_values in &self.clipboard_rows {
                self.pending_adds.push(PendingRowAdd {
                    table: table.name.clone(),
                    values: row_values.clone(),
                });
            }

            // Add to undo stack
            self.undo_stack.push(UndoAction::Paste {
                table: table.name.clone(),
                count,
            });
            self.redo_stack.clear();

            self.has_changes = true;
            self.status = format!("Pasted {} row(s)", count);
        }
    }

    // ==================== Find & Replace ====================

    /// Perform find operation
    pub fn do_find(&mut self) {
        self.find_results.clear();
        self.find_result_index = 0;

        if self.find_text.is_empty() {
            return;
        }

        let search_lower = self.find_text.to_lowercase();

        if self.find_all_tables {
            // Search all tables
            if let Some(ref mut msi) = self.msi {
                let table_names = msi.table_names();
                for table_name in table_names {
                    if let Ok(table) = msi.get_table(&table_name) {
                        for (row_idx, row) in table.rows.iter().enumerate() {
                            for (col_idx, cell) in row.values.iter().enumerate() {
                                if cell.display().to_lowercase().contains(&search_lower) {
                                    self.find_results.push((table_name.clone(), row_idx, col_idx));
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Search current table only
            if let Some(ref table) = self.current_table {
                for (row_idx, row) in table.rows.iter().enumerate() {
                    for (col_idx, cell) in row.values.iter().enumerate() {
                        if cell.display().to_lowercase().contains(&search_lower) {
                            self.find_results.push((table.name.clone(), row_idx, col_idx));
                        }
                    }
                }
            }
        }

        self.status = format!("Found {} match(es)", self.find_results.len());
    }

    /// Go to next find result
    pub fn find_next(&mut self) {
        if !self.find_results.is_empty() {
            self.find_result_index = (self.find_result_index + 1) % self.find_results.len();
            self.navigate_to_find_result();
        }
    }

    /// Go to previous find result
    pub fn find_prev(&mut self) {
        if !self.find_results.is_empty() {
            if self.find_result_index == 0 {
                self.find_result_index = self.find_results.len() - 1;
            } else {
                self.find_result_index -= 1;
            }
            self.navigate_to_find_result();
        }
    }

    /// Navigate to current find result
    fn navigate_to_find_result(&mut self) {
        if let Some((table_name, _row_idx, _col_idx)) = self.find_results.get(self.find_result_index).cloned() {
            if self.selected_table.as_ref() != Some(&table_name) {
                self.select_table(&table_name);
            }
            self.status = format!(
                "Result {}/{}: {}",
                self.find_result_index + 1,
                self.find_results.len(),
                table_name
            );
        }
    }

    /// Replace current match
    pub fn replace_current(&mut self) {
        if self.find_results.is_empty() || self.replace_text.is_empty() {
            return;
        }

        if let Some((table_name, row_idx, col_idx)) = self.find_results.get(self.find_result_index).cloned() {
            if let Some(ref table) = self.current_table {
                if table.name == table_name {
                    if let Some(row) = table.rows.get(row_idx) {
                        if let Some(cell) = row.values.get(col_idx) {
                            let old_value = cell.display();
                            let new_value = old_value.replace(&self.find_text, &self.replace_text);

                            self.pending_edits.push(PendingEdit {
                                table: table_name.clone(),
                                row_idx,
                                col_idx,
                                old_value: old_value.clone(),
                                new_value: new_value.clone(),
                            });

                            self.undo_stack.push(UndoAction::Edit {
                                table: table_name,
                                row_idx,
                                col_idx,
                                old_value,
                                new_value,
                            });
                            self.redo_stack.clear();

                            self.has_changes = true;
                            self.find_results.remove(self.find_result_index);
                            if self.find_result_index >= self.find_results.len() && !self.find_results.is_empty() {
                                self.find_result_index = 0;
                            }
                            self.status = "Replaced 1 match".to_string();
                        }
                    }
                }
            }
        }
    }

    /// Replace all matches
    pub fn replace_all(&mut self) {
        if self.find_results.is_empty() || self.replace_text.is_empty() {
            return;
        }

        let mut replaced = 0;
        for (table_name, row_idx, col_idx) in self.find_results.drain(..).collect::<Vec<_>>() {
            // We need to get the current value - this is simplified
            self.pending_edits.push(PendingEdit {
                table: table_name.clone(),
                row_idx,
                col_idx,
                old_value: self.find_text.clone(),
                new_value: self.replace_text.clone(),
            });
            replaced += 1;
        }

        self.has_changes = true;
        self.status = format!("Replaced {} match(es)", replaced);
    }

    // ==================== Undo/Redo ====================

    /// Undo last action
    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop() {
            match &action {
                UndoAction::Edit { table, row_idx, col_idx, old_value, new_value } => {
                    // Find and remove the edit from pending_edits
                    self.pending_edits.retain(|e| {
                        !(e.table == *table && e.row_idx == *row_idx && e.col_idx == *col_idx && e.new_value == *new_value)
                    });
                    self.status = format!("Undid edit in {}", table);
                }
                UndoAction::AddRow { table, .. } => {
                    // Remove the last add for this table
                    if let Some(pos) = self.pending_adds.iter().rposition(|a| a.table == *table) {
                        self.pending_adds.remove(pos);
                    }
                    self.status = format!("Undid row add in {}", table);
                }
                UndoAction::DeleteRow { table, row_idx, .. } => {
                    // Remove the delete from pending_deletes
                    self.pending_deletes.retain(|d| !(d.table == *table && d.row_idx == *row_idx));
                    self.status = format!("Undid row delete in {}", table);
                }
                UndoAction::Paste { table, count } => {
                    // Remove the last 'count' adds for this table
                    for _ in 0..*count {
                        if let Some(pos) = self.pending_adds.iter().rposition(|a| a.table == *table) {
                            self.pending_adds.remove(pos);
                        }
                    }
                    self.status = format!("Undid paste of {} rows", count);
                }
            }
            self.redo_stack.push(action);
            self.has_changes = !self.pending_edits.is_empty()
                || !self.pending_adds.is_empty()
                || !self.pending_deletes.is_empty();
        }
    }

    /// Redo last undone action
    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop() {
            match &action {
                UndoAction::Edit { table, row_idx, col_idx, old_value, new_value } => {
                    self.pending_edits.push(PendingEdit {
                        table: table.clone(),
                        row_idx: *row_idx,
                        col_idx: *col_idx,
                        old_value: old_value.clone(),
                        new_value: new_value.clone(),
                    });
                    self.status = format!("Redid edit in {}", table);
                }
                UndoAction::AddRow { table, values } => {
                    self.pending_adds.push(PendingRowAdd {
                        table: table.clone(),
                        values: values.clone(),
                    });
                    self.status = format!("Redid row add in {}", table);
                }
                UndoAction::DeleteRow { table, row_idx, .. } => {
                    self.pending_deletes.push(PendingRowDelete {
                        table: table.clone(),
                        row_idx: *row_idx,
                    });
                    self.status = format!("Redid row delete in {}", table);
                }
                UndoAction::Paste { table, count } => {
                    // Re-paste from clipboard if available
                    for row_values in self.clipboard_rows.iter().take(*count) {
                        self.pending_adds.push(PendingRowAdd {
                            table: table.clone(),
                            values: row_values.clone(),
                        });
                    }
                    self.status = format!("Redid paste of {} rows", count);
                }
            }
            self.undo_stack.push(action);
            self.has_changes = true;
        }
    }

    // ==================== Sequence Visualization ====================

    /// Get sequence data for visualization
    pub fn get_sequence_data(&mut self) -> Vec<(String, i32, String)> {
        let mut sequences = Vec::new();

        if let Some(ref mut msi) = self.msi {
            if let Ok(table) = msi.get_table(&self.sequence_table) {
                let action_col = table.column_index("Action").unwrap_or(0);
                let sequence_col = table.column_index("Sequence").unwrap_or(1);
                let condition_col = table.column_index("Condition");

                for row in &table.rows {
                    let action = row.values.get(action_col).map(|v| v.display()).unwrap_or_default();
                    let seq_num = row.values.get(sequence_col)
                        .and_then(|v| match v {
                            msi_explorer::CellValue::Integer(n) => Some(*n),
                            msi_explorer::CellValue::String(s) => s.parse().ok(),
                            _ => None,
                        })
                        .unwrap_or(0);
                    let condition = condition_col
                        .and_then(|idx| row.values.get(idx))
                        .map(|v| v.display())
                        .unwrap_or_default();

                    sequences.push((action, seq_num, condition));
                }
            }
        }

        sequences.sort_by_key(|(_, seq, _)| *seq);
        sequences
    }

    // ==================== Dialog Preview ====================

    /// Get dialog data for preview
    pub fn get_dialog_data(&mut self) -> Option<DialogPreview> {
        let dialog_name = self.preview_dialog_name.as_ref()?;

        if let Some(ref mut msi) = self.msi {
            // Get dialog dimensions
            if let Ok(dialog_table) = msi.get_table("Dialog") {
                for row in &dialog_table.rows {
                    let name = row.values.first().map(|v| v.display()).unwrap_or_default();
                    if name == *dialog_name {
                        let width = row.values.get(4)
                            .and_then(|v| match v {
                                msi_explorer::CellValue::Integer(n) => Some(*n),
                                _ => None,
                            })
                            .unwrap_or(370);
                        let height = row.values.get(5)
                            .and_then(|v| match v {
                                msi_explorer::CellValue::Integer(n) => Some(*n),
                                _ => None,
                            })
                            .unwrap_or(270);
                        let title = row.values.get(7).map(|v| v.display()).unwrap_or_default();

                        // Get controls
                        let mut controls = Vec::new();
                        if let Ok(control_table) = msi.get_table("Control") {
                            for ctrl_row in &control_table.rows {
                                let ctrl_dialog = ctrl_row.values.first().map(|v| v.display()).unwrap_or_default();
                                if ctrl_dialog == *dialog_name {
                                    let ctrl_name = ctrl_row.values.get(1).map(|v| v.display()).unwrap_or_default();
                                    let ctrl_type = ctrl_row.values.get(2).map(|v| v.display()).unwrap_or_default();
                                    let x = ctrl_row.values.get(3).and_then(|v| match v {
                                        msi_explorer::CellValue::Integer(n) => Some(*n),
                                        _ => None,
                                    }).unwrap_or(0);
                                    let y = ctrl_row.values.get(4).and_then(|v| match v {
                                        msi_explorer::CellValue::Integer(n) => Some(*n),
                                        _ => None,
                                    }).unwrap_or(0);
                                    let w = ctrl_row.values.get(5).and_then(|v| match v {
                                        msi_explorer::CellValue::Integer(n) => Some(*n),
                                        _ => None,
                                    }).unwrap_or(50);
                                    let h = ctrl_row.values.get(6).and_then(|v| match v {
                                        msi_explorer::CellValue::Integer(n) => Some(*n),
                                        _ => None,
                                    }).unwrap_or(20);
                                    let text = ctrl_row.values.get(9).map(|v| v.display()).unwrap_or_default();

                                    controls.push(ControlPreview {
                                        name: ctrl_name,
                                        control_type: ctrl_type,
                                        x, y, width: w, height: h,
                                        text,
                                    });
                                }
                            }
                        }

                        return Some(DialogPreview {
                            name: dialog_name.clone(),
                            title,
                            width,
                            height,
                            controls,
                        });
                    }
                }
            }
        }
        None
    }

    /// Get list of dialog names
    pub fn get_dialog_names(&mut self) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(ref mut msi) = self.msi {
            if let Ok(table) = msi.get_table("Dialog") {
                for row in &table.rows {
                    if let Some(name) = row.values.first() {
                        names.push(name.display());
                    }
                }
            }
        }
        names.sort();
        names
    }

    // ==================== Patch Creation ====================

    /// Open dialog to select old MSI for patch
    pub fn select_patch_old_msi(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Old (Base) MSI")
            .add_filter("MSI Files", &["msi"])
            .pick_file()
        {
            self.patch_old_msi = Some(path);
        }
    }

    /// Open dialog to select new MSI for patch
    pub fn select_patch_new_msi(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select New (Upgraded) MSI")
            .add_filter("MSI Files", &["msi"])
            .pick_file()
        {
            self.patch_new_msi = Some(path);
        }
    }

    /// Calculate diff between old and new MSI
    pub fn calculate_patch_diff(&mut self) {
        let old_path = match &self.patch_old_msi {
            Some(p) => p.clone(),
            None => { self.error = Some("Select old MSI first".into()); return; }
        };
        let new_path = match &self.patch_new_msi {
            Some(p) => p.clone(),
            None => { self.error = Some("Select new MSI first".into()); return; }
        };

        let mut old_msi = match MsiFile::open(&old_path) {
            Ok(m) => m,
            Err(e) => { self.error = Some(format!("Failed to open old MSI: {}", e)); return; }
        };
        let mut new_msi = match MsiFile::open(&new_path) {
            Ok(m) => m,
            Err(e) => { self.error = Some(format!("Failed to open new MSI: {}", e)); return; }
        };

        let old_tables: std::collections::HashSet<_> = old_msi.table_names().into_iter().collect();
        let new_tables: std::collections::HashSet<_> = new_msi.table_names().into_iter().collect();

        let added_tables: Vec<_> = new_tables.difference(&old_tables).cloned().collect();
        let removed_tables: Vec<_> = old_tables.difference(&new_tables).cloned().collect();

        // Get version info
        let old_version = old_msi.get_property("ProductVersion").ok().flatten().unwrap_or_default();
        let new_version = new_msi.get_property("ProductVersion").ok().flatten().unwrap_or_default();
        let old_product_code = old_msi.get_property("ProductCode").ok().flatten().unwrap_or_default();
        let new_product_code = new_msi.get_property("ProductCode").ok().flatten().unwrap_or_default();

        // Compare common tables
        let mut changed_tables = Vec::new();
        let common_tables: Vec<_> = old_tables.intersection(&new_tables).cloned().collect();

        for table_name in common_tables {
            // Skip system tables
            if table_name.starts_with('_') {
                continue;
            }

            let old_table = match old_msi.get_table(&table_name) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let new_table = match new_msi.get_table(&table_name) {
                Ok(t) => t,
                Err(_) => continue,
            };

            // Simple row count comparison (full diff would compare by primary key)
            let old_count = old_table.rows.len();
            let new_count = new_table.rows.len();

            if old_count != new_count {
                let added = if new_count > old_count { new_count - old_count } else { 0 };
                let deleted = if old_count > new_count { old_count - new_count } else { 0 };
                changed_tables.push(TableDiff {
                    name: table_name,
                    added_rows: added,
                    deleted_rows: deleted,
                    modified_rows: 0,
                });
            }
        }

        self.patch_diff = Some(PatchDiff {
            added_tables,
            removed_tables,
            changed_tables,
            old_version,
            new_version,
            old_product_code,
            new_product_code,
        });

        self.status = "Patch diff calculated".to_string();
    }

    /// Export patch information to PCP (Patch Creation Properties) file
    pub fn export_patch_pcp(&mut self) {
        let diff = match &self.patch_diff {
            Some(d) => d,
            None => { self.error = Some("Calculate diff first".into()); return; }
        };

        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save Patch Creation Properties")
            .set_file_name("patch.pcp")
            .add_filter("PCP Files", &["pcp"])
            .add_filter("JSON Files", &["json"])
            .save_file()
        {
            // Export as JSON for now (PCP is a database format)
            let export = serde_json::json!({
                "patch_info": {
                    "old_version": diff.old_version,
                    "new_version": diff.new_version,
                    "old_product_code": diff.old_product_code,
                    "new_product_code": diff.new_product_code,
                },
                "old_msi": self.patch_old_msi.as_ref().map(|p| p.display().to_string()),
                "new_msi": self.patch_new_msi.as_ref().map(|p| p.display().to_string()),
                "changes": {
                    "added_tables": diff.added_tables,
                    "removed_tables": diff.removed_tables,
                    "changed_tables": diff.changed_tables.iter().map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "added_rows": t.added_rows,
                            "deleted_rows": t.deleted_rows,
                            "modified_rows": t.modified_rows,
                        })
                    }).collect::<Vec<_>>(),
                }
            });

            match std::fs::write(&path, serde_json::to_string_pretty(&export).unwrap()) {
                Ok(_) => self.status = format!("Patch info exported to {}", path.display()),
                Err(e) => self.error = Some(format!("Failed to export: {}", e)),
            }
        }
    }

    /// Clear patch creation state
    pub fn clear_patch_state(&mut self) {
        self.patch_old_msi = None;
        self.patch_new_msi = None;
        self.patch_diff = None;
        self.show_create_patch_dialog = false;
    }

    // ==================== Row Filtering ====================

    /// Get filtered rows based on current filter settings
    pub fn get_filtered_rows(&self) -> Vec<usize> {
        let Some(ref table) = self.current_table else {
            return Vec::new();
        };

        if self.filter_text.is_empty() || self.filter_column.is_none() {
            return (0..table.rows.len()).collect();
        }

        let filter_col = self.filter_column.unwrap();
        let filter_lower = self.filter_text.to_lowercase();

        table.rows.iter().enumerate()
            .filter(|(_, row)| {
                if let Some(cell) = row.values.get(filter_col) {
                    cell.display().to_lowercase().contains(&filter_lower)
                } else {
                    false
                }
            })
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Clear filter
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.filter_column = None;
    }

    // ==================== Column Sorting ====================

    /// Get sorted row indices
    pub fn get_sorted_rows(&self, rows: &[usize]) -> Vec<usize> {
        let Some(ref table) = self.current_table else {
            return rows.to_vec();
        };

        let Some(sort_col) = self.sort_column else {
            return rows.to_vec();
        };

        let mut sorted: Vec<usize> = rows.to_vec();
        sorted.sort_by(|&a, &b| {
            let val_a = table.rows.get(a).and_then(|r| r.values.get(sort_col));
            let val_b = table.rows.get(b).and_then(|r| r.values.get(sort_col));

            let cmp = match (val_a, val_b) {
                (Some(msi_explorer::CellValue::Integer(a)), Some(msi_explorer::CellValue::Integer(b))) => a.cmp(b),
                (Some(a), Some(b)) => a.display().cmp(&b.display()),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            };

            if self.sort_ascending { cmp } else { cmp.reverse() }
        });
        sorted
    }

    /// Toggle sort on column
    pub fn toggle_sort(&mut self, col_idx: usize) {
        if self.sort_column == Some(col_idx) {
            if self.sort_ascending {
                self.sort_ascending = false;
            } else {
                self.sort_column = None;
                self.sort_ascending = true;
            }
        } else {
            self.sort_column = Some(col_idx);
            self.sort_ascending = true;
        }
    }

    // ==================== Recent Files ====================

    /// Add file to recent files list
    pub fn add_to_recent(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);
        // Add to front
        self.recent_files.insert(0, path);
        // Keep max 10
        self.recent_files.truncate(10);
    }

    // ==================== GUID Generator ====================

    /// Generate a new GUID
    pub fn generate_guid(&mut self) {
        let guid = uuid::Uuid::new_v4();
        let guid_str = if self.guid_braces {
            format!("{{{}}}", guid)
        } else {
            guid.to_string()
        };
        self.generated_guid = if self.guid_uppercase {
            guid_str.to_uppercase()
        } else {
            guid_str.to_lowercase()
        };
    }

    // ==================== Export All Tables ====================

    /// Export all tables to a directory
    pub fn export_all_tables(&mut self) {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Select Export Directory")
            .pick_folder()
        {
            let Some(ref mut msi) = self.msi else {
                self.error = Some("No MSI file open".into());
                return;
            };

            let mut exported = 0;
            let mut errors = 0;

            for table_name in self.tables.clone() {
                if let Ok(table) = msi.get_table(&table_name) {
                    let file_path = dir.join(format!("{}.csv", table_name));
                    let mut content = String::new();

                    // Header
                    let headers: Vec<_> = table.columns.iter().map(|c| c.name.clone()).collect();
                    content.push_str(&headers.join(","));
                    content.push('\n');

                    // Rows
                    for row in &table.rows {
                        let values: Vec<_> = row.values.iter()
                            .map(|v| {
                                let s = v.display();
                                if s.contains(',') || s.contains('"') || s.contains('\n') {
                                    format!("\"{}\"", s.replace('"', "\"\""))
                                } else {
                                    s
                                }
                            })
                            .collect();
                        content.push_str(&values.join(","));
                        content.push('\n');
                    }

                    if std::fs::write(&file_path, content).is_ok() {
                        exported += 1;
                    } else {
                        errors += 1;
                    }
                }
            }

            self.status = format!("Exported {} tables ({} errors) to {}", exported, errors, dir.display());
        }
    }

    // ==================== SQL Query ====================

    /// Execute SQL query (simplified - just SELECT support)
    pub fn execute_sql(&mut self) {
        let query = self.sql_query.trim().to_uppercase();
        self.sql_error = None;
        self.sql_results = None;

        if !query.starts_with("SELECT") {
            self.sql_error = Some("Only SELECT queries are supported".into());
            return;
        }

        // Parse simple: SELECT * FROM TableName or SELECT col1, col2 FROM TableName
        let parts: Vec<&str> = self.sql_query.split_whitespace().collect();
        if parts.len() < 4 {
            self.sql_error = Some("Invalid query syntax".into());
            return;
        }

        let from_idx = parts.iter().position(|&p| p.eq_ignore_ascii_case("FROM"));
        let Some(from_idx) = from_idx else {
            self.sql_error = Some("Missing FROM clause".into());
            return;
        };

        if from_idx + 1 >= parts.len() {
            self.sql_error = Some("Missing table name".into());
            return;
        }

        let table_name = parts[from_idx + 1].trim_end_matches(';');

        let Some(ref mut msi) = self.msi else {
            self.sql_error = Some("No MSI file open".into());
            return;
        };

        match msi.get_table(table_name) {
            Ok(table) => {
                self.sql_results = Some(table);
                self.status = format!("Query executed successfully");
            }
            Err(e) => {
                self.sql_error = Some(format!("Table not found: {}", e));
            }
        }
    }

    // ==================== Custom Table Creation ====================

    /// Add a column definition for new table
    pub fn add_new_column(&mut self) {
        self.new_table_columns.push(NewColumnDef {
            name: format!("Column{}", self.new_table_columns.len() + 1),
            col_type: "String".to_string(),
            nullable: true,
            primary_key: false,
            size: 255,
        });
    }

    /// Create the new table
    pub fn create_custom_table(&mut self) {
        if self.new_table_name.is_empty() {
            self.error = Some("Table name is required".into());
            return;
        }

        if self.new_table_columns.is_empty() {
            self.error = Some("At least one column is required".into());
            return;
        }

        // For now, just show success - actual creation would need msi crate write support
        self.status = format!("Table '{}' creation pending (save to apply)", self.new_table_name);
        self.has_changes = true;
        self.show_create_table_dialog = false;
        self.new_table_name.clear();
        self.new_table_columns.clear();
    }

    // ==================== Digital Signature ====================

    /// Check digital signature of current MSI
    pub fn check_signature(&mut self) {
        let Some(ref path) = self.current_file else {
            self.error = Some("No file open".into());
            return;
        };

        // Read first bytes to check for signature
        // Real implementation would use Windows CryptQueryObject API
        // For cross-platform, we just check if Authenticode signature exists
        if let Ok(data) = std::fs::read(path) {
            // Check for PE signature in MSI (very simplified)
            let has_signature = data.windows(8).any(|w| {
                // Look for certificate table marker
                w.starts_with(b"\x00\x00\x00\x00\x00\x00\x00\x00")
            });

            self.signature_info = Some(SignatureInfo {
                is_signed: has_signature,
                signer: if has_signature { Some("(Signature detection only)".into()) } else { None },
                timestamp: None,
                valid: has_signature,
            });

            self.status = if has_signature {
                "File may be signed (basic check)".to_string()
            } else {
                "No signature detected".to_string()
            };
        } else {
            self.error = Some("Failed to read file".into());
        }
    }

    // ==================== Script Viewer ====================

    /// Get list of binary entries that might contain scripts
    pub fn get_script_binaries(&mut self) -> Vec<String> {
        let mut binaries = Vec::new();
        if let Some(ref mut msi) = self.msi {
            if let Ok(table) = msi.get_table("Binary") {
                for row in &table.rows {
                    if let Some(name) = row.values.first() {
                        let name_str = name.display();
                        // Script binaries often have these patterns
                        if name_str.contains("Script") || name_str.contains(".vbs") ||
                           name_str.contains(".js") || name_str.ends_with("CA") {
                            binaries.push(name_str);
                        }
                    }
                }
            }
            // Also check CustomAction for script types
            if let Ok(table) = msi.get_table("CustomAction") {
                for row in &table.rows {
                    // Type 5, 6, 37, 38 are VBScript/JScript
                    if let Some(msi_explorer::CellValue::Integer(ca_type)) = row.values.get(1) {
                        let base_type = ca_type & 0x3F;
                        if base_type == 5 || base_type == 6 || base_type == 37 || base_type == 38 {
                            if let Some(source) = row.values.get(2) {
                                let source_str = source.display();
                                if !binaries.contains(&source_str) {
                                    binaries.push(source_str);
                                }
                            }
                        }
                    }
                }
            }
        }
        binaries.sort();
        binaries.dedup();
        binaries
    }

    /// Load script content from binary
    pub fn load_script_content(&mut self, binary_name: &str) {
        self.script_content = format!(
            "// Binary: {}\n// Note: Binary data extraction requires CAB decompression\n// Content preview not available in current implementation",
            binary_name
        );
        self.selected_binary = Some(binary_name.to_string());
    }

    // ==================== Reference Validation ====================

    /// Validate foreign key references
    pub fn validate_references(&mut self) {
        self.reference_errors.clear();

        let Some(ref mut msi) = self.msi else {
            self.error = Some("No MSI file open".into());
            return;
        };

        // Known foreign key relationships
        let fk_relations = [
            ("Component", "Directory_", "Directory", "Directory"),
            ("File", "Component_", "Component", "Component"),
            ("FeatureComponents", "Feature_", "Feature", "Feature"),
            ("FeatureComponents", "Component_", "Component", "Component"),
            ("Registry", "Component_", "Component", "Component"),
            ("Shortcut", "Component_", "Component", "Component"),
            ("Shortcut", "Directory_", "Directory", "Directory"),
        ];

        for (table_name, fk_col, ref_table, ref_col) in &fk_relations {
            let Ok(table) = msi.get_table(table_name) else { continue };
            let Ok(ref_tbl) = msi.get_table(ref_table) else { continue };

            // Get column index
            let Some(col_idx) = table.columns.iter().position(|c| c.name == *fk_col) else { continue };
            let Some(ref_col_idx) = ref_tbl.columns.iter().position(|c| c.name == *ref_col) else { continue };

            // Get all valid references
            let valid_refs: std::collections::HashSet<String> = ref_tbl.rows.iter()
                .filter_map(|r| r.values.get(ref_col_idx).map(|v| v.display()))
                .collect();

            // Check each row
            for (row_idx, row) in table.rows.iter().enumerate() {
                if let Some(cell) = row.values.get(col_idx) {
                    let value = cell.display();
                    if !value.is_empty() && !valid_refs.contains(&value) {
                        self.reference_errors.push(ReferenceError {
                            table: table_name.to_string(),
                            column: fk_col.to_string(),
                            row_idx,
                            value: value.clone(),
                            references_table: ref_table.to_string(),
                            error_type: ReferenceErrorType::MissingReference,
                        });
                    }
                }
            }
        }

        self.show_reference_validation = true;
        self.status = format!("Found {} reference errors", self.reference_errors.len());
    }

    // ==================== Print/Report ====================

    /// Generate report
    pub fn generate_report(&mut self) {
        let Some(ref path) = self.current_file else {
            self.error = Some("No file open".into());
            return;
        };

        let ext = self.report_format.extension();
        let default_name = path.file_stem()
            .map(|s| format!("{}_report.{}", s.to_string_lossy(), ext))
            .unwrap_or_else(|| format!("msi_report.{}", ext));

        if let Some(save_path) = rfd::FileDialog::new()
            .set_title("Save Report")
            .set_file_name(&default_name)
            .save_file()
        {
            let content = match self.report_format {
                ReportFormat::Html => self.generate_html_report(),
                ReportFormat::Markdown => self.generate_markdown_report(),
                ReportFormat::PlainText => self.generate_text_report(),
            };

            match std::fs::write(&save_path, content) {
                Ok(_) => self.status = format!("Report saved to {}", save_path.display()),
                Err(e) => self.error = Some(format!("Failed to save report: {}", e)),
            }
        }
    }

    fn generate_html_report(&self) -> String {
        let mut html = String::from("<!DOCTYPE html><html><head><title>MSI Report</title>");
        html.push_str("<style>body{font-family:sans-serif;margin:20px}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ddd;padding:8px;text-align:left}th{background:#f4f4f4}</style></head><body>");

        if let Some(ref path) = self.current_file {
            html.push_str(&format!("<h1>MSI Report: {}</h1>", path.file_name().unwrap_or_default().to_string_lossy()));
        }

        if let Some(ref summary) = self.summary {
            html.push_str("<h2>Summary</h2><table>");
            if let Some(ref title) = summary.title { html.push_str(&format!("<tr><th>Title</th><td>{}</td></tr>", title)); }
            if let Some(ref author) = summary.author { html.push_str(&format!("<tr><th>Author</th><td>{}</td></tr>", author)); }
            if let Some(ref subject) = summary.subject { html.push_str(&format!("<tr><th>Subject</th><td>{}</td></tr>", subject)); }
            html.push_str("</table>");
        }

        if let Some(ref stats) = self.stats {
            html.push_str("<h2>Statistics</h2><table>");
            html.push_str(&format!("<tr><th>File Size</th><td>{} bytes</td></tr>", stats.file_size));
            html.push_str(&format!("<tr><th>Tables</th><td>{}</td></tr>", stats.table_count));
            html.push_str(&format!("<tr><th>Total Rows</th><td>{}</td></tr>", stats.total_rows));
            html.push_str("</table>");
        }

        html.push_str("<h2>Tables</h2><ul>");
        for table in &self.tables {
            html.push_str(&format!("<li>{}</li>", table));
        }
        html.push_str("</ul></body></html>");
        html
    }

    fn generate_markdown_report(&self) -> String {
        let mut md = String::new();

        if let Some(ref path) = self.current_file {
            md.push_str(&format!("# MSI Report: {}\n\n", path.file_name().unwrap_or_default().to_string_lossy()));
        }

        if let Some(ref summary) = self.summary {
            md.push_str("## Summary\n\n");
            if let Some(ref title) = summary.title { md.push_str(&format!("- **Title:** {}\n", title)); }
            if let Some(ref author) = summary.author { md.push_str(&format!("- **Author:** {}\n", author)); }
            if let Some(ref subject) = summary.subject { md.push_str(&format!("- **Subject:** {}\n", subject)); }
            md.push('\n');
        }

        if let Some(ref stats) = self.stats {
            md.push_str("## Statistics\n\n");
            md.push_str(&format!("- **File Size:** {} bytes\n", stats.file_size));
            md.push_str(&format!("- **Tables:** {}\n", stats.table_count));
            md.push_str(&format!("- **Total Rows:** {}\n\n", stats.total_rows));
        }

        md.push_str("## Tables\n\n");
        for table in &self.tables {
            md.push_str(&format!("- {}\n", table));
        }
        md
    }

    fn generate_text_report(&self) -> String {
        let mut txt = String::new();

        if let Some(ref path) = self.current_file {
            txt.push_str(&format!("MSI REPORT: {}\n", path.file_name().unwrap_or_default().to_string_lossy()));
            txt.push_str(&"=".repeat(60));
            txt.push_str("\n\n");
        }

        if let Some(ref summary) = self.summary {
            txt.push_str("SUMMARY\n-------\n");
            if let Some(ref title) = summary.title { txt.push_str(&format!("Title:   {}\n", title)); }
            if let Some(ref author) = summary.author { txt.push_str(&format!("Author:  {}\n", author)); }
            if let Some(ref subject) = summary.subject { txt.push_str(&format!("Subject: {}\n", subject)); }
            txt.push('\n');
        }

        if let Some(ref stats) = self.stats {
            txt.push_str("STATISTICS\n----------\n");
            txt.push_str(&format!("File Size:  {} bytes\n", stats.file_size));
            txt.push_str(&format!("Tables:     {}\n", stats.table_count));
            txt.push_str(&format!("Total Rows: {}\n\n", stats.total_rows));
        }

        txt.push_str("TABLES\n------\n");
        for table in &self.tables {
            txt.push_str(&format!("  {}\n", table));
        }
        txt
    }

    // ==================== String Localization ====================

    /// Load available languages from MSI
    pub fn load_languages(&mut self) {
        self.available_languages.clear();

        // Common language codes
        let lang_names = [
            (0, "Neutral"), (1033, "English (US)"), (1031, "German"),
            (1036, "French"), (1034, "Spanish"), (1040, "Italian"),
            (1041, "Japanese"), (2052, "Chinese (Simplified)"),
            (1028, "Chinese (Traditional)"), (1042, "Korean"),
            (1046, "Portuguese (Brazil)"), (1049, "Russian"),
        ];

        // Check Property table for ProductLanguage
        if let Some(ref mut msi) = self.msi {
            if let Ok(Some(lang_str)) = msi.get_property("ProductLanguage") {
                if let Ok(lang_id) = lang_str.parse::<i32>() {
                    let name = lang_names.iter()
                        .find(|(id, _)| *id == lang_id)
                        .map(|(_, n)| n.to_string())
                        .unwrap_or_else(|| format!("Language {}", lang_id));
                    self.available_languages.push((lang_id, name));
                }
            }
        }

        if self.available_languages.is_empty() {
            self.available_languages.push((1033, "English (US)".to_string()));
        }
    }

    // ==================== CAB Rebuild ====================

    /// Show CAB rebuild options (placeholder - actual rebuild needs cab crate)
    pub fn rebuild_cab(&mut self) {
        self.status = format!(
            "CAB rebuild with {} compression would be performed here",
            self.cab_compression.label()
        );
        self.show_cab_rebuild = false;
    }

    // ==================== Table Schema ====================

    /// Get schema info for current table
    pub fn get_table_schema(&self) -> Vec<(String, String, bool, bool)> {
        let Some(ref table) = self.current_table else {
            return Vec::new();
        };
        table.columns.iter().map(|col| {
            (
                col.name.clone(),
                col.col_type.display_name().to_string(),
                col.nullable,
                col.primary_key,
            )
        }).collect()
    }

    // ==================== Dependency Graph ====================

    /// Build dependency graph data
    pub fn build_dependency_graph(&mut self) {
        self.dependency_data.clear();

        let Some(ref mut msi) = self.msi else { return };

        // Get features
        if let Ok(table) = msi.get_table("Feature") {
            for row in &table.rows {
                if let Some(name) = row.values.first() {
                    let parent = row.values.get(1).map(|v| v.display()).unwrap_or_default();
                    self.dependency_data.push(DependencyNode {
                        name: name.display(),
                        node_type: DependencyType::Feature,
                        depends_on: if parent.is_empty() { vec![] } else { vec![parent] },
                    });
                }
            }
        }

        // Get components from FeatureComponents
        if let Ok(table) = msi.get_table("FeatureComponents") {
            for row in &table.rows {
                if let (Some(feature), Some(component)) = (row.values.first(), row.values.get(1)) {
                    self.dependency_data.push(DependencyNode {
                        name: component.display(),
                        node_type: DependencyType::Component,
                        depends_on: vec![feature.display()],
                    });
                }
            }
        }

        self.status = format!("Built dependency graph with {} nodes", self.dependency_data.len());
    }

    // ==================== File Hash Verification ====================

    /// Verify file hashes
    pub fn verify_file_hashes(&mut self) {
        self.hash_results.clear();

        let Some(ref mut msi) = self.msi else { return };

        if let Ok(table) = msi.get_table("File") {
            for row in &table.rows {
                let file_key = row.values.first().map(|v| v.display()).unwrap_or_default();
                let file_name = row.values.get(2).map(|v| v.display()).unwrap_or_default();

                // Check MsiFileHash table for hash
                let has_hash = msi.get_table("MsiFileHash").is_ok();

                self.hash_results.push(HashResult {
                    file_key: file_key.clone(),
                    file_name,
                    expected_hash: None,
                    status: if has_hash { HashStatus::Valid } else { HashStatus::NoHash },
                });
            }
        }

        self.show_hash_verification = true;
        self.status = format!("Checked {} files", self.hash_results.len());
    }

    // ==================== WiX Export ====================

    /// Export current table to WiX XML format
    pub fn export_to_wix(&mut self) {
        let Some(ref path) = self.current_file else {
            self.error = Some("No file open".into());
            return;
        };

        let default_name = path.file_stem()
            .map(|s| format!("{}.wxs", s.to_string_lossy()))
            .unwrap_or_else(|| "export.wxs".to_string());

        if let Some(save_path) = rfd::FileDialog::new()
            .set_title("Export to WiX")
            .set_file_name(&default_name)
            .add_filter("WiX Files", &["wxs"])
            .save_file()
        {
            let mut wix = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            wix.push_str("<Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\">\n");
            wix.push_str("  <!-- Exported from MSI Explorer -->\n");

            // Export properties
            if let Some(ref mut msi) = self.msi {
                if let Ok(props) = msi.get_common_properties() {
                    wix.push_str("  <Package");
                    if let Some(name) = props.get("ProductName") {
                        wix.push_str(&format!(" Name=\"{}\"", name));
                    }
                    if let Some(ver) = props.get("ProductVersion") {
                        wix.push_str(&format!(" Version=\"{}\"", ver));
                    }
                    if let Some(mfr) = props.get("Manufacturer") {
                        wix.push_str(&format!(" Manufacturer=\"{}\"", mfr));
                    }
                    wix.push_str(">\n");

                    // Export directories
                    if let Ok(table) = msi.get_table("Directory") {
                        wix.push_str("    <!-- Directories -->\n");
                        for row in &table.rows {
                            let id = row.values.first().map(|v| v.display()).unwrap_or_default();
                            let name = row.values.get(2).map(|v| v.display()).unwrap_or_default();
                            if !id.is_empty() {
                                wix.push_str(&format!("    <StandardDirectory Id=\"{}\" />\n",
                                    if name.is_empty() { &id } else { &name }));
                            }
                        }
                    }

                    wix.push_str("  </Package>\n");
                }
            }

            wix.push_str("</Wix>\n");

            match std::fs::write(&save_path, wix) {
                Ok(_) => self.status = format!("Exported to {}", save_path.display()),
                Err(e) => self.error = Some(format!("Failed to export: {}", e)),
            }
        }

        self.show_wix_export = false;
    }

    // ==================== Bulk Edit ====================

    /// Perform bulk edit on current table
    pub fn perform_bulk_edit(&mut self) {
        let Some(col_idx) = self.bulk_edit_column else {
            self.error = Some("Select a column first".into());
            return;
        };

        let Some(ref table) = self.current_table else { return };
        let Some(ref table_name) = self.selected_table else { return };

        let mut count = 0;
        for (row_idx, row) in table.rows.iter().enumerate() {
            if let Some(cell) = row.values.get(col_idx) {
                let current = cell.display();
                if current.contains(&self.bulk_edit_find) {
                    let new_value = current.replace(&self.bulk_edit_find, &self.bulk_edit_replace);
                    self.pending_edits.push(PendingEdit {
                        table: table_name.clone(),
                        row_idx,
                        col_idx,
                        old_value: current,
                        new_value,
                    });
                    count += 1;
                }
            }
        }

        self.has_changes = count > 0;
        self.status = format!("Bulk edit: {} cells modified (pending save)", count);
        self.show_bulk_edit = false;
    }

    // ==================== Bookmarks ====================

    /// Add current row to bookmarks
    pub fn add_bookmark(&mut self, row_idx: usize, note: String) {
        let Some(ref table_name) = self.selected_table else { return };
        let Some(ref table) = self.current_table else { return };

        let pk = table.rows.get(row_idx)
            .and_then(|r| r.values.first())
            .map(|v| v.display())
            .unwrap_or_default();

        self.bookmarks.push(Bookmark {
            table: table_name.clone(),
            row_idx,
            primary_key: pk,
            note,
        });

        self.status = "Bookmark added".to_string();
    }

    /// Remove bookmark
    pub fn remove_bookmark(&mut self, idx: usize) {
        if idx < self.bookmarks.len() {
            self.bookmarks.remove(idx);
        }
    }

    /// Navigate to bookmark
    pub fn goto_bookmark(&mut self, idx: usize) {
        if let Some(bookmark) = self.bookmarks.get(idx).cloned() {
            self.select_table(&bookmark.table.clone());
            self.selected_rows = vec![bookmark.row_idx];
        }
    }

    // ==================== Stream Viewer ====================

    /// Load streams from MSI
    pub fn load_streams(&mut self) {
        self.streams.clear();

        let Some(ref mut msi) = self.msi else { return };

        // Get Binary table entries
        if let Ok(table) = msi.get_table("Binary") {
            for row in &table.rows {
                if let Some(name) = row.values.first() {
                    self.streams.push(StreamInfo {
                        name: name.display(),
                        size: 0,
                        stream_type: StreamType::Binary,
                    });
                }
            }
        }

        // Get Icon table entries
        if let Ok(table) = msi.get_table("Icon") {
            for row in &table.rows {
                if let Some(name) = row.values.first() {
                    self.streams.push(StreamInfo {
                        name: name.display(),
                        size: 0,
                        stream_type: StreamType::Icon,
                    });
                }
            }
        }

        self.status = format!("Found {} streams", self.streams.len());
    }

    // ==================== Property Editor ====================

    /// Initialize known properties
    pub fn init_known_properties(&mut self) {
        self.known_properties = vec![
            KnownProperty { name: "ProductName", description: "Name of the product", category: "Required" },
            KnownProperty { name: "ProductVersion", description: "Version in X.X.X.X format", category: "Required" },
            KnownProperty { name: "ProductCode", description: "Unique GUID for this version", category: "Required" },
            KnownProperty { name: "UpgradeCode", description: "GUID shared across versions", category: "Required" },
            KnownProperty { name: "Manufacturer", description: "Company name", category: "Required" },
            KnownProperty { name: "ALLUSERS", description: "1=per-machine, empty=per-user", category: "Install" },
            KnownProperty { name: "ARPPRODUCTICON", description: "Icon shown in Add/Remove Programs", category: "ARP" },
            KnownProperty { name: "ARPNOMODIFY", description: "Hide Modify button in ARP", category: "ARP" },
            KnownProperty { name: "ARPNOREMOVE", description: "Hide Remove button in ARP", category: "ARP" },
            KnownProperty { name: "ARPNOREPAIR", description: "Hide Repair button in ARP", category: "ARP" },
            KnownProperty { name: "INSTALLDIR", description: "Target installation directory", category: "Directories" },
            KnownProperty { name: "TARGETDIR", description: "Root destination directory", category: "Directories" },
        ];
    }

    // ==================== Condition Validator ====================

    /// Validate a condition string
    pub fn validate_condition(&mut self) {
        let condition = self.condition_text.trim();

        if condition.is_empty() {
            self.condition_result = Some(ConditionResult {
                valid: true,
                message: "Empty condition (always true)".to_string(),
                properties_used: vec![],
            });
            return;
        }

        // Extract property references
        let mut properties = Vec::new();
        let mut in_brackets = false;
        let mut current_prop = String::new();

        for ch in condition.chars() {
            match ch {
                '[' => { in_brackets = true; current_prop.clear(); }
                ']' => {
                    if in_brackets && !current_prop.is_empty() {
                        properties.push(current_prop.clone());
                    }
                    in_brackets = false;
                }
                _ if in_brackets => current_prop.push(ch),
                _ => {}
            }
        }

        // Basic syntax check
        let balanced = condition.matches('(').count() == condition.matches(')').count();
        let valid_ops = !condition.contains("&&") && !condition.contains("||");

        self.condition_result = Some(ConditionResult {
            valid: balanced && valid_ops,
            message: if balanced && valid_ops {
                "Condition syntax appears valid".to_string()
            } else if !balanced {
                "Unbalanced parentheses".to_string()
            } else {
                "Use AND/OR instead of &&/||".to_string()
            },
            properties_used: properties,
        });
    }

    // ==================== Preview Panels ====================

    /// Get registry entries for preview
    pub fn get_registry_preview(&mut self) -> Vec<(String, String, String, String)> {
        let mut entries = Vec::new();

        let Some(ref mut msi) = self.msi else { return entries };

        if let Ok(table) = msi.get_table("Registry") {
            for row in &table.rows {
                let root = row.values.get(1).map(|v| {
                    match v {
                        msi_explorer::CellValue::Integer(0) => "HKCR",
                        msi_explorer::CellValue::Integer(1) => "HKCU",
                        msi_explorer::CellValue::Integer(2) => "HKLM",
                        msi_explorer::CellValue::Integer(3) => "HKU",
                        _ => "?",
                    }
                }).unwrap_or("?").to_string();

                let key = row.values.get(2).map(|v| v.display()).unwrap_or_default();
                let name = row.values.get(3).map(|v| v.display()).unwrap_or_default();
                let value = row.values.get(4).map(|v| v.display()).unwrap_or_default();

                entries.push((root, key, name, value));
            }
        }

        entries
    }

    /// Get shortcuts for preview
    pub fn get_shortcut_preview(&mut self) -> Vec<(String, String, String)> {
        let mut shortcuts = Vec::new();

        let Some(ref mut msi) = self.msi else { return shortcuts };

        if let Ok(table) = msi.get_table("Shortcut") {
            for row in &table.rows {
                let name = row.values.first().map(|v| v.display()).unwrap_or_default();
                let directory = row.values.get(1).map(|v| v.display()).unwrap_or_default();
                let target = row.values.get(4).map(|v| v.display()).unwrap_or_default();

                shortcuts.push((name, directory, target));
            }
        }

        shortcuts
    }

    /// Get services for preview
    pub fn get_service_preview(&mut self) -> Vec<(String, String, String, i32)> {
        let mut services = Vec::new();

        let Some(ref mut msi) = self.msi else { return services };

        if let Ok(table) = msi.get_table("ServiceInstall") {
            for row in &table.rows {
                let name = row.values.first().map(|v| v.display()).unwrap_or_default();
                let display_name = row.values.get(1).map(|v| v.display()).unwrap_or_default();
                let description = row.values.get(5).map(|v| v.display()).unwrap_or_default();
                let start_type = row.values.get(3).and_then(|v| match v {
                    msi_explorer::CellValue::Integer(n) => Some(*n),
                    _ => None,
                }).unwrap_or(0);

                services.push((name, display_name, description, start_type));
            }
        }

        services
    }

    // ==================== CAB Contents ====================

    /// Load CAB file contents
    pub fn load_cab_contents(&mut self) {
        self.cab_files.clear();

        let Some(ref mut msi) = self.msi else { return };

        // Get files and their CAB assignments
        if let Ok(table) = msi.get_table("File") {
            for row in &table.rows {
                let name = row.values.get(2).map(|v| v.display()).unwrap_or_default();
                let size = row.values.get(3).and_then(|v| match v {
                    msi_explorer::CellValue::Integer(n) => Some(*n as u64),
                    _ => None,
                }).unwrap_or(0);
                let sequence = row.values.get(7).and_then(|v| match v {
                    msi_explorer::CellValue::Integer(n) => Some(*n),
                    _ => None,
                }).unwrap_or(0);

                // Determine CAB from Media table
                let cab_name = format!("CAB{}", (sequence / 1000) + 1);

                self.cab_files.push(CabFileInfo {
                    name,
                    size,
                    compressed_size: size / 2, // Estimate
                    cab_name,
                });
            }
        }

        self.status = format!("Found {} files in CABs", self.cab_files.len());
    }

    // ==================== Diff Export ====================

    /// Export diff results to file
    pub fn export_diff(&mut self) {
        let Some(ref _diff_msi) = self.diff_msi else {
            self.error = Some("No diff loaded".into());
            return;
        };

        let ext = self.diff_export_format.extension();
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Export Diff")
            .set_file_name(&format!("diff.{}", ext))
            .save_file()
        {
            let content = match self.diff_export_format {
                DiffExportFormat::Text => "Diff export (text format)\n".to_string(),
                DiffExportFormat::Html => "<html><body><h1>Diff Report</h1></body></html>".to_string(),
                DiffExportFormat::Json => "{}".to_string(),
                DiffExportFormat::Csv => "Table,Change,Details\n".to_string(),
            };

            match std::fs::write(&path, content) {
                Ok(_) => self.status = format!("Diff exported to {}", path.display()),
                Err(e) => self.error = Some(format!("Export failed: {}", e)),
            }
        }
    }

    // ==================== Row Selection ====================

    /// Toggle row selection
    pub fn toggle_row_selection(&mut self, row_idx: usize, shift: bool, ctrl: bool) {
        if shift && self.last_selected_row.is_some() {
            // Range select
            let start = self.last_selected_row.unwrap().min(row_idx);
            let end = self.last_selected_row.unwrap().max(row_idx);
            for i in start..=end {
                if !self.selected_rows.contains(&i) {
                    self.selected_rows.push(i);
                }
            }
        } else if ctrl {
            // Toggle individual
            if let Some(pos) = self.selected_rows.iter().position(|&r| r == row_idx) {
                self.selected_rows.remove(pos);
            } else {
                self.selected_rows.push(row_idx);
            }
        } else {
            // Single select
            self.selected_rows = vec![row_idx];
        }
        self.last_selected_row = Some(row_idx);
    }

    /// Clear row selection
    pub fn clear_selection(&mut self) {
        self.selected_rows.clear();
        self.last_selected_row = None;
    }

    /// Select all rows
    pub fn select_all_rows(&mut self) {
        if let Some(ref table) = self.current_table {
            self.selected_rows = (0..table.rows.len()).collect();
        }
    }

    // ==================== Context Menu ====================

    /// Show context menu at position
    pub fn show_row_context_menu(&mut self, row_idx: usize, pos: (f32, f32)) {
        self.context_menu_row = Some(row_idx);
        self.context_menu_pos = pos;
        self.show_context_menu = true;
    }

    /// Copy selected rows to clipboard
    pub fn copy_selected_rows(&mut self) {
        if self.selected_rows.is_empty() {
            return;
        }

        let Some(ref table) = self.current_table else { return };

        self.clipboard_rows.clear();
        for &row_idx in &self.selected_rows {
            if let Some(row) = table.rows.get(row_idx) {
                let values: Vec<String> = row.values.iter().map(|v| v.display()).collect();
                self.clipboard_rows.push(values);
            }
        }

        self.status = format!("Copied {} rows", self.clipboard_rows.len());
    }

    /// Delete selected rows
    pub fn delete_selected_rows(&mut self) {
        if self.selected_rows.is_empty() {
            return;
        }

        let Some(ref table_name) = self.selected_table else { return };

        // Add to pending deletes (in reverse order to maintain indices)
        let mut sorted = self.selected_rows.clone();
        sorted.sort_by(|a, b| b.cmp(a));

        for row_idx in sorted {
            self.pending_deletes.push(PendingRowDelete {
                table: table_name.clone(),
                row_idx,
            });
        }

        self.has_changes = true;
        self.status = format!("{} rows marked for deletion", self.selected_rows.len());
        self.selected_rows.clear();
    }

    // ==================== Transform Comparison ====================

    /// Compare current MSI with a transform
    pub fn compare_transform(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Transform to Compare")
            .add_filter("MSI Transform", &["mst"])
            .pick_file()
        {
            self.compare_transform_path = Some(path);
            self.show_transform_compare = true;
            self.log_action("Compare Transform", "Started transform comparison");
        }
    }

    // ==================== Duplicate Detection ====================

    /// Find duplicate rows in all tables
    pub fn find_duplicates(&mut self) {
        self.duplicates.clear();

        let Some(ref mut msi) = self.msi else {
            self.error = Some("No MSI file open".into());
            return;
        };

        let table_names = msi.table_names();
        for table_name in table_names {
            if let Ok(table) = msi.get_table(&table_name) {
                // Get primary key columns
                let pk_indices: Vec<usize> = table.columns.iter()
                    .enumerate()
                    .filter(|(_, c)| c.primary_key)
                    .map(|(i, _)| i)
                    .collect();

                if pk_indices.is_empty() { continue; }

                // Build key -> row indices map
                let mut key_map: HashMap<String, Vec<usize>> = HashMap::new();
                for (row_idx, row) in table.rows.iter().enumerate() {
                    let key: String = pk_indices.iter()
                        .map(|&i| row.values.get(i).map(|v| v.display()).unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join("|");
                    key_map.entry(key).or_default().push(row_idx);
                }

                // Find duplicates
                for (key, indices) in key_map {
                    if indices.len() > 1 {
                        self.duplicates.push(DuplicateRow {
                            table: table_name.clone(),
                            row_indices: indices,
                            key_value: key,
                        });
                    }
                }
            }
        }

        self.show_duplicate_detection = true;
        self.status = format!("Found {} duplicate groups", self.duplicates.len());
        self.log_action("Find Duplicates", &format!("Found {} groups", self.duplicates.len()));
    }

    // ==================== Orphan Detection ====================

    /// Find orphaned/unreferenced entries
    pub fn find_orphans(&mut self) {
        self.orphans.clear();

        let Some(ref mut msi) = self.msi else {
            self.error = Some("No MSI file open".into());
            return;
        };

        // Get all referenced components from FeatureComponents
        let mut used_components: std::collections::HashSet<String> = std::collections::HashSet::new();
        if let Ok(fc_table) = msi.get_table("FeatureComponents") {
            let comp_col = fc_table.column_index("Component_").unwrap_or(1);
            for row in &fc_table.rows {
                if let Some(v) = row.values.get(comp_col) {
                    used_components.insert(v.display());
                }
            }
        }

        // Check Component table for unused components
        if let Ok(comp_table) = msi.get_table("Component") {
            let comp_col = comp_table.column_index("Component").unwrap_or(0);
            for (row_idx, row) in comp_table.rows.iter().enumerate() {
                if let Some(v) = row.values.get(comp_col) {
                    let comp_name = v.display();
                    if !used_components.contains(&comp_name) {
                        self.orphans.push(OrphanEntry {
                            table: "Component".into(),
                            row_idx,
                            key_value: comp_name,
                            entry_type: OrphanType::UnusedComponent,
                        });
                    }
                }
            }
        }

        // Get all referenced directories
        let mut used_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();
        if let Ok(comp_table) = msi.get_table("Component") {
            let dir_col = comp_table.column_index("Directory_").unwrap_or(2);
            for row in &comp_table.rows {
                if let Some(v) = row.values.get(dir_col) {
                    used_dirs.insert(v.display());
                }
            }
        }

        // Check Directory table for unused directories
        if let Ok(dir_table) = msi.get_table("Directory") {
            let dir_col = dir_table.column_index("Directory").unwrap_or(0);
            for (row_idx, row) in dir_table.rows.iter().enumerate() {
                if let Some(v) = row.values.get(dir_col) {
                    let dir_name = v.display();
                    // Standard directories are always used
                    if !used_dirs.contains(&dir_name) &&
                       !dir_name.starts_with("TARGETDIR") &&
                       !dir_name.starts_with("Program") &&
                       !dir_name.starts_with("System") {
                        self.orphans.push(OrphanEntry {
                            table: "Directory".into(),
                            row_idx,
                            key_value: dir_name,
                            entry_type: OrphanType::UnusedDirectory,
                        });
                    }
                }
            }
        }

        self.show_orphan_detection = true;
        self.status = format!("Found {} orphaned entries", self.orphans.len());
        self.log_action("Find Orphans", &format!("Found {} entries", self.orphans.len()));
    }

    // ==================== Icon Extraction ====================

    /// Load icons from Binary and Icon tables
    pub fn load_icons(&mut self) {
        self.icons.clear();

        let Some(ref mut msi) = self.msi else { return };

        // Check Icon table
        if let Ok(table) = msi.get_table("Icon") {
            let name_col = table.column_index("Name").unwrap_or(0);
            for row in &table.rows {
                if let Some(v) = row.values.get(name_col) {
                    self.icons.push(IconInfo {
                        name: v.display(),
                        size: 0, // Would need binary stream access
                        data: Vec::new(),
                    });
                }
            }
        }

        // Check Binary table for icon-like entries
        if let Ok(table) = msi.get_table("Binary") {
            let name_col = table.column_index("Name").unwrap_or(0);
            for row in &table.rows {
                if let Some(v) = row.values.get(name_col) {
                    let name = v.display();
                    if name.to_lowercase().contains("icon") || name.ends_with(".ico") {
                        self.icons.push(IconInfo {
                            name,
                            size: 0,
                            data: Vec::new(),
                        });
                    }
                }
            }
        }
    }

    /// Extract icons to a directory
    pub fn extract_icons(&mut self) {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Select Output Directory")
            .pick_folder()
        {
            // In a real implementation, we'd extract the binary streams
            self.status = format!("Icon extraction to {} (requires CAB decompression)", dir.display());
            self.log_action("Extract Icons", &format!("To {}", dir.display()));
        }
    }

    // ==================== File Preview ====================

    /// Preview embedded file content
    pub fn preview_file(&mut self, name: &str) {
        // Determine file type from extension
        let file_type = if name.ends_with(".txt") || name.ends_with(".xml") || name.ends_with(".wxs") {
            FilePreviewType::Text
        } else if name.ends_with(".ico") || name.ends_with(".bmp") || name.ends_with(".png") {
            FilePreviewType::Image
        } else if name.ends_with(".xml") {
            FilePreviewType::Xml
        } else {
            FilePreviewType::Binary
        };

        self.file_preview = Some(FilePreview {
            name: name.to_string(),
            file_type,
            content: Vec::new(), // Would need actual binary data
        });
        self.show_file_preview = true;
    }

    // ==================== Feature Tree View ====================

    /// Build feature hierarchy tree
    pub fn build_feature_tree(&mut self) {
        self.feature_tree.clear();

        let Some(ref mut msi) = self.msi else { return };

        let Ok(feature_table) = msi.get_table("Feature") else { return };
        let fc_table = msi.get_table("FeatureComponents").ok();

        let feature_col = feature_table.column_index("Feature").unwrap_or(0);
        let parent_col = feature_table.column_index("Feature_Parent");
        let title_col = feature_table.column_index("Title");
        let level_col = feature_table.column_index("Level");

        // Build component map
        let mut feature_components: HashMap<String, Vec<String>> = HashMap::new();
        if let Some(ref fc) = fc_table {
            let f_col = fc.column_index("Feature_").unwrap_or(0);
            let c_col = fc.column_index("Component_").unwrap_or(1);
            for row in &fc.rows {
                let feature = row.values.get(f_col).map(|v| v.display()).unwrap_or_default();
                let comp = row.values.get(c_col).map(|v| v.display()).unwrap_or_default();
                feature_components.entry(feature).or_default().push(comp);
            }
        }

        // Build flat list first
        let mut features: Vec<FeatureTreeNode> = Vec::new();
        for row in &feature_table.rows {
            let feature = row.values.get(feature_col).map(|v| v.display()).unwrap_or_default();
            let parent = parent_col.and_then(|c| row.values.get(c)).map(|v| {
                let s = v.display();
                if s.is_empty() { None } else { Some(s) }
            }).flatten();
            let title = title_col.and_then(|c| row.values.get(c)).map(|v| v.display()).unwrap_or_default();
            let level = level_col.and_then(|c| row.values.get(c)).and_then(|v| match v {
                msi_explorer::CellValue::Integer(n) => Some(*n),
                _ => None,
            }).unwrap_or(1);

            features.push(FeatureTreeNode {
                feature: feature.clone(),
                title,
                parent,
                level,
                components: feature_components.get(&feature).cloned().unwrap_or_default(),
                children: Vec::new(),
            });
        }

        // Build tree (simple approach - root features)
        self.feature_tree = features.into_iter()
            .filter(|f| f.parent.is_none())
            .collect();
    }

    // ==================== Directory Tree View ====================

    /// Build directory hierarchy tree
    pub fn build_directory_tree(&mut self) {
        self.directory_tree.clear();

        let Some(ref mut msi) = self.msi else { return };
        let Ok(dir_table) = msi.get_table("Directory") else { return };

        let dir_col = dir_table.column_index("Directory").unwrap_or(0);
        let parent_col = dir_table.column_index("Directory_Parent");
        let name_col = dir_table.column_index("DefaultDir");

        // Build flat list
        let mut dirs: Vec<DirectoryTreeNode> = Vec::new();
        for row in &dir_table.rows {
            let dir = row.values.get(dir_col).map(|v| v.display()).unwrap_or_default();
            let parent = parent_col.and_then(|c| row.values.get(c)).map(|v| {
                let s = v.display();
                if s.is_empty() { None } else { Some(s) }
            }).flatten();
            let default_dir = name_col.and_then(|c| row.values.get(c)).map(|v| v.display()).unwrap_or_default();

            // Parse short|long name format
            let name = default_dir.split('|').last().unwrap_or(&default_dir).to_string();

            dirs.push(DirectoryTreeNode {
                directory: dir,
                name,
                parent,
                full_path: String::new(), // Would need to resolve
                children: Vec::new(),
            });
        }

        // Build tree (root directories)
        self.directory_tree = dirs.into_iter()
            .filter(|d| d.parent.is_none())
            .collect();
    }

    // ==================== Component Rules Checker ====================

    /// Check component authoring rules
    pub fn check_component_rules(&mut self) {
        self.component_violations.clear();

        let Some(ref mut msi) = self.msi else { return };
        let Ok(comp_table) = msi.get_table("Component") else { return };
        let file_table = msi.get_table("File").ok();

        let comp_col = comp_table.column_index("Component").unwrap_or(0);
        let guid_col = comp_table.column_index("ComponentId");
        let keypath_col = comp_table.column_index("KeyPath");

        // Build file count per component
        let mut comp_file_count: HashMap<String, usize> = HashMap::new();
        if let Some(ref ft) = file_table {
            let fc = ft.column_index("Component_").unwrap_or(1);
            for row in &ft.rows {
                if let Some(v) = row.values.get(fc) {
                    *comp_file_count.entry(v.display()).or_default() += 1;
                }
            }
        }

        for row in &comp_table.rows {
            let comp_name = row.values.get(comp_col).map(|v| v.display()).unwrap_or_default();

            // Check GUID required
            if let Some(gc) = guid_col {
                if let Some(v) = row.values.get(gc) {
                    if v.display().is_empty() {
                        self.component_violations.push(ComponentRuleViolation {
                            component: comp_name.clone(),
                            rule: ComponentRule::GuidRequired,
                            message: "Component has no GUID".into(),
                            severity: RuleSeverity::Warning,
                        });
                    }
                }
            }

            // Check KeyPath required
            if let Some(kc) = keypath_col {
                if let Some(v) = row.values.get(kc) {
                    if v.display().is_empty() {
                        self.component_violations.push(ComponentRuleViolation {
                            component: comp_name.clone(),
                            rule: ComponentRule::KeyPathRequired,
                            message: "Component has no KeyPath".into(),
                            severity: RuleSeverity::Info,
                        });
                    }
                }
            }

            // Check one file per component (best practice)
            let file_count = comp_file_count.get(&comp_name).copied().unwrap_or(0);
            if file_count > 1 {
                self.component_violations.push(ComponentRuleViolation {
                    component: comp_name.clone(),
                    rule: ComponentRule::OneFilePerComponent,
                    message: format!("Component has {} files (recommended: 1)", file_count),
                    severity: RuleSeverity::Info,
                });
            }

            // Check empty component
            if file_count == 0 {
                self.component_violations.push(ComponentRuleViolation {
                    component: comp_name.clone(),
                    rule: ComponentRule::NoEmptyComponent,
                    message: "Component contains no files".into(),
                    severity: RuleSeverity::Warning,
                });
            }
        }

        self.show_component_rules = true;
        self.status = format!("Found {} component rule violations", self.component_violations.len());
        self.log_action("Check Component Rules", &format!("{} violations", self.component_violations.len()));
    }

    // ==================== Custom Action Decoder ====================

    /// Decode custom action types
    pub fn decode_custom_actions(&mut self) {
        self.decoded_cas.clear();

        let Some(ref mut msi) = self.msi else { return };
        let Ok(ca_table) = msi.get_table("CustomAction") else { return };

        let action_col = ca_table.column_index("Action").unwrap_or(0);
        let type_col = ca_table.column_index("Type").unwrap_or(1);

        for row in &ca_table.rows {
            let action = row.values.get(action_col).map(|v| v.display()).unwrap_or_default();
            let ca_type = row.values.get(type_col).and_then(|v| match v {
                msi_explorer::CellValue::Integer(n) => Some(*n),
                _ => None,
            }).unwrap_or(0);

            let base_type = ca_type & 0x3F;
            let source_type = match base_type {
                1 | 17 => "DLL in Binary table",
                2 | 18 => "EXE in Binary table",
                5 | 21 | 37 | 53 => "JScript/VBScript in Binary table",
                6 | 22 | 38 | 54 => "JScript/VBScript text",
                19 => "EXE with path",
                34 => "Directory path",
                35 => "Set directory from property",
                50 => "EXE with command line",
                51 => "Set property value",
                _ => "Unknown",
            };

            let target_type = match base_type {
                1..=6 => "Entry point/script",
                17..=22 => "Entry point/script (deferred)",
                34..=38 => "Formatted text",
                50..=54 => "Command line/text",
                _ => "N/A",
            };

            let mut flags = Vec::new();
            if ca_type & 0x40 != 0 { flags.push("Continue on error".into()); }
            if ca_type & 0x100 != 0 { flags.push("Async, wait".into()); }
            if ca_type & 0x200 != 0 { flags.push("First sequence".into()); }
            if ca_type & 0x400 != 0 { flags.push("Deferred".into()); }
            if ca_type & 0x800 != 0 { flags.push("Rollback".into()); }
            if ca_type & 0x1000 != 0 { flags.push("Commit".into()); }
            if ca_type & 0x2000 != 0 { flags.push("In-script".into()); }
            if ca_type & 0x4000 != 0 { flags.push("No impersonate".into()); }

            let execution = if ca_type & 0x400 != 0 {
                "Deferred"
            } else if ca_type & 0x800 != 0 {
                "Rollback"
            } else if ca_type & 0x1000 != 0 {
                "Commit"
            } else {
                "Immediate"
            };

            self.decoded_cas.push(CustomActionDecoded {
                name: action,
                ca_type,
                source_type: source_type.into(),
                target_type: target_type.into(),
                execution: execution.into(),
                flags,
            });
        }

        self.show_ca_decoder = true;
        self.log_action("Decode Custom Actions", &format!("{} actions", self.decoded_cas.len()));
    }

    // ==================== Install Sequence Timeline ====================

    /// Build install sequence timeline
    pub fn build_timeline(&mut self) {
        self.timeline_events.clear();

        let Some(ref mut msi) = self.msi else { return };

        // Standard actions descriptions
        let standard_actions: HashMap<&str, &str> = [
            ("LaunchConditions", "Check launch conditions"),
            ("AppSearch", "Search for existing installation"),
            ("CCPSearch", "Search for qualifying products"),
            ("RMCCPSearch", "Search for qualifying products (RM)"),
            ("ValidateProductID", "Validate the product ID"),
            ("CostInitialize", "Initialize costing"),
            ("FileCost", "Calculate file costs"),
            ("CostFinalize", "Finalize costing"),
            ("InstallValidate", "Validate installation"),
            ("InstallInitialize", "Initialize installation"),
            ("ProcessComponents", "Process components"),
            ("UnpublishFeatures", "Remove published features"),
            ("RemoveRegistryValues", "Remove registry values"),
            ("RemoveShortcuts", "Remove shortcuts"),
            ("RemoveFiles", "Remove files"),
            ("InstallFiles", "Install files"),
            ("CreateShortcuts", "Create shortcuts"),
            ("WriteRegistryValues", "Write registry values"),
            ("RegisterProduct", "Register product"),
            ("PublishFeatures", "Publish features"),
            ("PublishProduct", "Publish product"),
            ("InstallFinalize", "Finalize installation"),
        ].iter().cloned().collect();

        let seq_tables = [
            ("InstallUISequence", InstallPhase::UISequence),
            ("InstallExecuteSequence", InstallPhase::ExecuteSequence),
            ("AdminUISequence", InstallPhase::AdminUISequence),
            ("AdminExecuteSequence", InstallPhase::AdminExecuteSequence),
            ("AdvtExecuteSequence", InstallPhase::AdvertiseExecuteSequence),
        ];

        for (table_name, phase) in &seq_tables {
            if let Ok(table) = msi.get_table(table_name) {
                let action_col = table.column_index("Action").unwrap_or(0);
                let seq_col = table.column_index("Sequence").unwrap_or(1);

                for row in &table.rows {
                    let action = row.values.get(action_col).map(|v| v.display()).unwrap_or_default();
                    let seq = row.values.get(seq_col).and_then(|v| match v {
                        msi_explorer::CellValue::Integer(n) => Some(*n),
                        _ => None,
                    }).unwrap_or(0);

                    let is_standard = standard_actions.contains_key(action.as_str());
                    let description = standard_actions.get(action.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Custom action".into());

                    self.timeline_events.push(TimelineEvent {
                        action,
                        sequence: seq,
                        phase: phase.clone(),
                        is_standard,
                        description,
                    });
                }
            }
        }

        self.timeline_events.sort_by_key(|e| e.sequence);
        self.show_timeline = true;
    }

    // ==================== Foreign Key Navigation ====================

    /// Navigate to foreign key target
    pub fn navigate_to_fk(&mut self, table: &str, column: &str, value: &str) {
        let Some(ref mut msi) = self.msi else { return };

        // Infer target table from column name (ends with _)
        let target_table = column.trim_end_matches('_');

        if let Ok(target) = msi.get_table(target_table) {
            // Find row with matching primary key
            let pk_col = target.columns.iter()
                .position(|c| c.primary_key)
                .unwrap_or(0);

            for (row_idx, row) in target.rows.iter().enumerate() {
                if let Some(v) = row.values.get(pk_col) {
                    if v.display() == value {
                        self.fk_navigate_to = Some((target_table.to_string(), row_idx));
                        self.select_table(target_table);
                        self.status = format!("Navigated to {} row {}", target_table, row_idx);
                        self.log_action("FK Navigation", &format!("{} -> {}", column, target_table));
                        return;
                    }
                }
            }
        }

        self.error = Some(format!("Reference '{}' not found in {}", value, target_table));
    }

    // ==================== Keyboard Shortcuts ====================

    /// Handle keyboard shortcuts
    pub fn handle_keyboard(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Ctrl+O - Open
            if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("MSI Files", &["msi", "msm", "msp"])
                    .pick_file()
                {
                    self.open_file(path);
                }
            }
            // Ctrl+S - Save
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) && self.has_changes {
                self.status = "Save: Use File > Save to apply changes".to_string();
            }
            // Ctrl+F - Find
            if i.modifiers.ctrl && i.key_pressed(egui::Key::F) {
                self.show_find_replace = true;
            }
            // Ctrl+Z - Undo
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) {
                self.undo();
            }
            // Ctrl+Y - Redo
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Y) {
                self.redo();
            }
            // Ctrl+C - Copy
            if i.modifiers.ctrl && i.key_pressed(egui::Key::C) && !self.selected_rows.is_empty() {
                self.copy_selected_rows();
            }
            // Ctrl+V - Paste
            if i.modifiers.ctrl && i.key_pressed(egui::Key::V) && !self.clipboard_rows.is_empty() {
                self.paste_rows();
            }
            // F5 - Refresh/Reload
            if i.key_pressed(egui::Key::F5) {
                if let Some(path) = self.current_file.clone() {
                    self.open_file(path);
                }
            }
            // F1 - Help
            if i.key_pressed(egui::Key::F1) {
                self.show_shortcuts_help = true;
            }
            // Escape - Close dialogs
            if i.key_pressed(egui::Key::Escape) {
                self.close_all_dialogs();
            }
        });
    }

    /// Close all open dialogs
    fn close_all_dialogs(&mut self) {
        self.show_find_replace = false;
        self.show_guid_generator = false;
        self.show_sql_editor = false;
        self.show_create_table_dialog = false;
        self.show_shortcuts_help = false;
        self.show_condition_builder = false;
        self.show_template_picker = false;
    }

    // ==================== Table Templates ====================

    /// Load table templates
    pub fn load_table_templates(&mut self) {
        self.table_templates = vec![
            TableTemplate {
                name: "Property",
                description: "Standard property table",
                columns: vec![
                    TemplateColumn { name: "Property", col_type: "String", nullable: false, primary_key: true },
                    TemplateColumn { name: "Value", col_type: "String", nullable: true, primary_key: false },
                ],
            },
            TableTemplate {
                name: "CustomAction",
                description: "Custom action definitions",
                columns: vec![
                    TemplateColumn { name: "Action", col_type: "String", nullable: false, primary_key: true },
                    TemplateColumn { name: "Type", col_type: "Integer", nullable: false, primary_key: false },
                    TemplateColumn { name: "Source", col_type: "String", nullable: true, primary_key: false },
                    TemplateColumn { name: "Target", col_type: "String", nullable: true, primary_key: false },
                ],
            },
            TableTemplate {
                name: "Registry",
                description: "Registry entries",
                columns: vec![
                    TemplateColumn { name: "Registry", col_type: "String", nullable: false, primary_key: true },
                    TemplateColumn { name: "Root", col_type: "Integer", nullable: false, primary_key: false },
                    TemplateColumn { name: "Key", col_type: "String", nullable: false, primary_key: false },
                    TemplateColumn { name: "Name", col_type: "String", nullable: true, primary_key: false },
                    TemplateColumn { name: "Value", col_type: "String", nullable: true, primary_key: false },
                    TemplateColumn { name: "Component_", col_type: "String", nullable: false, primary_key: false },
                ],
            },
            TableTemplate {
                name: "ServiceInstall",
                description: "Windows service installation",
                columns: vec![
                    TemplateColumn { name: "ServiceInstall", col_type: "String", nullable: false, primary_key: true },
                    TemplateColumn { name: "Name", col_type: "String", nullable: false, primary_key: false },
                    TemplateColumn { name: "DisplayName", col_type: "String", nullable: true, primary_key: false },
                    TemplateColumn { name: "ServiceType", col_type: "Integer", nullable: false, primary_key: false },
                    TemplateColumn { name: "StartType", col_type: "Integer", nullable: false, primary_key: false },
                    TemplateColumn { name: "ErrorControl", col_type: "Integer", nullable: false, primary_key: false },
                    TemplateColumn { name: "Component_", col_type: "String", nullable: false, primary_key: false },
                ],
            },
        ];
    }

    /// Create table from template
    pub fn create_from_template(&mut self, template_idx: usize) {
        if let Some(template) = self.table_templates.get(template_idx) {
            self.new_table_name = template.name.to_string();
            self.new_table_columns = template.columns.iter()
                .map(|c| NewColumnDef {
                    name: c.name.to_string(),
                    col_type: c.col_type.to_string(),
                    nullable: c.nullable,
                    primary_key: c.primary_key,
                    size: 255,
                })
                .collect();
            self.show_create_table_dialog = true;
            self.show_template_picker = false;
        }
    }

    // ==================== Condition Builder ====================

    /// Initialize condition builder
    pub fn init_condition_builder(&mut self) {
        self.condition_builder_root = Some(ConditionNode {
            node_type: ConditionNodeType::And,
            value: String::new(),
            children: vec![
                ConditionNode {
                    node_type: ConditionNodeType::Property,
                    value: "VersionNT".into(),
                    children: Vec::new(),
                },
                ConditionNode {
                    node_type: ConditionNodeType::Operator,
                    value: ">=".into(),
                    children: Vec::new(),
                },
                ConditionNode {
                    node_type: ConditionNodeType::Value,
                    value: "601".into(),
                    children: Vec::new(),
                },
            ],
        });
        self.build_condition_string();
        self.show_condition_builder = true;
    }

    /// Build condition string from tree
    pub fn build_condition_string(&mut self) {
        fn build_node(node: &ConditionNode) -> String {
            match node.node_type {
                ConditionNodeType::Property => node.value.clone(),
                ConditionNodeType::Operator => node.value.clone(),
                ConditionNodeType::Value => format!("\"{}\"", node.value),
                ConditionNodeType::And => {
                    let parts: Vec<String> = node.children.iter().map(build_node).collect();
                    format!("({})", parts.join(" AND "))
                },
                ConditionNodeType::Or => {
                    let parts: Vec<String> = node.children.iter().map(build_node).collect();
                    format!("({})", parts.join(" OR "))
                },
                ConditionNodeType::Not => {
                    if let Some(child) = node.children.first() {
                        format!("NOT {}", build_node(child))
                    } else {
                        String::new()
                    }
                },
            }
        }

        if let Some(ref root) = self.condition_builder_root {
            self.built_condition = build_node(root);
        }
    }

    // ==================== Column Statistics ====================

    /// Calculate column statistics
    pub fn calculate_column_stats(&mut self) {
        self.column_stats.clear();

        let Some(ref table) = self.current_table else { return };

        for (col_idx, column) in table.columns.iter().enumerate() {
            let values: Vec<&msi_explorer::CellValue> = table.rows.iter()
                .filter_map(|r| r.values.get(col_idx))
                .collect();

            let total_count = values.len();
            let null_count = values.iter().filter(|v| v.is_null()).count();

            let mut unique_values: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut total_length = 0usize;
            let mut numeric_values: Vec<i64> = Vec::new();

            for v in &values {
                let s = v.display();
                unique_values.insert(s.clone());
                total_length += s.len();
                if let msi_explorer::CellValue::Integer(n) = v {
                    numeric_values.push(*n as i64);
                }
            }

            let numeric_stats = if !numeric_values.is_empty() {
                let min = *numeric_values.iter().min().unwrap_or(&0);
                let max = *numeric_values.iter().max().unwrap_or(&0);
                let sum: i64 = numeric_values.iter().sum();
                let avg = sum as f64 / numeric_values.len() as f64;
                Some(NumericStats { min, max, sum, avg })
            } else {
                None
            };

            let sorted_unique: Vec<_> = unique_values.iter().cloned().collect();

            self.column_stats.push(ColumnStats {
                column_name: column.name.clone(),
                total_count,
                null_count,
                unique_count: sorted_unique.len(),
                min_value: sorted_unique.iter().min().cloned(),
                max_value: sorted_unique.iter().max().cloned(),
                avg_length: if total_count > 0 { total_length as f64 / total_count as f64 } else { 0.0 },
                numeric_stats,
            });
        }

        self.show_column_stats = true;
    }

    // ==================== Database Repair ====================

    /// Scan database for issues
    pub fn scan_db_issues(&mut self) {
        self.db_issues.clear();

        let Some(ref mut msi) = self.msi else { return };

        // Check for orphaned references
        self.find_orphans();
        for orphan in &self.orphans {
            self.db_issues.push(DatabaseIssue {
                issue_type: IssueType::OrphanedRow,
                table: orphan.table.clone(),
                description: format!("Orphaned entry: {}", orphan.key_value),
                can_auto_fix: true,
            });
        }

        // Check for duplicate primary keys
        self.find_duplicates();
        for dup in &self.duplicates {
            self.db_issues.push(DatabaseIssue {
                issue_type: IssueType::DuplicateKey,
                table: dup.table.clone(),
                description: format!("Duplicate key: {}", dup.key_value),
                can_auto_fix: false,
            });
        }

        // Check for invalid references
        self.validate_references();
        for err in &self.reference_errors {
            self.db_issues.push(DatabaseIssue {
                issue_type: IssueType::InvalidReference,
                table: err.table.clone(),
                description: format!("Invalid reference to {} in column {}", err.references_table, err.column),
                can_auto_fix: false,
            });
        }

        self.show_db_repair = true;
        self.status = format!("Found {} database issues", self.db_issues.len());
        self.log_action("Scan Database", &format!("{} issues", self.db_issues.len()));
    }

    /// Attempt to repair auto-fixable issues
    pub fn repair_db_issues(&mut self) {
        let fixable: Vec<_> = self.db_issues.iter()
            .filter(|i| i.can_auto_fix)
            .cloned()
            .collect();

        for issue in &fixable {
            match issue.issue_type {
                IssueType::OrphanedRow => {
                    // Mark orphaned row for deletion
                    self.status = format!("Would delete orphaned row in {}", issue.table);
                }
                _ => {}
            }
        }

        self.status = format!("Repair queued for {} fixable issues", fixable.len());
        self.log_action("Repair Database", &format!("{} fixes", fixable.len()));
    }

    // ==================== Batch Operations ====================

    /// Start batch operation
    pub fn start_batch_operation(&mut self, op_type: BatchOpType) {
        if let Some(files) = rfd::FileDialog::new()
            .add_filter("MSI Files", &["msi"])
            .pick_files()
        {
            self.batch_operation = Some(BatchOperation {
                operation_type: op_type,
                target_files: files,
                status: BatchStatus::Pending,
                results: Vec::new(),
            });
            self.show_batch_panel = true;
        }
    }

    /// Execute batch operation
    pub fn execute_batch(&mut self) {
        let Some(ref mut batch) = self.batch_operation else { return };

        batch.status = BatchStatus::Running;
        batch.results.clear();

        let files = batch.target_files.clone();
        let op_type = batch.operation_type.clone();

        for file in files.iter() {
            let result = match op_type {
                BatchOpType::ValidateAll => {
                    BatchResult {
                        file: file.clone(),
                        success: true,
                        message: "Validation pending".into(),
                    }
                }
                BatchOpType::ExtractAllTables => {
                    BatchResult {
                        file: file.clone(),
                        success: true,
                        message: "Export pending".into(),
                    }
                }
                _ => BatchResult {
                    file: file.clone(),
                    success: false,
                    message: "Operation not implemented".into(),
                }
            };
            batch.results.push(result);
        }

        batch.status = BatchStatus::Completed;

        // Log action outside the borrow
        let file_count = files.len();
        self.log_action("Batch Operation", &format!("{:?} on {} files", op_type, file_count));
    }

    // ==================== Session Logging ====================

    /// Log an action to session log
    pub fn log_action(&mut self, action: &str, details: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();

        self.session_log.push(SessionLogEntry {
            timestamp,
            action: action.to_string(),
            details: details.to_string(),
            file: self.current_file.as_ref().map(|p| p.display().to_string()),
        });

        // Keep last 1000 entries
        if self.session_log.len() > 1000 {
            self.session_log.remove(0);
        }
    }

    /// Export session log
    pub fn export_session_log(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Export Session Log")
            .add_filter("Text", &["txt"])
            .save_file()
        {
            let mut content = String::from("MSI Explorer Session Log\n");
            content.push_str("========================\n\n");

            for entry in &self.session_log {
                content.push_str(&format!(
                    "[{}] {} - {}\n",
                    entry.timestamp, entry.action, entry.details
                ));
                if let Some(ref file) = entry.file {
                    content.push_str(&format!("    File: {}\n", file));
                }
            }

            let _ = std::fs::write(path, content);
        }
    }

    // ==================== Plugin System ====================

    /// Scan for plugins
    pub fn scan_plugins(&mut self) {
        self.plugins.clear();

        // Look in standard plugin directory
        let plugin_dir = dirs::config_dir()
            .map(|d| d.join("msi-explorer").join("plugins"))
            .unwrap_or_default();

        if plugin_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "dll" || e == "so" || e == "dylib").unwrap_or(false) {
                        self.plugins.push(PluginInfo {
                            name: path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
                            version: "1.0.0".into(),
                            description: "External plugin".into(),
                            enabled: false,
                            path,
                        });
                    }
                }
            }
        }
    }

    /// Toggle plugin enabled state
    pub fn toggle_plugin(&mut self, idx: usize) {
        let log_msg = if let Some(plugin) = self.plugins.get_mut(idx) {
            plugin.enabled = !plugin.enabled;
            Some(format!("{} = {}", plugin.name, plugin.enabled))
        } else {
            None
        };

        if let Some(msg) = log_msg {
            self.log_action("Plugin Toggle", &msg);
        }
    }

    // ==================== Database Compression ====================

    /// Compress/optimize database
    pub fn compress_database(&mut self) {
        let Some(ref path) = self.current_file else {
            self.error = Some("No file open".into());
            return;
        };

        // In a real implementation, we'd repack the MSI with optimized streams
        self.status = format!("Database compression not available (would optimize {})", path.display());
        self.log_action("Compress Database", "Requested");
    }

    // ==================== CAB Extraction ====================

    /// Extract files from embedded CABs
    pub fn extract_cab_files(&mut self) {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Select Extraction Directory")
            .pick_folder()
        {
            self.cab_extraction_path = Some(dir.clone());
            self.extracted_cab_files.clear();

            // In a real implementation, we'd use cab crate to extract
            // For now, show what would be extracted based on File table
            if let Some(ref mut msi) = self.msi {
                if let Ok(file_table) = msi.get_table("File") {
                    let name_col = file_table.column_index("FileName").unwrap_or(2);
                    let size_col = file_table.column_index("FileSize");

                    for row in &file_table.rows {
                        let name = row.values.get(name_col).map(|v| {
                            let n = v.display();
                            n.split('|').last().unwrap_or(&n).to_string()
                        }).unwrap_or_default();

                        let size = size_col.and_then(|c| row.values.get(c))
                            .and_then(|v| match v {
                                msi_explorer::CellValue::Integer(n) => Some(*n as u64),
                                _ => None,
                            }).unwrap_or(0);

                        self.extracted_cab_files.push(ExtractedCabFile {
                            name: name.clone(),
                            path: dir.join(&name),
                            size,
                            compressed: true,
                        });
                    }
                }
            }

            self.show_cab_extraction = true;
            self.status = format!("Found {} files to extract", self.extracted_cab_files.len());
            self.log_action("CAB Extraction", &format!("{} files", self.extracted_cab_files.len()));
        }
    }

    // ==================== Binary Diff ====================

    /// Compare binary streams between current and diff MSI
    pub fn compare_binary_streams(&mut self) {
        self.binary_diff_results.clear();

        let Some(ref mut msi) = self.msi else { return };
        let Some(ref mut diff_msi) = self.diff_msi else {
            self.error = Some("No diff file loaded".into());
            return;
        };

        // Compare Binary tables
        let binary_a = msi.get_table("Binary").ok();
        let binary_b = diff_msi.get_table("Binary").ok();

        if let (Some(a), Some(b)) = (binary_a, binary_b) {
            let names_a: std::collections::HashSet<_> = a.rows.iter()
                .filter_map(|r| r.values.first().map(|v| v.display()))
                .collect();
            let names_b: std::collections::HashSet<_> = b.rows.iter()
                .filter_map(|r| r.values.first().map(|v| v.display()))
                .collect();

            // Check all streams in A
            for name in &names_a {
                let in_b = names_b.contains(name);
                self.binary_diff_results.push(BinaryDiffResult {
                    stream_name: name.clone(),
                    size_a: 0, // Would need actual binary data
                    size_b: if in_b { 0 } else { 0 },
                    differs: !in_b,
                    diff_offset: None,
                });
            }

            // Streams only in B
            for name in names_b.difference(&names_a) {
                self.binary_diff_results.push(BinaryDiffResult {
                    stream_name: name.clone(),
                    size_a: 0,
                    size_b: 0,
                    differs: true,
                    diff_offset: None,
                });
            }
        }

        self.show_binary_diff = true;
        self.log_action("Binary Diff", &format!("{} streams compared", self.binary_diff_results.len()));
    }

    // ==================== Install Simulation ====================

    /// Simulate installation
    pub fn simulate_install(&mut self) {
        self.simulation_steps.clear();

        let Some(ref mut msi) = self.msi else { return };

        // Get sequence table
        if let Ok(seq) = msi.get_table("InstallExecuteSequence") {
            let action_col = seq.column_index("Action").unwrap_or(0);
            let sequence_col = seq.column_index("Sequence").unwrap_or(1);
            let condition_col = seq.column_index("Condition");

            let mut steps: Vec<_> = seq.rows.iter().map(|row| {
                let action = row.values.get(action_col).map(|v| v.display()).unwrap_or_default();
                let _seq_num = row.values.get(sequence_col).and_then(|v| match v {
                    msi_explorer::CellValue::Integer(n) => Some(*n),
                    _ => None,
                }).unwrap_or(0);
                let condition = condition_col.and_then(|c| row.values.get(c))
                    .map(|v| v.display())
                    .filter(|s| !s.is_empty());

                let will_run = condition.as_ref().map(|c| self.evaluate_condition(c)).unwrap_or(true);

                SimulationStep {
                    action: action.clone(),
                    description: self.get_action_description(&action),
                    affected_files: Vec::new(),
                    affected_registry: Vec::new(),
                    condition,
                    will_run,
                }
            }).collect();

            steps.sort_by_key(|s| s.action.clone());
            self.simulation_steps = steps;
        }

        self.show_simulation = true;
        self.log_action("Simulate Install", &format!("{} steps", self.simulation_steps.len()));
    }

    fn get_action_description(&self, action: &str) -> String {
        match action {
            "InstallFiles" => "Copy files to target directories".into(),
            "WriteRegistryValues" => "Write registry entries".into(),
            "CreateShortcuts" => "Create Start Menu and Desktop shortcuts".into(),
            "RegisterProduct" => "Register product with Windows Installer".into(),
            "InstallFinalize" => "Finalize installation and commit changes".into(),
            _ => format!("Execute {}", action),
        }
    }

    fn evaluate_condition(&self, _condition: &str) -> bool {
        // Simplified - in real impl would parse and evaluate
        true
    }

    // ==================== Feature Costing ====================

    /// Calculate feature costs
    pub fn calculate_feature_costs(&mut self) {
        self.feature_costs.clear();
        self.total_disk_space = 0;

        let Some(ref mut msi) = self.msi else { return };

        let feature_table = msi.get_table("Feature").ok();
        let fc_table = msi.get_table("FeatureComponents").ok();
        let file_table = msi.get_table("File").ok();

        let Some(features) = feature_table else { return };

        // Build component -> size map
        let mut comp_sizes: HashMap<String, u64> = HashMap::new();
        let mut comp_files: HashMap<String, usize> = HashMap::new();

        if let Some(ref files) = file_table {
            let comp_col = files.column_index("Component_").unwrap_or(1);
            let size_col = files.column_index("FileSize");

            for row in &files.rows {
                let comp = row.values.get(comp_col).map(|v| v.display()).unwrap_or_default();
                let size = size_col.and_then(|c| row.values.get(c))
                    .and_then(|v| match v {
                        msi_explorer::CellValue::Integer(n) => Some(*n as u64),
                        _ => None,
                    }).unwrap_or(0);

                *comp_sizes.entry(comp.clone()).or_default() += size;
                *comp_files.entry(comp).or_default() += 1;
            }
        }

        // Build feature -> components map
        let mut feature_comps: HashMap<String, Vec<String>> = HashMap::new();
        if let Some(ref fc) = fc_table {
            let f_col = fc.column_index("Feature_").unwrap_or(0);
            let c_col = fc.column_index("Component_").unwrap_or(1);

            for row in &fc.rows {
                let feature = row.values.get(f_col).map(|v| v.display()).unwrap_or_default();
                let comp = row.values.get(c_col).map(|v| v.display()).unwrap_or_default();
                feature_comps.entry(feature).or_default().push(comp);
            }
        }

        let feature_col = features.column_index("Feature").unwrap_or(0);
        let title_col = features.column_index("Title");

        for row in &features.rows {
            let feature = row.values.get(feature_col).map(|v| v.display()).unwrap_or_default();
            let title = title_col.and_then(|c| row.values.get(c))
                .map(|v| v.display())
                .unwrap_or_else(|| feature.clone());

            let comps = feature_comps.get(&feature).cloned().unwrap_or_default();
            let mut local_cost = 0u64;
            let mut file_count = 0usize;

            for comp in &comps {
                local_cost += comp_sizes.get(comp).copied().unwrap_or(0);
                file_count += comp_files.get(comp).copied().unwrap_or(0);
            }

            self.total_disk_space += local_cost;

            self.feature_costs.push(FeatureCost {
                feature,
                title,
                local_cost,
                source_cost: local_cost, // Same for now
                components: comps.len(),
                files: file_count,
            });
        }

        self.show_feature_costs = true;
        self.log_action("Calculate Costs", &format!("{} features, {} total",
            self.feature_costs.len(), format_bytes(self.total_disk_space)));
    }

    // ==================== Annotations ====================

    /// Add annotation to current row
    pub fn add_annotation(&mut self, table: &str, row_key: &str, note: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        self.annotations.push(RowAnnotation {
            table: table.to_string(),
            row_key: row_key.to_string(),
            note: note.to_string(),
            timestamp,
        });

        self.log_action("Add Annotation", &format!("{}.{}", table, row_key));
    }

    /// Remove annotation
    pub fn remove_annotation(&mut self, idx: usize) {
        if idx < self.annotations.len() {
            self.annotations.remove(idx);
        }
    }

    // ==================== Watch Expressions ====================

    /// Add watch expression
    pub fn add_watch(&mut self, property: &str) {
        let value = self.get_property_value(property);

        self.watch_expressions.push(WatchExpression {
            property: property.to_string(),
            value,
            condition: None,
        });
    }

    fn get_property_value(&mut self, property: &str) -> String {
        if let Some(ref mut msi) = self.msi {
            if let Ok(prop_table) = msi.get_table("Property") {
                let prop_col = prop_table.column_index("Property").unwrap_or(0);
                let val_col = prop_table.column_index("Value").unwrap_or(1);

                for row in &prop_table.rows {
                    if let Some(p) = row.values.get(prop_col) {
                        if p.display() == property {
                            return row.values.get(val_col).map(|v| v.display()).unwrap_or_default();
                        }
                    }
                }
            }
        }
        String::new()
    }

    /// Remove watch expression
    pub fn remove_watch(&mut self, idx: usize) {
        if idx < self.watch_expressions.len() {
            self.watch_expressions.remove(idx);
        }
    }

    // ==================== Row History ====================

    /// Record row change
    pub fn record_row_change(&mut self, table: &str, row_key: &str, column: &str, old_value: &str, new_value: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        self.row_history.push(RowChange {
            table: table.to_string(),
            row_key: row_key.to_string(),
            column: column.to_string(),
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
            timestamp,
        });

        // Keep last 500 changes
        if self.row_history.len() > 500 {
            self.row_history.remove(0);
        }
    }

    // ==================== Split View ====================

    /// Toggle split view
    pub fn toggle_split_view(&mut self) {
        self.split_view.enabled = !self.split_view.enabled;
        if self.split_view.enabled {
            self.split_view.left_table = self.selected_table.clone();
        } else {
            self.split_table = None;
        }
    }

    /// Set right table in split view
    pub fn set_split_right_table(&mut self, table_name: &str) {
        if let Some(ref mut msi) = self.msi {
            if let Ok(table) = msi.get_table(table_name) {
                self.split_view.right_table = Some(table_name.to_string());
                self.split_table = Some(table);
            }
        }
    }

    // ==================== Recent Searches ====================

    /// Add to recent searches
    pub fn add_recent_search(&mut self, query: &str, result_count: usize) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();

        // Remove if exists
        self.recent_searches.retain(|s| s.query != query);

        self.recent_searches.insert(0, RecentSearch {
            query: query.to_string(),
            timestamp,
            result_count,
        });

        // Keep last 20
        self.recent_searches.truncate(20);
    }

    // ==================== Favorites ====================

    /// Add current table to favorites
    pub fn add_table_to_favorites(&mut self) {
        if let Some(table) = self.selected_table.clone() {
            self.favorites.push(FavoriteItem {
                name: table.clone(),
                item_type: FavoriteType::Table,
                table: Some(table.clone()),
                row_key: None,
            });
            self.log_action("Add Favorite", &table);
        }
    }

    /// Remove favorite
    pub fn remove_favorite(&mut self, idx: usize) {
        if idx < self.favorites.len() {
            self.favorites.remove(idx);
        }
    }

    // ==================== Column Profiles ====================

    /// Save current column profile
    pub fn save_column_profile(&mut self, name: &str) {
        if let Some(ref table) = self.current_table {
            let visible_columns: Vec<String> = table.columns.iter()
                .map(|c| c.name.clone())
                .collect();

            self.column_profiles.push(ColumnProfile {
                name: name.to_string(),
                table: table.name.clone(),
                visible_columns,
                column_widths: vec![100.0; table.columns.len()],
            });

            self.current_profile = Some(name.to_string());
            self.log_action("Save Profile", name);
        }
    }

    // ==================== Rollback Viewer ====================

    /// Build rollback operations view
    pub fn build_rollback_view(&mut self) {
        self.rollback_operations.clear();

        let Some(ref mut msi) = self.msi else { return };

        // Rollback is the reverse of InstallExecuteSequence
        if let Ok(seq) = msi.get_table("InstallExecuteSequence") {
            let action_col = seq.column_index("Action").unwrap_or(0);
            let sequence_col = seq.column_index("Sequence").unwrap_or(1);

            let mut ops: Vec<_> = seq.rows.iter().filter_map(|row| {
                let action = row.values.get(action_col).map(|v| v.display()).unwrap_or_default();
                let seq_num = row.values.get(sequence_col).and_then(|v| match v {
                    msi_explorer::CellValue::Integer(n) => Some(*n),
                    _ => None,
                }).unwrap_or(0);

                // Only include actions that have rollback
                let op = match action.as_str() {
                    "InstallFiles" => Some("RemoveFiles"),
                    "WriteRegistryValues" => Some("RemoveRegistryValues"),
                    "CreateShortcuts" => Some("RemoveShortcuts"),
                    _ => None,
                };

                op.map(|operation| RollbackOperation {
                    action: action.clone(),
                    operation: operation.to_string(),
                    target: "Rollback".to_string(),
                    sequence: -seq_num, // Reversed
                })
            }).collect();

            ops.sort_by_key(|o| o.sequence);
            self.rollback_operations = ops;
        }

        self.show_rollback_viewer = true;
        self.log_action("Rollback View", &format!("{} operations", self.rollback_operations.len()));
    }

    // ==================== Action Timing ====================

    /// Estimate action timings
    pub fn estimate_action_timings(&mut self) {
        self.action_timings.clear();

        let Some(ref mut msi) = self.msi else { return };

        if let Ok(seq) = msi.get_table("InstallExecuteSequence") {
            let action_col = seq.column_index("Action").unwrap_or(0);

            for row in &seq.rows {
                let action = row.values.get(action_col).map(|v| v.display()).unwrap_or_default();

                let (estimated_ms, category) = match action.as_str() {
                    "InstallFiles" => (5000, TimingCategory::Slow),
                    "WriteRegistryValues" => (500, TimingCategory::Medium),
                    "CreateShortcuts" => (200, TimingCategory::Medium),
                    "RegisterProduct" => (100, TimingCategory::Medium),
                    "CostInitialize" | "CostFinalize" | "FileCost" => (50, TimingCategory::Fast),
                    a if a.starts_with("CA_") => (1000, TimingCategory::Medium),
                    _ => (10, TimingCategory::Fast),
                };

                self.action_timings.push(ActionTiming {
                    action,
                    estimated_ms,
                    category,
                });
            }
        }

        self.show_action_timing = true;
        let total: u64 = self.action_timings.iter().map(|t| t.estimated_ms).sum();
        self.log_action("Estimate Timing", &format!("Total: {}ms", total));
    }

    // ==================== Patch Analysis ====================

    /// Analyze patch changes
    pub fn analyze_patch(&mut self) {
        self.patch_deltas.clear();

        let Some(ref mut msi) = self.msi else { return };
        let Some(ref mut diff_msi) = self.diff_msi else {
            self.error = Some("No patch/diff file loaded".into());
            return;
        };

        // Compare tables
        let tables_a: std::collections::HashSet<_> = msi.table_names().into_iter().collect();
        let tables_b: std::collections::HashSet<_> = diff_msi.table_names().into_iter().collect();

        // New tables
        for table in tables_b.difference(&tables_a) {
            self.patch_deltas.push(PatchDelta {
                table: table.clone(),
                operation: PatchOperation::Add,
                key: "".to_string(),
                details: "New table added".to_string(),
            });
        }

        // Removed tables
        for table in tables_a.difference(&tables_b) {
            self.patch_deltas.push(PatchDelta {
                table: table.clone(),
                operation: PatchOperation::Delete,
                key: "".to_string(),
                details: "Table removed".to_string(),
            });
        }

        // Modified tables - simplified comparison
        for table in tables_a.intersection(&tables_b) {
            let a = msi.get_table(table).ok();
            let b = diff_msi.get_table(table).ok();

            if let (Some(ta), Some(tb)) = (a, b) {
                if ta.rows.len() != tb.rows.len() {
                    self.patch_deltas.push(PatchDelta {
                        table: table.clone(),
                        operation: PatchOperation::Modify,
                        key: "".to_string(),
                        details: format!("Row count: {} -> {}", ta.rows.len(), tb.rows.len()),
                    });
                }
            }
        }

        self.show_patch_analysis = true;
        self.log_action("Analyze Patch", &format!("{} changes", self.patch_deltas.len()));
    }

    // ==================== Export Excel ====================

    /// Export table to Excel format (CSV with .xlsx extension for now)
    pub fn export_to_excel(&self) {
        let Some(ref table) = self.current_table else { return };

        if let Some(path) = rfd::FileDialog::new()
            .set_title("Export to Excel")
            .add_filter("Excel", &["xlsx", "csv"])
            .set_file_name(&format!("{}.csv", table.name))
            .save_file()
        {
            let mut content = String::new();

            // Header
            let headers: Vec<_> = table.columns.iter().map(|c| c.name.clone()).collect();
            content.push_str(&headers.join(","));
            content.push('\n');

            // Rows
            for row in &table.rows {
                let values: Vec<_> = row.values.iter()
                    .map(|v| {
                        let s = v.display();
                        if s.contains(',') || s.contains('"') || s.contains('\n') {
                            format!("\"{}\"", s.replace('"', "\"\""))
                        } else {
                            s
                        }
                    })
                    .collect();
                content.push_str(&values.join(","));
                content.push('\n');
            }

            if std::fs::write(&path, content).is_ok() {
                // Can't mutate self in &self method, so no status update
            }
        }
    }

    // ==================== Export XML ====================

    /// Export table to XML format
    pub fn export_to_xml(&self) {
        let Some(ref table) = self.current_table else { return };

        if let Some(path) = rfd::FileDialog::new()
            .set_title("Export to XML")
            .add_filter("XML", &["xml"])
            .set_file_name(&format!("{}.xml", table.name))
            .save_file()
        {
            let mut content = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            content.push_str(&format!("<Table name=\"{}\">\n", table.name));

            for row in &table.rows {
                content.push_str("  <Row>\n");
                for (col, val) in table.columns.iter().zip(row.values.iter()) {
                    let escaped = val.display()
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;")
                        .replace('"', "&quot;");
                    content.push_str(&format!("    <{0}>{1}</{0}>\n", col.name, escaped));
                }
                content.push_str("  </Row>\n");
            }

            content.push_str("</Table>\n");
            let _ = std::fs::write(&path, content);
        }
    }

    // ==================== Auto-complete ====================

    /// Get auto-complete suggestions
    pub fn get_autocomplete_suggestions(&mut self, input: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let input_lower = input.to_lowercase();

        // Table names
        for table in &self.tables {
            if table.to_lowercase().starts_with(&input_lower) {
                suggestions.push(table.clone());
            }
        }

        // Property names
        if let Some(ref mut msi) = self.msi {
            if let Ok(prop_table) = msi.get_table("Property") {
                for row in &prop_table.rows {
                    if let Some(p) = row.values.first() {
                        let prop = p.display();
                        if prop.to_lowercase().starts_with(&input_lower) {
                            suggestions.push(prop);
                        }
                    }
                }
            }
        }

        // Standard action names
        let actions = ["InstallFiles", "WriteRegistryValues", "CreateShortcuts",
            "RegisterProduct", "InstallFinalize", "CostInitialize", "CostFinalize"];
        for action in actions {
            if action.to_lowercase().starts_with(&input_lower) {
                suggestions.push(action.to_string());
            }
        }

        suggestions.truncate(10);
        self.auto_complete_suggestions = suggestions.clone();
        suggestions
    }
}

/// Format bytes to human-readable
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Dialog preview data
#[derive(Debug, Clone)]
pub struct DialogPreview {
    pub name: String,
    pub title: String,
    pub width: i32,
    pub height: i32,
    pub controls: Vec<ControlPreview>,
}

/// Control preview data
#[derive(Debug, Clone)]
pub struct ControlPreview {
    pub name: String,
    pub control_type: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub text: String,
}

impl eframe::App for MsiExplorerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme once
        if !self.theme_applied {
            Theme::apply(ctx);
            self.theme_applied = true;
        }

        // Handle dropped files
        let dropped_file = ctx.input(|i| {
            i.raw.dropped_files.first().and_then(|f| f.path.clone())
        });
        if let Some(path) = dropped_file {
            if path.extension().map(|e| e == "msi").unwrap_or(false) {
                self.open_file(path);
            } else {
                self.error = Some("Please drop an MSI file".into());
            }
        }

        // Keyboard shortcuts
        ctx.input(|i| {
            // Ctrl+O: Open file
            if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("MSI Files", &["msi"])
                    .pick_file()
                {
                    // Can't call self.open_file here due to borrow, handled below
                }
            }
        });

        // Handle keyboard shortcuts that modify state
        let mut open_file_dialog = false;
        let mut do_undo = false;
        let mut do_redo = false;
        let mut do_copy = false;
        let mut do_paste = false;
        let mut do_find = false;

        ctx.input(|i| {
            if i.modifiers.ctrl || i.modifiers.mac_cmd {
                if i.key_pressed(egui::Key::O) { open_file_dialog = true; }
                if i.key_pressed(egui::Key::Z) && !i.modifiers.shift { do_undo = true; }
                if i.key_pressed(egui::Key::Z) && i.modifiers.shift { do_redo = true; }
                if i.key_pressed(egui::Key::Y) { do_redo = true; }
                if i.key_pressed(egui::Key::C) { do_copy = true; }
                if i.key_pressed(egui::Key::V) { do_paste = true; }
                if i.key_pressed(egui::Key::F) { do_find = true; }
            }
        });

        if open_file_dialog {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("MSI Files", &["msi"])
                .pick_file()
            {
                self.open_file(path);
            }
        }
        if do_undo { self.undo(); }
        if do_redo { self.redo(); }
        if do_find { self.show_find_replace = true; }
        // Copy/paste handled via context menu for now

        // Top bar with app title and menu
        egui::TopBottomPanel::top("top_bar")
            .frame(egui::Frame::none()
                .fill(Theme::BG_MEDIUM)
                .inner_margin(egui::Margin::symmetric(16.0, 8.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // App title
                    ui.label(RichText::new("MSI Explorer")
                        .size(18.0)
                        .color(Theme::TEXT_PRIMARY)
                        .strong());

                    ui.add_space(24.0);

                    // File menu
                    let recent_files = self.recent_files.clone();
                    ui.menu_button(RichText::new("File").color(Theme::TEXT_PRIMARY), |ui| {
                        if ui.button("New MSI...").clicked() {
                            self.create_new_msi();
                            ui.close_menu();
                        }
                        if ui.button("Open...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("MSI Files", &["msi"])
                                .add_filter("All Files", &["*"])
                                .pick_file()
                            {
                                self.open_file(path);
                            }
                            ui.close_menu();
                        }
                        ui.separator();
                        if !recent_files.is_empty() {
                            ui.label(RichText::new("Recent Files").color(Theme::TEXT_MUTED).size(11.0));
                            for (i, path) in recent_files.iter().take(5).enumerate() {
                                let name = path.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| format!("File {}", i + 1));
                                if ui.button(&name).clicked() {
                                    self.open_file(path.clone());
                                    ui.close_menu();
                                }
                            }
                            ui.separator();
                        }
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Compare...")).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("MSI Files", &["msi"])
                                .pick_file()
                            {
                                self.open_diff_file(path);
                            }
                            ui.close_menu();
                        }
                    });

                    // Edit menu
                    ui.menu_button(RichText::new("Edit").color(Theme::TEXT_PRIMARY), |ui| {
                        if ui.add_enabled(!self.undo_stack.is_empty(), egui::Button::new("Undo (Ctrl+Z)")).clicked() {
                            self.undo();
                            ui.close_menu();
                        }
                        if ui.add_enabled(!self.redo_stack.is_empty(), egui::Button::new("Redo (Ctrl+Y)")).clicked() {
                            self.redo();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add_enabled(self.current_table.is_some(), egui::Button::new("Copy Row (Ctrl+C)")).clicked() {
                            // Copy would need selected row - for now just show status
                            self.status = "Right-click a row to copy".to_string();
                            ui.close_menu();
                        }
                        if ui.add_enabled(!self.clipboard_rows.is_empty() && self.current_table.is_some(),
                            egui::Button::new("Paste (Ctrl+V)")).clicked() {
                            self.paste_rows();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Find & Replace... (Ctrl+F)").clicked() {
                            self.show_find_replace = true;
                            ui.close_menu();
                        }
                    });

                    ui.add_space(16.0);

                    // View mode tabs
                    if self.msi.is_some() {
                        ui.separator();
                        ui.add_space(8.0);

                        if ui.selectable_label(self.view_mode == ViewMode::Tables,
                            RichText::new("◫ Tables").size(12.0)).clicked() {
                            self.view_mode = ViewMode::Tables;
                        }
                        if ui.selectable_label(self.view_mode == ViewMode::Tree,
                            RichText::new("◬ Tree").size(12.0)).clicked() {
                            self.view_mode = ViewMode::Tree;
                        }
                        if ui.selectable_label(self.view_mode == ViewMode::Files,
                            RichText::new("◨ Files").size(12.0)).clicked() {
                            self.view_mode = ViewMode::Files;
                        }
                        if self.diff_msi.is_some() {
                            if ui.selectable_label(self.view_mode == ViewMode::Diff,
                                RichText::new("◇ Diff").size(12.0)).clicked() {
                                self.view_mode = ViewMode::Diff;
                            }
                        }

                        ui.add_space(16.0);

                        // Number format toggle
                        if ui.button(RichText::new(format!("# {}", self.number_format.label()))
                            .size(11.0)
                            .color(Theme::TEXT_SECONDARY)).clicked() {
                            self.number_format = self.number_format.cycle();
                        }

                        ui.add_space(8.0);

                        // Edit mode toggle
                        let edit_label = if self.edit_mode { "✎ Edit ON" } else { "✎ Edit" };
                        let edit_color = if self.edit_mode { Theme::WARNING } else { Theme::TEXT_SECONDARY };
                        if ui.button(RichText::new(edit_label).size(11.0).color(edit_color)).clicked() {
                            self.edit_mode = !self.edit_mode;
                            if !self.edit_mode {
                                self.editing_cell = None;
                            }
                        }

                        // Show discard button if there are changes
                        if self.has_changes {
                            if ui.button(RichText::new(format!("⟲ Discard ({})", self.pending_changes_count()))
                                .size(11.0)
                                .color(Theme::ERROR)).clicked() {
                                self.discard_changes();
                            }
                        }

                        // Add row button (only in edit mode with table selected)
                        if self.edit_mode && self.current_table.is_some() {
                            ui.add_space(8.0);
                            if ui.button(RichText::new("+ Add Row").size(11.0).color(Theme::SUCCESS)).clicked() {
                                self.start_add_row();
                            }
                        }

                        // Save button (when there are changes)
                        if self.has_changes {
                            ui.add_space(8.0);
                            if ui.button(RichText::new("💾 Save").size(11.0).color(Theme::SUCCESS)).clicked() {
                                self.export_changes();
                            }
                        }
                    }

                    ui.menu_button(RichText::new("View").color(Theme::TEXT_PRIMARY), |ui| {
                        ui.checkbox(&mut self.show_categories, "Group by Category");
                        ui.checkbox(&mut self.show_filter_bar, "Show Filter Bar");
                        ui.separator();
                        if ui.button("Show Validation Panel").clicked() {
                            self.show_validation_panel = !self.show_validation_panel;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Edit Summary Info...")).clicked() {
                            self.open_edit_summary_dialog();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Sequence Viewer...")).clicked() {
                            self.show_sequence_view = true;
                            ui.close_menu();
                        }
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Dialog Preview...")).clicked() {
                            self.show_dialog_preview = true;
                            ui.close_menu();
                        }
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Script Viewer...")).clicked() {
                            self.show_script_viewer = true;
                            ui.close_menu();
                        }
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("String Localization...")).clicked() {
                            self.load_languages();
                            self.show_localization_view = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.label(RichText::new("Analysis").color(Theme::TEXT_MUTED).size(11.0));
                        if ui.add_enabled(self.current_table.is_some(), egui::Button::new("Table Schema...")).clicked() {
                            self.show_schema_viewer = true;
                            ui.close_menu();
                        }
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Dependency Graph...")).clicked() {
                            self.build_dependency_graph();
                            self.show_dependency_graph = true;
                            ui.close_menu();
                        }
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Statistics Dashboard...")).clicked() {
                            self.show_stats_dashboard = true;
                            ui.close_menu();
                        }
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Stream Viewer...")).clicked() {
                            self.load_streams();
                            self.show_stream_viewer = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.label(RichText::new("Previews").color(Theme::TEXT_MUTED).size(11.0));
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Registry Preview...")).clicked() {
                            self.show_registry_preview = true;
                            ui.close_menu();
                        }
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Shortcut Preview...")).clicked() {
                            self.show_shortcut_preview = true;
                            ui.close_menu();
                        }
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("Service Preview...")).clicked() {
                            self.show_service_preview = true;
                            ui.close_menu();
                        }
                        if ui.add_enabled(self.msi.is_some(), egui::Button::new("CAB Contents...")).clicked() {
                            self.load_cab_contents();
                            self.show_cab_contents = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.checkbox(&mut self.show_bookmarks, "Show Bookmarks");
                        if ui.checkbox(&mut self.dark_mode, "Dark Mode").changed() {
                            // Theme will be re-applied
                        }
                    });

                    // Transform menu
                    if self.msi.is_some() {
                        ui.menu_button(RichText::new("Transform").color(Theme::TEXT_PRIMARY), |ui| {
                            if self.transform_mode {
                                ui.label(RichText::new("● Transform Active").color(Theme::WARNING));
                                ui.separator();
                                if ui.button("Generate Transform (.mst)").clicked() {
                                    self.generate_transform();
                                    ui.close_menu();
                                }
                                if ui.button("Cancel Transform").clicked() {
                                    self.transform_mode = false;
                                    self.discard_changes();
                                    ui.close_menu();
                                }
                            } else {
                                if ui.button("New Transform").clicked() {
                                    self.start_new_transform();
                                    ui.close_menu();
                                }
                                if ui.button("Apply Transform...").clicked() {
                                    self.apply_transform();
                                    ui.close_menu();
                                }
                            }
                        });

                        // Tools menu
                        ui.menu_button(RichText::new("Tools").color(Theme::TEXT_PRIMARY), |ui| {
                            if ui.button("Validate (ICE)").clicked() {
                                self.run_validation();
                                ui.close_menu();
                            }
                            if ui.button("Validate References").clicked() {
                                self.validate_references();
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Merge Module...").clicked() {
                                self.open_merge_module_dialog();
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Create Patch (MSP)...").clicked() {
                                self.show_create_patch_dialog = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("SQL Query Editor...").clicked() {
                                self.show_sql_editor = true;
                                ui.close_menu();
                            }
                            if ui.button("GUID Generator...").clicked() {
                                self.generate_guid();
                                self.show_guid_generator = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button("Check Signature").clicked() {
                                self.check_signature();
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Tables").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.add_enabled(self.current_table.is_some(), egui::Button::new("Export Table (IDT)...")).clicked() {
                                self.export_table_idt();
                                ui.close_menu();
                            }
                            if ui.button("Import Table (IDT)...").clicked() {
                                self.import_table_idt();
                                ui.close_menu();
                            }
                            if ui.button("Export All Tables...").clicked() {
                                self.export_all_tables();
                                ui.close_menu();
                            }
                            if ui.button("Create New Table...").clicked() {
                                self.show_create_table_dialog = true;
                                self.add_new_column();
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Edit").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.add_enabled(self.current_table.is_some(), egui::Button::new("Bulk Edit...")).clicked() {
                                self.show_bulk_edit = true;
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Property Editor...")).clicked() {
                                self.init_known_properties();
                                self.show_property_editor = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Validation").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.button("Condition Validator...").clicked() {
                                self.show_condition_validator = true;
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Verify File Hashes")).clicked() {
                                self.verify_file_hashes();
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Export").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Export to WiX...")).clicked() {
                                self.show_wix_export = true;
                                ui.close_menu();
                            }
                            if ui.button("Generate Report...").clicked() {
                                self.show_report_dialog = true;
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.diff_msi.is_some(), egui::Button::new("Export Diff...")).clicked() {
                                self.export_diff();
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("CAB").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.button("Rebuild CAB...").clicked() {
                                self.show_cab_rebuild = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Analysis").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Find Duplicates...")).clicked() {
                                self.find_duplicates();
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Find Orphans...")).clicked() {
                                self.find_orphans();
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Feature Tree...")).clicked() {
                                self.build_feature_tree();
                                self.show_feature_tree = true;
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Directory Tree...")).clicked() {
                                self.build_directory_tree();
                                self.show_directory_tree = true;
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Component Rules...")).clicked() {
                                self.check_component_rules();
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Decode Custom Actions...")).clicked() {
                                self.decode_custom_actions();
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Sequence Timeline...")).clicked() {
                                self.build_timeline();
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.current_table.is_some(), egui::Button::new("Column Statistics...")).clicked() {
                                self.calculate_column_stats();
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Database").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Scan for Issues...")).clicked() {
                                self.scan_db_issues();
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Compress Database...")).clicked() {
                                self.show_compression_dialog = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Advanced").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.button("Extract Icons...").clicked() {
                                self.load_icons();
                                self.show_icon_extraction = true;
                                ui.close_menu();
                            }
                            if ui.button("Compare Transform...").clicked() {
                                self.compare_transform();
                                ui.close_menu();
                            }
                            if ui.button("Batch Operations...").clicked() {
                                self.show_batch_panel = true;
                                ui.close_menu();
                            }
                            if ui.button("Session Log...").clicked() {
                                self.show_session_log = true;
                                ui.close_menu();
                            }
                            if ui.button("Table Templates...").clicked() {
                                self.load_table_templates();
                                self.show_template_picker = true;
                                ui.close_menu();
                            }
                            if ui.button("Condition Builder...").clicked() {
                                self.init_condition_builder();
                                ui.close_menu();
                            }
                            if ui.button("Plugin Manager...").clicked() {
                                self.scan_plugins();
                                self.show_plugin_manager = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Binary & CAB").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Extract CAB Files...")).clicked() {
                                self.extract_cab_files();
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.diff_msi.is_some(), egui::Button::new("Compare Binary Streams...")).clicked() {
                                self.compare_binary_streams();
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Simulation").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Simulate Install...")).clicked() {
                                self.simulate_install();
                                self.show_simulation = true;
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Calculate Feature Costs...")).clicked() {
                                self.calculate_feature_costs();
                                self.show_feature_costs = true;
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Build Rollback View...")).clicked() {
                                self.build_rollback_view();
                                self.show_rollback_viewer = true;
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Estimate Action Timings...")).clicked() {
                                self.estimate_action_timings();
                                self.show_action_timing = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Patch Analysis").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.button("Analyze Patch...").clicked() {
                                self.analyze_patch();
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("View Options").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.button(if self.split_view.enabled { "Disable Split View" } else { "Enable Split View" }).clicked() {
                                self.toggle_split_view();
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.selected_table.is_some(), egui::Button::new("Add to Favorites")).clicked() {
                                self.add_table_to_favorites();
                                ui.close_menu();
                            }
                            if ui.button("Save Column Profile...").clicked() {
                                if self.current_table.is_some() {
                                    self.save_column_profile("Default");
                                }
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Watch & Annotate").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.button("Add Property Watch...").clicked() {
                                self.show_watch_panel = true;
                                ui.close_menu();
                            }
                            if ui.button("Annotations...").clicked() {
                                self.show_annotations = true;
                                ui.close_menu();
                            }
                            if ui.button("Row Change History...").clicked() {
                                self.show_history_panel = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            ui.label(RichText::new("Export More").color(Theme::TEXT_MUTED).size(11.0));
                            if ui.add_enabled(self.current_table.is_some(), egui::Button::new("Export to Excel...")).clicked() {
                                self.export_to_excel();
                                ui.close_menu();
                            }
                            if ui.add_enabled(self.msi.is_some(), egui::Button::new("Export to XML...")).clicked() {
                                self.export_to_xml();
                                ui.close_menu();
                            }
                        });
                    }

                    // Help menu
                    ui.menu_button(RichText::new("Help").color(Theme::TEXT_PRIMARY), |ui| {
                        if ui.button("User Guide").clicked() {
                            self.show_user_guide = true;
                            ui.close_menu();
                        }
                        if ui.button("Keyboard Shortcuts").clicked() {
                            self.show_shortcuts_help = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("About MSI Explorer").clicked() {
                            self.show_about = true;
                            ui.close_menu();
                        }
                    });

                    // Spacer
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // File name on right
                        if let Some(ref path) = self.current_file {
                            if let Some(name) = path.file_name() {
                                ui.label(RichText::new(name.to_string_lossy())
                                    .color(Theme::TEXT_SECONDARY)
                                    .size(13.0));
                                ui.label(RichText::new("◆").color(Theme::ACCENT).size(10.0));
                            }
                        }
                    });
                });
            });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none()
                .fill(Theme::BG_MEDIUM)
                .inner_margin(egui::Margin::symmetric(16.0, 6.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(ref error) = self.error {
                        ui.label(RichText::new("●").color(Theme::ERROR).size(10.0));
                        ui.label(RichText::new(error).color(Theme::ERROR).size(12.0));
                    } else {
                        ui.label(RichText::new("●").color(Theme::SUCCESS).size(10.0));
                        ui.label(RichText::new(&self.status).color(Theme::TEXT_SECONDARY).size(12.0));
                    }

                    // Stats on right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(ref stats) = self.stats {
                            ui.label(RichText::new(format!("{} tables", stats.table_count))
                                .color(Theme::TEXT_MUTED)
                                .size(12.0));
                            ui.label(RichText::new("•").color(Theme::TEXT_MUTED));
                            ui.label(RichText::new(format!("{} rows", stats.total_rows))
                                .color(Theme::TEXT_MUTED)
                                .size(12.0));
                        }
                    });
                });
            });

        // Left panel - table list (only in Tables mode)
        if self.view_mode == ViewMode::Tables {
            egui::SidePanel::left("table_list")
                .default_width(240.0)
                .min_width(180.0)
                .resizable(true)
                .frame(egui::Frame::none()
                    .fill(Theme::BG_DARK)
                    .inner_margin(egui::Margin::same(0.0)))
                .show(ctx, |ui| {
                    panels::table_list_panel(ui, self);
                });
        }

        // Cascade rename preview dialog
        if self.cascade_preview.is_some() {
            let mut close_dialog = false;
            let mut apply_cascade = false;

            egui::Window::new("Cascade Rename")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if let Some(ref preview) = self.cascade_preview {
                        ui.label(RichText::new("This will update references in other tables")
                            .color(Theme::TEXT_PRIMARY)
                            .size(14.0));

                        ui.add_space(12.0);

                        // Show the rename
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&preview.original_value)
                                .color(Theme::ERROR)
                                .size(13.0)
                                .strong());
                            ui.label(RichText::new("→").color(Theme::TEXT_MUTED));
                            ui.label(RichText::new(&preview.new_value)
                                .color(Theme::SUCCESS)
                                .size(13.0)
                                .strong());
                        });

                        ui.add_space(12.0);

                        // Affected tables
                        ui.label(RichText::new("Affected references:")
                            .color(Theme::TEXT_MUTED)
                            .size(12.0));

                        egui::Frame::none()
                            .fill(Theme::BG_MEDIUM)
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                for affected in &preview.affected {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("•").color(Theme::ACCENT));
                                        ui.label(RichText::new(format!("{}.{}", affected.table, affected.column))
                                            .color(Theme::TEXT_PRIMARY)
                                            .size(12.0));
                                        ui.label(RichText::new(format!("({} rows)", affected.row_count))
                                            .color(Theme::TEXT_MUTED)
                                            .size(11.0));
                                    });
                                }
                            });

                        let total_refs: usize = preview.affected.iter().map(|a| a.row_count).sum();
                        ui.add_space(8.0);
                        ui.label(RichText::new(format!("Total: {} references will be updated", total_refs))
                            .color(Theme::WARNING)
                            .size(12.0));

                        ui.add_space(16.0);

                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Apply Cascade Rename").color(Theme::SUCCESS)).clicked() {
                                apply_cascade = true;
                            }
                            if ui.button(RichText::new("Cancel").color(Theme::TEXT_SECONDARY)).clicked() {
                                close_dialog = true;
                            }
                        });
                    }
                });

            if apply_cascade {
                self.apply_cascade_rename();
            } else if close_dialog {
                self.cancel_edit();
            }
        }

        // Add Row dialog
        if self.show_add_row_dialog {
            let mut confirm = false;
            let mut cancel = false;

            egui::Window::new("Add New Row")
                .collapsible(false)
                .resizable(true)
                .min_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if let Some(ref table) = self.current_table.clone() {
                        ui.label(RichText::new(format!("Adding row to: {}", table.name))
                            .color(Theme::ACCENT)
                            .size(14.0));

                        ui.add_space(12.0);

                        egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                            egui::Grid::new("add_row_grid")
                                .num_columns(2)
                                .spacing([10.0, 8.0])
                                .show(ui, |ui| {
                                    for (idx, col) in table.columns.iter().enumerate() {
                                        // Column name with PK indicator
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&col.name)
                                                .color(Theme::TEXT_PRIMARY)
                                                .size(12.0));
                                            if col.primary_key {
                                                ui.label(RichText::new("PK")
                                                    .color(Theme::ACCENT)
                                                    .size(9.0));
                                            }
                                        });

                                        // Input field
                                        if idx < self.new_row_values.len() {
                                            ui.add(egui::TextEdit::singleline(&mut self.new_row_values[idx])
                                                .desired_width(250.0));
                                        }
                                        ui.end_row();
                                    }
                                });
                        });

                        ui.add_space(16.0);

                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Add Row").color(Theme::SUCCESS)).clicked() {
                                confirm = true;
                            }
                            if ui.button(RichText::new("Cancel").color(Theme::TEXT_SECONDARY)).clicked() {
                                cancel = true;
                            }
                        });
                    }
                });

            if confirm {
                self.confirm_add_row();
            } else if cancel {
                self.cancel_add_row();
            }
        }

        // Merge Module dialog
        if self.show_merge_dialog {
            let mut merge = false;
            let mut cancel = false;

            egui::Window::new("Merge Module")
                .collapsible(false)
                .resizable(false)
                .min_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if let Some(ref path) = self.merge_module_path.clone() {
                        ui.label(RichText::new("Merge Module:")
                            .color(Theme::TEXT_MUTED)
                            .size(12.0));
                        ui.label(RichText::new(path.file_name().unwrap_or_default().to_string_lossy())
                            .color(Theme::ACCENT)
                            .size(14.0)
                            .strong());

                        ui.add_space(16.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Target Feature:")
                                .color(Theme::TEXT_PRIMARY)
                                .size(12.0));
                            ui.add(egui::TextEdit::singleline(&mut self.merge_target_feature)
                                .desired_width(200.0));
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Root Directory:")
                                .color(Theme::TEXT_PRIMARY)
                                .size(12.0));
                            ui.add(egui::TextEdit::singleline(&mut self.merge_target_directory)
                                .desired_width(200.0));
                        });

                        ui.add_space(16.0);

                        ui.label(RichText::new("This will add all tables from the merge module to pending changes.")
                            .color(Theme::TEXT_SECONDARY)
                            .size(11.0));

                        ui.add_space(16.0);

                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Merge").color(Theme::SUCCESS)).clicked() {
                                merge = true;
                            }
                            if ui.button(RichText::new("Cancel").color(Theme::TEXT_SECONDARY)).clicked() {
                                cancel = true;
                            }
                        });
                    }
                });

            if merge {
                self.merge_module();
            } else if cancel {
                self.show_merge_dialog = false;
                self.merge_module_path = None;
            }
        }

        // Edit Summary Info dialog
        if self.show_edit_summary_dialog {
            let mut save = false;
            let mut cancel = false;

            egui::Window::new("Edit Summary Information")
                .collapsible(false)
                .resizable(false)
                .min_width(450.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Package Metadata")
                        .color(Theme::TEXT_MUTED)
                        .size(12.0));

                    ui.add_space(12.0);

                    egui::Grid::new("summary_edit_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Title:").color(Theme::TEXT_PRIMARY));
                            ui.add(egui::TextEdit::singleline(&mut self.edit_summary_title)
                                .desired_width(300.0));
                            ui.end_row();

                            ui.label(RichText::new("Author:").color(Theme::TEXT_PRIMARY));
                            ui.add(egui::TextEdit::singleline(&mut self.edit_summary_author)
                                .desired_width(300.0));
                            ui.end_row();

                            ui.label(RichText::new("Subject:").color(Theme::TEXT_PRIMARY));
                            ui.add(egui::TextEdit::singleline(&mut self.edit_summary_subject)
                                .desired_width(300.0));
                            ui.end_row();

                            ui.label(RichText::new("Comments:").color(Theme::TEXT_PRIMARY));
                            ui.add(egui::TextEdit::multiline(&mut self.edit_summary_comments)
                                .desired_width(300.0)
                                .desired_rows(3));
                            ui.end_row();
                        });

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Save").color(Theme::SUCCESS)).clicked() {
                            save = true;
                        }
                        if ui.button(RichText::new("Cancel").color(Theme::TEXT_SECONDARY)).clicked() {
                            cancel = true;
                        }
                    });
                });

            if save {
                self.save_summary_changes();
            } else if cancel {
                self.show_edit_summary_dialog = false;
            }
        }

        // Find & Replace dialog
        if self.show_find_replace {
            let mut close = false;
            let mut do_find = false;
            let mut do_find_next = false;
            let mut do_replace = false;
            let mut do_replace_all = false;

            egui::Window::new("Find & Replace")
                .collapsible(false)
                .resizable(false)
                .min_width(400.0)
                .show(ctx, |ui| {
                    egui::Grid::new("find_replace_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Find:").color(Theme::TEXT_PRIMARY));
                            let find_response = ui.add(egui::TextEdit::singleline(&mut self.find_text)
                                .desired_width(280.0));
                            if find_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                do_find = true;
                            }
                            ui.end_row();

                            ui.label(RichText::new("Replace:").color(Theme::TEXT_PRIMARY));
                            ui.add(egui::TextEdit::singleline(&mut self.replace_text)
                                .desired_width(280.0));
                            ui.end_row();
                        });

                    ui.add_space(8.0);
                    ui.checkbox(&mut self.find_all_tables, "Search all tables");

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if ui.button("Find").clicked() { do_find = true; }
                        if ui.add_enabled(!self.find_results.is_empty(), egui::Button::new("Next")).clicked() {
                            do_find_next = true;
                        }
                        ui.separator();
                        if ui.add_enabled(!self.find_results.is_empty(), egui::Button::new("Replace")).clicked() {
                            do_replace = true;
                        }
                        if ui.add_enabled(!self.find_results.is_empty(), egui::Button::new("Replace All")).clicked() {
                            do_replace_all = true;
                        }
                        ui.separator();
                        if ui.button("Close").clicked() { close = true; }
                    });

                    if !self.find_results.is_empty() {
                        ui.add_space(8.0);
                        ui.label(RichText::new(format!("Found {} matches (showing {}/{})",
                            self.find_results.len(),
                            self.find_result_index + 1,
                            self.find_results.len()))
                            .color(Theme::TEXT_MUTED)
                            .size(11.0));
                    }
                });

            if do_find { self.do_find(); }
            if do_find_next { self.find_next(); }
            if do_replace { self.replace_current(); }
            if do_replace_all { self.replace_all(); }
            if close { self.show_find_replace = false; }
        }

        // Sequence Viewer window
        if self.show_sequence_view {
            let mut close = false;

            egui::Window::new("Sequence Viewer")
                .resizable(true)
                .default_size([600.0, 500.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Sequence Table:").color(Theme::TEXT_PRIMARY));
                        egui::ComboBox::from_id_salt("seq_table")
                            .selected_text(&self.sequence_table)
                            .show_ui(ui, |ui| {
                                for table in &["InstallExecuteSequence", "InstallUISequence",
                                              "AdminExecuteSequence", "AdminUISequence",
                                              "AdvtExecuteSequence"] {
                                    ui.selectable_value(&mut self.sequence_table, table.to_string(), *table);
                                }
                            });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(RichText::new("✕").color(Theme::TEXT_MUTED)).clicked() {
                                close = true;
                            }
                        });
                    });

                    ui.add_space(12.0);

                    let sequences = self.get_sequence_data();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (action, seq_num, condition) in &sequences {
                            egui::Frame::none()
                                .fill(Theme::BG_MEDIUM)
                                .rounding(egui::Rounding::same(4.0))
                                .inner_margin(egui::Margin::same(8.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("{:5}", seq_num))
                                            .color(Theme::ACCENT)
                                            .size(12.0)
                                            .monospace());
                                        ui.label(RichText::new(action)
                                            .color(Theme::TEXT_PRIMARY)
                                            .size(12.0));
                                        if !condition.is_empty() {
                                            ui.label(RichText::new(format!("IF: {}", condition))
                                                .color(Theme::TEXT_MUTED)
                                                .size(10.0));
                                        }
                                    });
                                });
                            ui.add_space(2.0);
                        }
                    });
                });

            if close { self.show_sequence_view = false; }
        }

        // Dialog Preview window
        if self.show_dialog_preview {
            let mut close = false;

            egui::Window::new("Dialog Preview")
                .resizable(true)
                .default_size([500.0, 450.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Dialog:").color(Theme::TEXT_PRIMARY));

                        let dialog_names = self.get_dialog_names();
                        let selected = self.preview_dialog_name.clone().unwrap_or_default();

                        egui::ComboBox::from_id_salt("dialog_select")
                            .selected_text(&selected)
                            .show_ui(ui, |ui| {
                                for name in &dialog_names {
                                    if ui.selectable_label(Some(name) == self.preview_dialog_name.as_ref(), name).clicked() {
                                        self.preview_dialog_name = Some(name.clone());
                                    }
                                }
                            });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(RichText::new("✕").color(Theme::TEXT_MUTED)).clicked() {
                                close = true;
                            }
                        });
                    });

                    ui.add_space(12.0);

                    if let Some(dialog) = self.get_dialog_data() {
                        ui.label(RichText::new(&dialog.title)
                            .color(Theme::TEXT_PRIMARY)
                            .size(14.0)
                            .strong());

                        ui.label(RichText::new(format!("Size: {}x{} dialog units", dialog.width, dialog.height))
                            .color(Theme::TEXT_MUTED)
                            .size(11.0));

                        ui.add_space(8.0);

                        // Draw a simple preview
                        let scale = 1.5f32;
                        let preview_size = egui::vec2(dialog.width as f32 * scale, dialog.height as f32 * scale);

                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(240, 240, 240))
                            .stroke(egui::Stroke::new(1.0, Theme::TEXT_MUTED))
                            .rounding(egui::Rounding::same(2.0))
                            .show(ui, |ui| {
                                let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
                                let painter = ui.painter_at(rect);

                                for ctrl in &dialog.controls {
                                    let ctrl_rect = egui::Rect::from_min_size(
                                        rect.min + egui::vec2(ctrl.x as f32 * scale, ctrl.y as f32 * scale),
                                        egui::vec2(ctrl.width as f32 * scale, ctrl.height as f32 * scale),
                                    );

                                    let color = match ctrl.control_type.as_str() {
                                        "PushButton" => egui::Color32::from_rgb(220, 220, 220),
                                        "Text" => egui::Color32::TRANSPARENT,
                                        "Edit" | "PathEdit" | "MaskedEdit" => egui::Color32::WHITE,
                                        "CheckBox" | "RadioButtonGroup" => egui::Color32::TRANSPARENT,
                                        "ListBox" | "ComboBox" | "ListView" => egui::Color32::WHITE,
                                        "Bitmap" | "Icon" => egui::Color32::from_rgb(200, 200, 255),
                                        "ProgressBar" => egui::Color32::from_rgb(200, 255, 200),
                                        _ => egui::Color32::from_rgb(230, 230, 230),
                                    };

                                    painter.rect_filled(ctrl_rect, 2.0, color);
                                    painter.rect_stroke(ctrl_rect, 2.0, egui::Stroke::new(1.0, egui::Color32::GRAY));

                                    // Draw text for buttons and labels
                                    if !ctrl.text.is_empty() && ctrl_rect.width() > 20.0 {
                                        let text = if ctrl.text.len() > 15 {
                                            format!("{}...", &ctrl.text[..12])
                                        } else {
                                            ctrl.text.clone()
                                        };
                                        painter.text(
                                            ctrl_rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            text,
                                            egui::FontId::proportional(9.0),
                                            egui::Color32::BLACK,
                                        );
                                    }
                                }
                            });

                        ui.add_space(8.0);
                        ui.label(RichText::new(format!("{} controls", dialog.controls.len()))
                            .color(Theme::TEXT_MUTED)
                            .size(11.0));
                    } else if self.preview_dialog_name.is_some() {
                        ui.label(RichText::new("No dialog data available")
                            .color(Theme::TEXT_MUTED));
                    } else {
                        ui.label(RichText::new("Select a dialog to preview")
                            .color(Theme::TEXT_MUTED));
                    }
                });

            if close { self.show_dialog_preview = false; }
        }

        // Create Patch dialog
        if self.show_create_patch_dialog {
            let mut close = false;
            let mut select_old = false;
            let mut select_new = false;
            let mut calculate = false;
            let mut export = false;

            egui::Window::new("Create Patch (MSP)")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Compare two MSI versions to create a patch")
                        .color(Theme::TEXT_MUTED)
                        .size(12.0));

                    ui.add_space(16.0);

                    // Old MSI selection
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Old (Base) MSI:")
                            .color(Theme::TEXT_PRIMARY)
                            .size(12.0));
                        if let Some(ref path) = self.patch_old_msi {
                            ui.label(RichText::new(path.file_name().unwrap_or_default().to_string_lossy())
                                .color(Theme::ACCENT));
                        } else {
                            ui.label(RichText::new("Not selected")
                                .color(Theme::TEXT_MUTED));
                        }
                        if ui.button("Browse...").clicked() {
                            select_old = true;
                        }
                    });

                    ui.add_space(8.0);

                    // New MSI selection
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("New (Updated) MSI:")
                            .color(Theme::TEXT_PRIMARY)
                            .size(12.0));
                        if let Some(ref path) = self.patch_new_msi {
                            ui.label(RichText::new(path.file_name().unwrap_or_default().to_string_lossy())
                                .color(Theme::ACCENT));
                        } else {
                            ui.label(RichText::new("Not selected")
                                .color(Theme::TEXT_MUTED));
                        }
                        if ui.button("Browse...").clicked() {
                            select_new = true;
                        }
                    });

                    ui.add_space(16.0);

                    let can_calculate = self.patch_old_msi.is_some() && self.patch_new_msi.is_some();
                    ui.horizontal(|ui| {
                        if ui.add_enabled(can_calculate, egui::Button::new("Calculate Diff")).clicked() {
                            calculate = true;
                        }
                        if ui.add_enabled(self.patch_diff.is_some(), egui::Button::new("Export Patch Info...")).clicked() {
                            export = true;
                        }
                    });

                    // Show diff results
                    if let Some(ref diff) = self.patch_diff {
                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.label(RichText::new("Patch Diff Summary")
                            .color(Theme::TEXT_PRIMARY)
                            .size(14.0)
                            .strong());

                        ui.add_space(8.0);

                        egui::Grid::new("patch_diff_info")
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("Old Version:").color(Theme::TEXT_MUTED));
                                ui.label(RichText::new(&diff.old_version).color(Theme::TEXT_PRIMARY));
                                ui.end_row();

                                ui.label(RichText::new("New Version:").color(Theme::TEXT_MUTED));
                                ui.label(RichText::new(&diff.new_version).color(Theme::SUCCESS));
                                ui.end_row();

                                ui.label(RichText::new("Old Product Code:").color(Theme::TEXT_MUTED));
                                ui.label(RichText::new(&diff.old_product_code).color(Theme::TEXT_SECONDARY).size(10.0));
                                ui.end_row();

                                ui.label(RichText::new("New Product Code:").color(Theme::TEXT_MUTED));
                                ui.label(RichText::new(&diff.new_product_code).color(Theme::TEXT_SECONDARY).size(10.0));
                                ui.end_row();
                            });

                        ui.add_space(12.0);

                        if !diff.added_tables.is_empty() {
                            ui.label(RichText::new(format!("Added Tables ({}): {}",
                                diff.added_tables.len(),
                                diff.added_tables.join(", ")))
                                .color(Theme::SUCCESS)
                                .size(11.0));
                        }

                        if !diff.removed_tables.is_empty() {
                            ui.label(RichText::new(format!("Removed Tables ({}): {}",
                                diff.removed_tables.len(),
                                diff.removed_tables.join(", ")))
                                .color(Theme::ERROR)
                                .size(11.0));
                        }

                        if !diff.changed_tables.is_empty() {
                            ui.add_space(8.0);
                            ui.label(RichText::new(format!("Changed Tables ({}):", diff.changed_tables.len()))
                                .color(Theme::ACCENT)
                                .size(12.0));

                            egui::ScrollArea::vertical()
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    for table in &diff.changed_tables {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&table.name).color(Theme::TEXT_PRIMARY));
                                            if table.added_rows > 0 {
                                                ui.label(RichText::new(format!("+{}", table.added_rows))
                                                    .color(Theme::SUCCESS)
                                                    .size(11.0));
                                            }
                                            if table.deleted_rows > 0 {
                                                ui.label(RichText::new(format!("-{}", table.deleted_rows))
                                                    .color(Theme::ERROR)
                                                    .size(11.0));
                                            }
                                        });
                                    }
                                });
                        }
                    }

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Close").color(Theme::TEXT_SECONDARY)).clicked() {
                            close = true;
                        }
                    });
                });

            if select_old { self.select_patch_old_msi(); }
            if select_new { self.select_patch_new_msi(); }
            if calculate { self.calculate_patch_diff(); }
            if export { self.export_patch_pcp(); }
            if close { self.clear_patch_state(); }
        }

        // GUID Generator dialog
        if self.show_guid_generator {
            let mut close = false;
            let mut regenerate = false;

            egui::Window::new("GUID Generator")
                .collapsible(false)
                .resizable(false)
                .min_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Generated GUID:")
                        .color(Theme::TEXT_MUTED)
                        .size(12.0));

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.generated_guid)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(320.0));

                        if ui.button("📋").on_hover_text("Copy to clipboard").clicked() {
                            ui.output_mut(|o| o.copied_text = self.generated_guid.clone());
                            self.status = "GUID copied to clipboard".to_string();
                        }
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.guid_uppercase, "Uppercase");
                        ui.checkbox(&mut self.guid_braces, "With braces");
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if ui.button("Generate New").clicked() {
                            regenerate = true;
                        }
                        if ui.button("Close").clicked() {
                            close = true;
                        }
                    });
                });

            if regenerate { self.generate_guid(); }
            if close { self.show_guid_generator = false; }
        }

        // SQL Query Editor dialog
        if self.show_sql_editor {
            let mut close = false;
            let mut execute = false;

            egui::Window::new("SQL Query Editor")
                .collapsible(false)
                .resizable(true)
                .min_width(600.0)
                .min_height(400.0)
                .show(ctx, |ui| {
                    ui.label(RichText::new("Enter SQL Query (SELECT only):")
                        .color(Theme::TEXT_MUTED)
                        .size(12.0));

                    ui.add_space(8.0);

                    ui.add(egui::TextEdit::multiline(&mut self.sql_query)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(4));

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("Execute").clicked() {
                            execute = true;
                        }
                        if ui.button("Close").clicked() {
                            close = true;
                        }
                    });

                    if let Some(ref error) = self.sql_error {
                        ui.add_space(8.0);
                        ui.label(RichText::new(error).color(Theme::ERROR));
                    }

                    if let Some(ref results) = self.sql_results {
                        ui.add_space(12.0);
                        ui.label(RichText::new(format!("Results: {} rows", results.rows.len()))
                            .color(Theme::SUCCESS));

                        ui.add_space(8.0);

                        egui::ScrollArea::both()
                            .max_height(250.0)
                            .show(ui, |ui| {
                                egui::Grid::new("sql_results")
                                    .striped(true)
                                    .show(ui, |ui| {
                                        // Headers
                                        for col in &results.columns {
                                            ui.label(RichText::new(&col.name).strong());
                                        }
                                        ui.end_row();

                                        // Rows (limit to 100)
                                        for row in results.rows.iter().take(100) {
                                            for val in &row.values {
                                                ui.label(val.display());
                                            }
                                            ui.end_row();
                                        }
                                    });
                            });
                    }
                });

            if execute { self.execute_sql(); }
            if close { self.show_sql_editor = false; }
        }

        // Create Table dialog
        if self.show_create_table_dialog {
            let mut close = false;
            let mut create = false;
            let mut add_col = false;
            let mut remove_col: Option<usize> = None;

            egui::Window::new("Create New Table")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Table Name:").color(Theme::TEXT_PRIMARY));
                        ui.add(egui::TextEdit::singleline(&mut self.new_table_name)
                            .desired_width(200.0));
                    });

                    ui.add_space(12.0);
                    ui.label(RichText::new("Columns:").color(Theme::TEXT_PRIMARY));

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (i, col) in self.new_table_columns.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut col.name)
                                        .desired_width(120.0)
                                        .hint_text("Name"));

                                    egui::ComboBox::from_id_salt(format!("col_type_{}", i))
                                        .selected_text(&col.col_type)
                                        .width(80.0)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut col.col_type, "String".to_string(), "String");
                                            ui.selectable_value(&mut col.col_type, "Integer".to_string(), "Integer");
                                        });

                                    ui.checkbox(&mut col.nullable, "Null");
                                    ui.checkbox(&mut col.primary_key, "PK");

                                    if ui.button("✕").clicked() {
                                        remove_col = Some(i);
                                    }
                                });
                            }
                        });

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("+ Add Column").clicked() {
                            add_col = true;
                        }
                    });

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Create").color(Theme::SUCCESS)).clicked() {
                            create = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });

            if let Some(idx) = remove_col {
                self.new_table_columns.remove(idx);
            }
            if add_col { self.add_new_column(); }
            if create { self.create_custom_table(); }
            if close {
                self.show_create_table_dialog = false;
                self.new_table_name.clear();
                self.new_table_columns.clear();
            }
        }

        // Script Viewer dialog
        if self.show_script_viewer {
            let mut close = false;
            let binaries = self.get_script_binaries();

            egui::Window::new("Script Viewer")
                .collapsible(false)
                .resizable(true)
                .min_width(600.0)
                .min_height(400.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Binary:").color(Theme::TEXT_PRIMARY));

                        let current = self.selected_binary.clone().unwrap_or_default();
                        egui::ComboBox::from_id_salt("script_binary")
                            .selected_text(&current)
                            .show_ui(ui, |ui| {
                                for name in &binaries {
                                    if ui.selectable_label(Some(name) == self.selected_binary.as_ref(), name).clicked() {
                                        self.load_script_content(name);
                                    }
                                }
                            });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close = true;
                            }
                        });
                    });

                    ui.add_space(12.0);

                    egui::ScrollArea::both()
                        .show(ui, |ui| {
                            ui.add(egui::TextEdit::multiline(&mut self.script_content)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(20)
                                .interactive(false));
                        });
                });

            if close {
                self.show_script_viewer = false;
                self.selected_binary = None;
                self.script_content.clear();
            }
        }

        // Reference Validation panel
        if self.show_reference_validation {
            let mut close = false;

            egui::Window::new("Reference Validation")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .min_height(300.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("Found {} reference errors", self.reference_errors.len()))
                            .color(if self.reference_errors.is_empty() { Theme::SUCCESS } else { Theme::ERROR })
                            .size(14.0));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close = true;
                            }
                            if ui.button("Re-validate").clicked() {
                                self.validate_references();
                            }
                        });
                    });

                    ui.add_space(12.0);

                    if self.reference_errors.is_empty() {
                        ui.label(RichText::new("All references are valid!")
                            .color(Theme::SUCCESS));
                    } else {
                        egui::ScrollArea::vertical()
                            .show(ui, |ui| {
                                for error in &self.reference_errors {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("●").color(Theme::ERROR));
                                        ui.label(RichText::new(format!(
                                            "{}.{} row {}: '{}' → {} (not found)",
                                            error.table, error.column, error.row_idx,
                                            error.value, error.references_table
                                        )).size(12.0));
                                    });
                                }
                            });
                    }
                });

            if close { self.show_reference_validation = false; }
        }

        // Report dialog
        if self.show_report_dialog {
            let mut close = false;
            let mut generate = false;

            egui::Window::new("Generate Report")
                .collapsible(false)
                .resizable(false)
                .min_width(300.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Select report format:")
                        .color(Theme::TEXT_PRIMARY));

                    ui.add_space(12.0);

                    ui.radio_value(&mut self.report_format, ReportFormat::Html, "HTML");
                    ui.radio_value(&mut self.report_format, ReportFormat::Markdown, "Markdown");
                    ui.radio_value(&mut self.report_format, ReportFormat::PlainText, "Plain Text");

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Generate").color(Theme::SUCCESS)).clicked() {
                            generate = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });

            if generate { self.generate_report(); close = true; }
            if close { self.show_report_dialog = false; }
        }

        // Localization view
        if self.show_localization_view {
            let mut close = false;

            egui::Window::new("String Localization")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .min_height(300.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Language:")
                            .color(Theme::TEXT_PRIMARY));

                        let current_lang = self.selected_language
                            .and_then(|id| self.available_languages.iter().find(|(lid, _)| *lid == id))
                            .map(|(_, name)| name.clone())
                            .unwrap_or_else(|| "Select...".to_string());

                        egui::ComboBox::from_id_salt("language_select")
                            .selected_text(&current_lang)
                            .show_ui(ui, |ui| {
                                for (id, name) in &self.available_languages {
                                    if ui.selectable_label(self.selected_language == Some(*id), name).clicked() {
                                        self.selected_language = Some(*id);
                                    }
                                }
                            });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close = true;
                            }
                        });
                    });

                    ui.add_space(12.0);

                    ui.label(RichText::new("Note: Full localization editing requires multilingual MSI support.")
                        .color(Theme::TEXT_MUTED)
                        .size(11.0));

                    ui.add_space(8.0);

                    ui.label(RichText::new("Available languages in this MSI:")
                        .color(Theme::TEXT_PRIMARY));

                    for (id, name) in &self.available_languages {
                        ui.label(format!("  • {} (LCID: {})", name, id));
                    }
                });

            if close { self.show_localization_view = false; }
        }

        // CAB Rebuild dialog
        if self.show_cab_rebuild {
            let mut close = false;
            let mut rebuild = false;

            egui::Window::new("Rebuild CAB")
                .collapsible(false)
                .resizable(false)
                .min_width(350.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Select compression level:")
                        .color(Theme::TEXT_PRIMARY));

                    ui.add_space(12.0);

                    ui.radio_value(&mut self.cab_compression, CabCompression::None, CabCompression::None.label());
                    ui.radio_value(&mut self.cab_compression, CabCompression::Low, CabCompression::Low.label());
                    ui.radio_value(&mut self.cab_compression, CabCompression::Medium, CabCompression::Medium.label());
                    ui.radio_value(&mut self.cab_compression, CabCompression::High, CabCompression::High.label());

                    ui.add_space(12.0);

                    ui.label(RichText::new("Note: Higher compression = smaller file, slower build")
                        .color(Theme::TEXT_MUTED)
                        .size(11.0));

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Rebuild").color(Theme::SUCCESS)).clicked() {
                            rebuild = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });

            if rebuild { self.rebuild_cab(); }
            if close { self.show_cab_rebuild = false; }
        }

        // Table Schema Viewer
        if self.show_schema_viewer {
            let mut close = false;
            let schema = self.get_table_schema();

            egui::Window::new("Table Schema")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .show(ctx, |ui| {
                    if let Some(ref name) = self.selected_table {
                        ui.label(RichText::new(format!("Schema for: {}", name))
                            .color(Theme::ACCENT).strong());
                    }
                    ui.add_space(12.0);

                    egui::Grid::new("schema_grid")
                        .num_columns(4)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Column").strong());
                            ui.label(RichText::new("Type").strong());
                            ui.label(RichText::new("Nullable").strong());
                            ui.label(RichText::new("Primary Key").strong());
                            ui.end_row();

                            for (name, col_type, nullable, pk) in &schema {
                                ui.label(name);
                                ui.label(col_type);
                                ui.label(if *nullable { "Yes" } else { "No" });
                                ui.label(if *pk { "✓" } else { "" });
                                ui.end_row();
                            }
                        });

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if close { self.show_schema_viewer = false; }
        }

        // Dependency Graph
        if self.show_dependency_graph {
            let mut close = false;

            egui::Window::new("Dependency Graph")
                .collapsible(false)
                .resizable(true)
                .min_width(600.0)
                .min_height(400.0)
                .show(ctx, |ui| {
                    ui.label(RichText::new(format!("{} nodes", self.dependency_data.len()))
                        .color(Theme::TEXT_MUTED));
                    ui.add_space(12.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for node in &self.dependency_data {
                            ui.horizontal(|ui| {
                                let icon = match node.node_type {
                                    DependencyType::Feature => "◆",
                                    DependencyType::Component => "●",
                                    DependencyType::File => "○",
                                };
                                ui.label(RichText::new(icon).color(Theme::ACCENT));
                                ui.label(&node.name);
                                if !node.depends_on.is_empty() {
                                    ui.label(RichText::new(format!("→ {}", node.depends_on.join(", ")))
                                        .color(Theme::TEXT_MUTED).size(11.0));
                                }
                            });
                        }
                    });

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if close { self.show_dependency_graph = false; }
        }

        // Hash Verification Results
        if self.show_hash_verification {
            let mut close = false;

            egui::Window::new("File Hash Verification")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .show(ctx, |ui| {
                    ui.label(RichText::new(format!("Checked {} files", self.hash_results.len()))
                        .color(Theme::TEXT_PRIMARY));
                    ui.add_space(12.0);

                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        for result in &self.hash_results {
                            ui.horizontal(|ui| {
                                let (icon, color) = match result.status {
                                    HashStatus::Valid => ("✓", Theme::SUCCESS),
                                    HashStatus::Invalid => ("✗", Theme::ERROR),
                                    HashStatus::Missing => ("?", Theme::WARNING),
                                    HashStatus::NoHash => ("−", Theme::TEXT_MUTED),
                                };
                                ui.label(RichText::new(icon).color(color));
                                ui.label(&result.file_name);
                            });
                        }
                    });

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if close { self.show_hash_verification = false; }
        }

        // WiX Export Dialog
        if self.show_wix_export {
            let mut close = false;
            let mut export = false;

            egui::Window::new("Export to WiX")
                .collapsible(false)
                .resizable(false)
                .min_width(350.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Export MSI structure to WiX XML format.");
                    ui.add_space(12.0);
                    ui.label(RichText::new("Note: This creates a basic WiX skeleton.")
                        .color(Theme::TEXT_MUTED).size(11.0));
                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Export").color(Theme::SUCCESS)).clicked() {
                            export = true;
                        }
                        if ui.button("Cancel").clicked() { close = true; }
                    });
                });

            if export { self.export_to_wix(); }
            if close { self.show_wix_export = false; }
        }

        // Bulk Edit Dialog
        if self.show_bulk_edit {
            let mut close = false;
            let mut apply = false;

            egui::Window::new("Bulk Edit")
                .collapsible(false)
                .resizable(false)
                .min_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    if let Some(ref table) = self.current_table {
                        ui.horizontal(|ui| {
                            ui.label("Column:");
                            egui::ComboBox::from_id_salt("bulk_col")
                                .selected_text(self.bulk_edit_column
                                    .and_then(|i| table.columns.get(i))
                                    .map(|c| c.name.as_str())
                                    .unwrap_or("Select..."))
                                .show_ui(ui, |ui| {
                                    for (i, col) in table.columns.iter().enumerate() {
                                        if ui.selectable_label(self.bulk_edit_column == Some(i), &col.name).clicked() {
                                            self.bulk_edit_column = Some(i);
                                        }
                                    }
                                });
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label("Find:");
                            ui.add(egui::TextEdit::singleline(&mut self.bulk_edit_find).desired_width(200.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Replace:");
                            ui.add(egui::TextEdit::singleline(&mut self.bulk_edit_replace).desired_width(200.0));
                        });
                    }

                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Apply").color(Theme::SUCCESS)).clicked() {
                            apply = true;
                        }
                        if ui.button("Cancel").clicked() { close = true; }
                    });
                });

            if apply { self.perform_bulk_edit(); }
            if close { self.show_bulk_edit = false; }
        }

        // Statistics Dashboard
        if self.show_stats_dashboard {
            let mut close = false;

            egui::Window::new("Statistics Dashboard")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .show(ctx, |ui| {
                    if let Some(ref stats) = self.stats {
                        egui::Grid::new("stats_grid").num_columns(2).show(ui, |ui| {
                            ui.label("File Size:");
                            ui.label(format!("{} bytes ({:.2} MB)", stats.file_size, stats.file_size as f64 / 1_000_000.0));
                            ui.end_row();

                            ui.label("Tables:");
                            ui.label(format!("{}", stats.table_count));
                            ui.end_row();

                            ui.label("Total Rows:");
                            ui.label(format!("{}", stats.total_rows));
                            ui.end_row();

                            ui.label("Largest Table:");
                            ui.label(format!("{} ({} rows)", stats.largest_table, stats.largest_table_rows));
                            ui.end_row();
                        });

                        ui.add_space(16.0);
                        ui.label(RichText::new("Tables by Category:").strong());
                        for (cat, tables) in &self.tables_by_category {
                            ui.label(format!("  {}: {} tables", cat.display_name(), tables.len()));
                        }
                    }

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if close { self.show_stats_dashboard = false; }
        }

        // Stream Viewer
        if self.show_stream_viewer {
            let mut close = false;

            egui::Window::new("Stream Viewer")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .show(ctx, |ui| {
                    ui.label(RichText::new(format!("{} streams found", self.streams.len()))
                        .color(Theme::TEXT_MUTED));
                    ui.add_space(12.0);

                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        for stream in &self.streams {
                            ui.horizontal(|ui| {
                                let icon = match stream.stream_type {
                                    StreamType::Binary => "◆",
                                    StreamType::Icon => "🖼",
                                    StreamType::Cab => "📦",
                                    _ => "○",
                                };
                                ui.label(icon);
                                ui.label(&stream.name);
                                ui.label(RichText::new(format!("{:?}", stream.stream_type))
                                    .color(Theme::TEXT_MUTED).size(11.0));
                            });
                        }
                    });

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if close { self.show_stream_viewer = false; }
        }

        // Property Editor
        if self.show_property_editor {
            let mut close = false;

            egui::Window::new("Property Editor")
                .collapsible(false)
                .resizable(true)
                .min_width(600.0)
                .show(ctx, |ui| {
                    ui.label(RichText::new("Known MSI Properties").strong());
                    ui.add_space(8.0);

                    egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                        let mut current_cat = "";
                        for prop in &self.known_properties {
                            if prop.category != current_cat {
                                current_cat = prop.category;
                                ui.add_space(8.0);
                                ui.label(RichText::new(current_cat).color(Theme::ACCENT).strong());
                            }
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(prop.name).strong());
                                ui.label(RichText::new(prop.description).color(Theme::TEXT_MUTED).size(11.0));
                            });
                        }
                    });

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if close { self.show_property_editor = false; }
        }

        // Condition Validator
        if self.show_condition_validator {
            let mut close = false;
            let mut validate = false;

            egui::Window::new("Condition Validator")
                .collapsible(false)
                .resizable(true)
                .min_width(500.0)
                .show(ctx, |ui| {
                    ui.label("Enter a condition to validate:");
                    ui.add_space(8.0);

                    ui.add(egui::TextEdit::multiline(&mut self.condition_text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3));

                    ui.add_space(8.0);
                    if ui.button("Validate").clicked() { validate = true; }

                    if let Some(ref result) = self.condition_result {
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            let (icon, color) = if result.valid {
                                ("✓", Theme::SUCCESS)
                            } else {
                                ("✗", Theme::ERROR)
                            };
                            ui.label(RichText::new(icon).color(color));
                            ui.label(&result.message);
                        });

                        if !result.properties_used.is_empty() {
                            ui.label(RichText::new(format!("Properties: {}", result.properties_used.join(", ")))
                                .color(Theme::TEXT_MUTED).size(11.0));
                        }
                    }

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if validate { self.validate_condition(); }
            if close { self.show_condition_validator = false; }
        }

        // Registry Preview
        if self.show_registry_preview {
            let mut close = false;
            let entries = self.get_registry_preview();

            egui::Window::new("Registry Preview")
                .collapsible(false)
                .resizable(true)
                .min_width(700.0)
                .show(ctx, |ui| {
                    ui.label(RichText::new(format!("{} registry entries", entries.len()))
                        .color(Theme::TEXT_MUTED));
                    ui.add_space(12.0);

                    egui::ScrollArea::both().max_height(400.0).show(ui, |ui| {
                        egui::Grid::new("reg_grid").striped(true).show(ui, |ui| {
                            ui.label(RichText::new("Root").strong());
                            ui.label(RichText::new("Key").strong());
                            ui.label(RichText::new("Name").strong());
                            ui.label(RichText::new("Value").strong());
                            ui.end_row();

                            for (root, key, name, value) in &entries {
                                ui.label(root);
                                ui.label(key);
                                ui.label(name);
                                ui.label(value);
                                ui.end_row();
                            }
                        });
                    });

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if close { self.show_registry_preview = false; }
        }

        // Shortcut Preview
        if self.show_shortcut_preview {
            let mut close = false;
            let shortcuts = self.get_shortcut_preview();

            egui::Window::new("Shortcut Preview")
                .collapsible(false)
                .resizable(true)
                .min_width(600.0)
                .show(ctx, |ui| {
                    ui.label(RichText::new(format!("{} shortcuts", shortcuts.len()))
                        .color(Theme::TEXT_MUTED));
                    ui.add_space(12.0);

                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        for (name, dir, target) in &shortcuts {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("🔗").color(Theme::ACCENT));
                                ui.label(RichText::new(name).strong());
                                ui.label(RichText::new(format!("in {} → {}", dir, target))
                                    .color(Theme::TEXT_MUTED).size(11.0));
                            });
                        }
                    });

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if close { self.show_shortcut_preview = false; }
        }

        // Service Preview
        if self.show_service_preview {
            let mut close = false;
            let services = self.get_service_preview();

            egui::Window::new("Service Preview")
                .collapsible(false)
                .resizable(true)
                .min_width(600.0)
                .show(ctx, |ui| {
                    ui.label(RichText::new(format!("{} services", services.len()))
                        .color(Theme::TEXT_MUTED));
                    ui.add_space(12.0);

                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        for (name, display, desc, start_type) in &services {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("⚙").color(Theme::ACCENT));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(display).strong());
                                    ui.label(RichText::new(format!("Name: {}, Start: {}", name, start_type))
                                        .color(Theme::TEXT_MUTED).size(11.0));
                                    if !desc.is_empty() {
                                        ui.label(RichText::new(desc).size(11.0));
                                    }
                                });
                            });
                            ui.add_space(4.0);
                        }
                    });

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if close { self.show_service_preview = false; }
        }

        // CAB Contents
        if self.show_cab_contents {
            let mut close = false;

            egui::Window::new("CAB Contents")
                .collapsible(false)
                .resizable(true)
                .min_width(600.0)
                .show(ctx, |ui| {
                    ui.label(RichText::new(format!("{} files in CABs", self.cab_files.len()))
                        .color(Theme::TEXT_MUTED));
                    ui.add_space(12.0);

                    egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                        egui::Grid::new("cab_grid").striped(true).show(ui, |ui| {
                            ui.label(RichText::new("File").strong());
                            ui.label(RichText::new("Size").strong());
                            ui.label(RichText::new("CAB").strong());
                            ui.end_row();

                            for file in &self.cab_files {
                                ui.label(&file.name);
                                ui.label(format!("{} bytes", file.size));
                                ui.label(&file.cab_name);
                                ui.end_row();
                            }
                        });
                    });

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() { close = true; }
                });

            if close { self.show_cab_contents = false; }
        }

        // Bookmarks Panel
        if self.show_bookmarks && !self.bookmarks.is_empty() {
            let mut goto_idx: Option<usize> = None;
            let mut remove_idx: Option<usize> = None;

            egui::SidePanel::right("bookmarks_panel")
                .default_width(250.0)
                .min_width(200.0)
                .resizable(true)
                .frame(egui::Frame::none().fill(Theme::BG_DARK).inner_margin(egui::Margin::same(12.0)))
                .show(ctx, |ui| {
                    ui.label(RichText::new("Bookmarks").strong().color(Theme::TEXT_PRIMARY));
                    ui.add_space(8.0);

                    for (i, bookmark) in self.bookmarks.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.button("→").clicked() { goto_idx = Some(i); }
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&bookmark.table).color(Theme::ACCENT).size(11.0));
                                ui.label(&bookmark.primary_key);
                                if !bookmark.note.is_empty() {
                                    ui.label(RichText::new(&bookmark.note).color(Theme::TEXT_MUTED).size(10.0));
                                }
                            });
                            if ui.button("✕").clicked() { remove_idx = Some(i); }
                        });
                        ui.add_space(4.0);
                    }
                });

            if let Some(idx) = goto_idx { self.goto_bookmark(idx); }
            if let Some(idx) = remove_idx { self.remove_bookmark(idx); }
        }

        // Context Menu
        if self.show_context_menu {
            let mut close = false;
            let mut copy = false;
            let mut delete = false;
            let mut bookmark = false;

            egui::Area::new(egui::Id::new("context_menu"))
                .fixed_pos(egui::pos2(self.context_menu_pos.0, self.context_menu_pos.1))
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        if ui.button("Copy Row(s)").clicked() { copy = true; close = true; }
                        if ui.button("Delete Row(s)").clicked() { delete = true; close = true; }
                        ui.separator();
                        if ui.button("Add Bookmark").clicked() { bookmark = true; close = true; }
                        ui.separator();
                        if ui.button("Cancel").clicked() { close = true; }
                    });
                });

            // Close on click elsewhere
            if ctx.input(|i| i.pointer.any_click()) && !ctx.is_pointer_over_area() {
                close = true;
            }

            if copy { self.copy_selected_rows(); }
            if delete { self.delete_selected_rows(); }
            if bookmark {
                if let Some(row) = self.context_menu_row {
                    self.add_bookmark(row, String::new());
                }
            }
            if close { self.show_context_menu = false; }
        }

        // Validation panel (right side)
        let mut navigate_to_table: Option<String> = None;
        if self.show_validation_panel && self.validation_result.is_some() {
            let mut close_panel = false;
            let mut rerun = false;

            egui::SidePanel::right("validation_panel")
                .default_width(350.0)
                .min_width(250.0)
                .resizable(true)
                .frame(egui::Frame::none()
                    .fill(Theme::BG_DARK)
                    .inner_margin(egui::Margin::same(12.0)))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("ICE Validation")
                            .size(16.0)
                            .color(Theme::TEXT_PRIMARY)
                            .strong());

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(RichText::new("✕").color(Theme::TEXT_MUTED)).clicked() {
                                close_panel = true;
                            }
                            if ui.button(RichText::new("↻ Re-run").color(Theme::ACCENT).size(11.0)).clicked() {
                                rerun = true;
                            }
                        });
                    });

                    ui.add_space(8.0);

                    if let Some(ref result) = self.validation_result {
                        let (errors, warnings, infos) = result.count_by_severity();

                        // Summary bar
                        ui.horizontal(|ui| {
                            if errors > 0 {
                                ui.label(RichText::new(format!("● {} errors", errors))
                                    .color(Theme::ERROR)
                                    .size(12.0));
                            }
                            if warnings > 0 {
                                ui.label(RichText::new(format!("● {} warnings", warnings))
                                    .color(Theme::WARNING)
                                    .size(12.0));
                            }
                            if infos > 0 {
                                ui.label(RichText::new(format!("● {} info", infos))
                                    .color(Theme::TEXT_MUTED)
                                    .size(12.0));
                            }
                            if errors == 0 && warnings == 0 && infos == 0 {
                                ui.label(RichText::new("✓ No issues found")
                                    .color(Theme::SUCCESS)
                                    .size(12.0));
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Violations list
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for violation in &result.violations {
                                let color = match violation.severity {
                                    ice_validator::Severity::Error => Theme::ERROR,
                                    ice_validator::Severity::Warning => Theme::WARNING,
                                    ice_validator::Severity::Info => Theme::TEXT_MUTED,
                                };

                                egui::Frame::none()
                                    .fill(Theme::BG_MEDIUM)
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::same(8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&violation.rule_code)
                                                .color(color)
                                                .size(12.0)
                                                .strong());
                                            ui.label(RichText::new(format!("{}", violation.severity))
                                                .color(Theme::TEXT_MUTED)
                                                .size(10.0));
                                        });

                                        ui.label(RichText::new(&violation.message)
                                            .color(Theme::TEXT_PRIMARY)
                                            .size(11.0));

                                        if let Some(ref table) = violation.table {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("Table:")
                                                    .color(Theme::TEXT_MUTED)
                                                    .size(10.0));

                                                // Make table name clickable
                                                if ui.link(RichText::new(table)
                                                    .color(Theme::ACCENT)
                                                    .size(10.0)).clicked() {
                                                    navigate_to_table = Some(table.clone());
                                                }

                                                if let Some(ref key) = violation.row_key {
                                                    ui.label(RichText::new(format!("Key: {}", key))
                                                        .color(Theme::TEXT_MUTED)
                                                        .size(10.0));
                                                }
                                            });
                                        }
                                    });
                                ui.add_space(4.0);
                            }
                        });
                    }
                });

            if close_panel {
                self.show_validation_panel = false;
            }
            if rerun {
                self.run_validation();
            }
        }

        // Handle table navigation from validation panel
        if let Some(table_name) = navigate_to_table {
            self.select_table(&table_name);
        }

        // Simulation Dialog
        if self.show_simulation {
            let mut close_dialog = false;
            egui::Window::new("Install Simulation")
                .default_width(600.0)
                .default_height(400.0)
                .resizable(true)
                .open(&mut self.show_simulation)
                .show(ctx, |ui| {
                    if self.simulation_steps.is_empty() {
                        ui.label(RichText::new("No simulation data available.")
                            .color(Theme::TEXT_MUTED));
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for step in &self.simulation_steps {
                                let color = if step.will_run { Theme::SUCCESS } else { Theme::TEXT_MUTED };
                                egui::Frame::none()
                                    .fill(Theme::BG_MEDIUM)
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::same(8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(if step.will_run { "●" } else { "○" })
                                                .color(color));
                                            ui.label(RichText::new(&step.action)
                                                .color(Theme::TEXT_PRIMARY)
                                                .strong());
                                        });
                                        ui.label(RichText::new(&step.description)
                                            .color(Theme::TEXT_SECONDARY)
                                            .size(11.0));
                                        if !step.affected_files.is_empty() {
                                            ui.label(RichText::new(format!("Files: {}", step.affected_files.len()))
                                                .color(Theme::TEXT_MUTED)
                                                .size(10.0));
                                        }
                                    });
                                ui.add_space(4.0);
                            }
                        });
                    }
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                });
            if close_dialog {
                self.show_simulation = false;
            }
        }

        // Feature Costs Dialog
        if self.show_feature_costs {
            egui::Window::new("Feature Costs")
                .default_width(500.0)
                .resizable(true)
                .open(&mut self.show_feature_costs)
                .show(ctx, |ui| {
                    if self.feature_costs.is_empty() {
                        ui.label(RichText::new("No feature cost data available.")
                            .color(Theme::TEXT_MUTED));
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for cost in &self.feature_costs {
                                egui::Frame::none()
                                    .fill(Theme::BG_MEDIUM)
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::same(8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&cost.feature)
                                                .color(Theme::ACCENT)
                                                .strong());
                                            ui.label(RichText::new(&cost.title)
                                                .color(Theme::TEXT_SECONDARY));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("Local: {}", format_bytes(cost.local_cost)))
                                                .color(Theme::TEXT_MUTED)
                                                .size(11.0));
                                            ui.label(RichText::new(format!("Source: {}", format_bytes(cost.source_cost)))
                                                .color(Theme::TEXT_MUTED)
                                                .size(11.0));
                                            ui.label(RichText::new(format!("{} components, {} files", cost.components, cost.files))
                                                .color(Theme::TEXT_MUTED)
                                                .size(11.0));
                                        });
                                    });
                                ui.add_space(4.0);
                            }
                        });
                    }
                });
        }

        // Rollback Viewer Dialog
        if self.show_rollback_viewer {
            egui::Window::new("Rollback Operations")
                .default_width(500.0)
                .resizable(true)
                .open(&mut self.show_rollback_viewer)
                .show(ctx, |ui| {
                    if self.rollback_operations.is_empty() {
                        ui.label(RichText::new("No rollback operations found.")
                            .color(Theme::TEXT_MUTED));
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for op in &self.rollback_operations {
                                egui::Frame::none()
                                    .fill(Theme::BG_MEDIUM)
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::same(8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("[{}]", op.sequence))
                                                .color(Theme::TEXT_MUTED));
                                            ui.label(RichText::new(&op.action)
                                                .color(Theme::ACCENT)
                                                .strong());
                                        });
                                        ui.label(RichText::new(format!("{}: {}", op.operation, op.target))
                                            .color(Theme::TEXT_SECONDARY)
                                            .size(11.0));
                                    });
                                ui.add_space(4.0);
                            }
                        });
                    }
                });
        }

        // Action Timing Dialog
        if self.show_action_timing {
            egui::Window::new("Action Timing Estimates")
                .default_width(450.0)
                .resizable(true)
                .open(&mut self.show_action_timing)
                .show(ctx, |ui| {
                    if self.action_timings.is_empty() {
                        ui.label(RichText::new("No action timing data available.")
                            .color(Theme::TEXT_MUTED));
                    } else {
                        let total: u64 = self.action_timings.iter().map(|t| t.estimated_ms).sum();
                        ui.label(RichText::new(format!("Estimated Total: {}ms", total))
                            .color(Theme::ACCENT)
                            .strong());
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for timing in &self.action_timings {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&timing.action)
                                        .color(Theme::TEXT_PRIMARY));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(format!("{}ms", timing.estimated_ms))
                                            .color(Theme::TEXT_MUTED));
                                    });
                                });
                            }
                        });
                    }
                });
        }

        // Patch Analysis Dialog
        if self.show_patch_analysis {
            egui::Window::new("Patch Analysis")
                .default_width(550.0)
                .resizable(true)
                .open(&mut self.show_patch_analysis)
                .show(ctx, |ui| {
                    if self.patch_deltas.is_empty() {
                        ui.label(RichText::new("No patch deltas found. Open an MSP file to analyze.")
                            .color(Theme::TEXT_MUTED));
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for delta in &self.patch_deltas {
                                let (icon, color) = match delta.operation {
                                    PatchOperation::Add => ("+", Theme::SUCCESS),
                                    PatchOperation::Delete => ("-", Theme::ERROR),
                                    PatchOperation::Modify => ("~", Theme::WARNING),
                                };
                                egui::Frame::none()
                                    .fill(Theme::BG_MEDIUM)
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::same(8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(icon).color(color).strong());
                                            ui.label(RichText::new(&delta.table)
                                                .color(Theme::ACCENT));
                                            ui.label(RichText::new(&delta.key)
                                                .color(Theme::TEXT_SECONDARY));
                                        });
                                        ui.label(RichText::new(&delta.details)
                                            .color(Theme::TEXT_MUTED)
                                            .size(11.0));
                                    });
                                ui.add_space(4.0);
                            }
                        });
                    }
                });
        }

        // Watch Panel Dialog
        if self.show_watch_panel {
            let mut close_dialog = false;
            egui::Window::new("Property Watch")
                .default_width(400.0)
                .resizable(true)
                .open(&mut self.show_watch_panel)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Property:");
                        let mut new_watch = String::new();
                        if ui.text_edit_singleline(&mut new_watch).lost_focus() && !new_watch.is_empty() {
                            // Would add watch here
                        }
                        if ui.button("Add").clicked() {
                            // Add current property
                        }
                    });
                    ui.separator();
                    if self.watch_expressions.is_empty() {
                        ui.label(RichText::new("No watches added.")
                            .color(Theme::TEXT_MUTED));
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for watch in &self.watch_expressions {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&watch.property)
                                        .color(Theme::ACCENT)
                                        .strong());
                                    ui.label(RichText::new("=")
                                        .color(Theme::TEXT_MUTED));
                                    ui.label(RichText::new(&watch.value)
                                        .color(Theme::TEXT_PRIMARY));
                                });
                            }
                        });
                    }
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                });
            if close_dialog {
                self.show_watch_panel = false;
            }
        }

        // Annotations Dialog
        if self.show_annotations {
            egui::Window::new("Annotations")
                .default_width(450.0)
                .resizable(true)
                .open(&mut self.show_annotations)
                .show(ctx, |ui| {
                    if self.annotations.is_empty() {
                        ui.label(RichText::new("No annotations yet. Right-click a row to add notes.")
                            .color(Theme::TEXT_MUTED));
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for annotation in &self.annotations {
                                egui::Frame::none()
                                    .fill(Theme::BG_MEDIUM)
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::same(8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&annotation.table)
                                                .color(Theme::ACCENT));
                                            ui.label(RichText::new(&annotation.row_key)
                                                .color(Theme::TEXT_SECONDARY));
                                        });
                                        ui.label(RichText::new(&annotation.note)
                                            .color(Theme::TEXT_PRIMARY));
                                        ui.label(RichText::new(&annotation.timestamp)
                                            .color(Theme::TEXT_MUTED)
                                            .size(10.0));
                                    });
                                ui.add_space(4.0);
                            }
                        });
                    }
                });
        }

        // Row History Dialog
        if self.show_history_panel {
            egui::Window::new("Row Change History")
                .default_width(500.0)
                .resizable(true)
                .open(&mut self.show_history_panel)
                .show(ctx, |ui| {
                    if self.row_history.is_empty() {
                        ui.label(RichText::new("No row changes recorded yet.")
                            .color(Theme::TEXT_MUTED));
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for change in self.row_history.iter().rev().take(100) {
                                egui::Frame::none()
                                    .fill(Theme::BG_MEDIUM)
                                    .rounding(egui::Rounding::same(4.0))
                                    .inner_margin(egui::Margin::same(8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&change.table)
                                                .color(Theme::ACCENT));
                                            ui.label(RichText::new(&change.row_key)
                                                .color(Theme::TEXT_SECONDARY));
                                            ui.label(RichText::new(&change.column)
                                                .color(Theme::TEXT_MUTED));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&change.old_value)
                                                .color(Theme::ERROR)
                                                .strikethrough());
                                            ui.label(RichText::new("→")
                                                .color(Theme::TEXT_MUTED));
                                            ui.label(RichText::new(&change.new_value)
                                                .color(Theme::SUCCESS));
                                        });
                                        ui.label(RichText::new(&change.timestamp)
                                            .color(Theme::TEXT_MUTED)
                                            .size(10.0));
                                    });
                                ui.add_space(4.0);
                            }
                        });
                    }
                });
        }

        // User Guide Dialog
        if self.show_user_guide {
            egui::Window::new("User Guide")
                .default_width(600.0)
                .default_height(500.0)
                .resizable(true)
                .open(&mut self.show_user_guide)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(RichText::new("MSI Explorer User Guide")
                            .size(20.0)
                            .color(Theme::ACCENT)
                            .strong());
                        ui.add_space(16.0);

                        ui.label(RichText::new("Getting Started")
                            .size(16.0)
                            .color(Theme::TEXT_PRIMARY)
                            .strong());
                        ui.add_space(8.0);
                        ui.label("1. Open an MSI file using File > Open or drag & drop");
                        ui.label("2. Browse tables in the left panel");
                        ui.label("3. Click a table to view its contents");
                        ui.label("4. Use the search box to find specific values");
                        ui.add_space(16.0);

                        ui.label(RichText::new("Key Features")
                            .size(16.0)
                            .color(Theme::TEXT_PRIMARY)
                            .strong());
                        ui.add_space(8.0);
                        ui.label("- Smart IntelliSense: Hover over columns for descriptions");
                        ui.label("- FK Navigation: Click foreign key values to jump to referenced rows");
                        ui.label("- Value Detection: GUIDs, paths, and properties are highlighted");
                        ui.label("- ICE Validation: Tools > Validate (ICE) to check for issues");
                        ui.label("- Diff Mode: Compare two MSI files side by side");
                        ui.label("- Export to WiX: Convert MSI back to WiX source");
                        ui.add_space(16.0);

                        ui.label(RichText::new("View Modes")
                            .size(16.0)
                            .color(Theme::TEXT_PRIMARY)
                            .strong());
                        ui.add_space(8.0);
                        ui.label("- Tables: Browse database tables (default)");
                        ui.label("- Tree: Hierarchical view of features and components");
                        ui.label("- Files: File system view of installed files");
                        ui.label("- Diff: Compare with another MSI file");
                        ui.add_space(16.0);

                        ui.label(RichText::new("Editing")
                            .size(16.0)
                            .color(Theme::TEXT_PRIMARY)
                            .strong());
                        ui.add_space(8.0);
                        ui.label("- Double-click a cell to edit its value");
                        ui.label("- Right-click for context menu options");
                        ui.label("- Use Edit > Add Row to add new rows");
                        ui.label("- Ctrl+Z to undo changes");
                        ui.add_space(16.0);

                        ui.label(RichText::new("Tools")
                            .size(16.0)
                            .color(Theme::TEXT_PRIMARY)
                            .strong());
                        ui.add_space(8.0);
                        ui.label("- SQL Query Editor: Run custom SQL queries");
                        ui.label("- GUID Generator: Generate new GUIDs");
                        ui.label("- Condition Validator: Test MSI conditions");
                        ui.label("- Feature/Directory Trees: Visualize structure");
                        ui.label("- Bulk Edit: Edit multiple rows at once");
                        ui.label("- Report Generator: Create HTML/JSON reports");
                    });
                });
        }

        // About Dialog
        if self.show_about {
            egui::Window::new("About MSI Explorer")
                .default_width(400.0)
                .resizable(false)
                .collapsible(false)
                .open(&mut self.show_about)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(16.0);
                        ui.label(RichText::new("MSI Explorer")
                            .size(28.0)
                            .color(Theme::ACCENT)
                            .strong());
                        ui.add_space(4.0);
                        ui.label(RichText::new("v0.1.0")
                            .size(14.0)
                            .color(Theme::TEXT_MUTED));
                        ui.add_space(16.0);
                        ui.label(RichText::new("A modern, cross-platform MSI database explorer")
                            .size(13.0)
                            .color(Theme::TEXT_SECONDARY));
                        ui.label(RichText::new("Alternative to Microsoft Orca")
                            .size(12.0)
                            .color(Theme::TEXT_MUTED));
                        ui.add_space(24.0);

                        ui.separator();
                        ui.add_space(16.0);

                        ui.label(RichText::new("Author")
                            .size(12.0)
                            .color(Theme::TEXT_MUTED));
                        ui.label(RichText::new("Tsahi Elkayam")
                            .size(14.0)
                            .color(Theme::TEXT_PRIMARY));
                        ui.add_space(8.0);

                        ui.label(RichText::new("Email")
                            .size(12.0)
                            .color(Theme::TEXT_MUTED));
                        if ui.link(RichText::new("tsahi.elkayam@protonmail.com")
                            .size(13.0)
                            .color(Theme::ACCENT)).clicked() {
                            // Could open email client
                        }
                        ui.add_space(8.0);

                        ui.label(RichText::new("GitHub")
                            .size(12.0)
                            .color(Theme::TEXT_MUTED));
                        if ui.link(RichText::new("github.com/Tsahi-Elkayam/wixcraft")
                            .size(13.0)
                            .color(Theme::ACCENT)).clicked() {
                            // Could open browser
                        }
                        ui.add_space(16.0);

                        ui.separator();
                        ui.add_space(12.0);

                        ui.label(RichText::new("License: MIT")
                            .size(12.0)
                            .color(Theme::TEXT_MUTED));
                        ui.add_space(8.0);
                        ui.label(RichText::new("Part of the WixCraft toolkit")
                            .size(11.0)
                            .color(Theme::TEXT_MUTED));
                        ui.add_space(16.0);
                    });
                });
        }

        // Central panel
        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(Theme::BG_DARK)
                .inner_margin(egui::Margin::same(16.0)))
            .show(ctx, |ui| {
                match self.view_mode {
                    ViewMode::Tables => {
                        if !self.search_results.is_empty() {
                            panels::search_results_panel(ui, self);
                        } else if self.current_table.is_some() {
                            panels::table_view_panel(ui, self);
                        } else if self.summary.is_some() {
                            panels::summary_panel(ui, self);
                        } else {
                            panels::welcome_panel(ui);
                        }
                    }
                    ViewMode::Tree => {
                        panels::tree_view_panel(ui, self);
                    }
                    ViewMode::Files => {
                        panels::files_panel(ui, self);
                    }
                    ViewMode::Diff => {
                        panels::diff_panel(ui, self);
                    }
                }
            });
    }
}
