//! MSI Schema knowledge for smart tooltips and navigation

#![allow(dead_code)]

use std::collections::HashMap;

/// Column metadata for smart display
#[derive(Clone)]
pub struct ColumnInfo {
    pub description: &'static str,
    pub foreign_key: Option<(&'static str, &'static str)>, // (table, column)
    pub value_type: ValueType,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ValueType {
    Identifier,
    Guid,
    Path,
    Property,
    Condition,
    Version,
    Integer,
    Binary,
    Text,
    Formatted, // Can contain [Property] references
}

impl ValueType {
    pub fn description(&self) -> &'static str {
        match self {
            ValueType::Identifier => "Unique identifier",
            ValueType::Guid => "GUID in format {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}",
            ValueType::Path => "File system path",
            ValueType::Property => "Property name",
            ValueType::Condition => "Conditional expression",
            ValueType::Version => "Version number (e.g., 1.0.0.0)",
            ValueType::Integer => "Integer value",
            ValueType::Binary => "Binary data reference",
            ValueType::Text => "Text value",
            ValueType::Formatted => "Formatted string - can contain [PropertyName] references",
        }
    }
}

/// Get schema info for known MSI tables
pub fn get_schema() -> HashMap<&'static str, HashMap<&'static str, ColumnInfo>> {
    let mut schema = HashMap::new();

    // Component table
    let mut component = HashMap::new();
    component.insert("Component", ColumnInfo {
        description: "Primary key - unique identifier for this component",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    component.insert("ComponentId", ColumnInfo {
        description: "GUID that uniquely identifies the component across all products. Used for component sharing and repair.",
        foreign_key: None,
        value_type: ValueType::Guid,
    });
    component.insert("Directory_", ColumnInfo {
        description: "Reference to the directory where the component's files are installed",
        foreign_key: Some(("Directory", "Directory")),
        value_type: ValueType::Identifier,
    });
    component.insert("Attributes", ColumnInfo {
        description: "Component attributes (bit flags): 1=Local only, 2=Source only, 4=Optional, 16=Registry key path, 32=Shared DLL, 256=64-bit",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    component.insert("Condition", ColumnInfo {
        description: "Conditional expression - component only installed if this evaluates to true",
        foreign_key: None,
        value_type: ValueType::Condition,
    });
    component.insert("KeyPath", ColumnInfo {
        description: "File or registry key used to detect if component is installed. References File.File or Registry.Registry",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    schema.insert("Component", component);

    // Directory table
    let mut directory = HashMap::new();
    directory.insert("Directory", ColumnInfo {
        description: "Primary key - unique identifier for this directory",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    directory.insert("Directory_Parent", ColumnInfo {
        description: "Reference to parent directory. Empty for root directories like TARGETDIR",
        foreign_key: Some(("Directory", "Directory")),
        value_type: ValueType::Identifier,
    });
    directory.insert("DefaultDir", ColumnInfo {
        description: "Directory name. Format: 'ShortName|LongName' or just name. Use '.' for same as parent",
        foreign_key: None,
        value_type: ValueType::Path,
    });
    schema.insert("Directory", directory);

    // File table
    let mut file = HashMap::new();
    file.insert("File", ColumnInfo {
        description: "Primary key - unique identifier for this file",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    file.insert("Component_", ColumnInfo {
        description: "Reference to the component that controls this file's installation",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    file.insert("FileName", ColumnInfo {
        description: "File name. Format: 'ShortName|LongName' for 8.3 compatibility",
        foreign_key: None,
        value_type: ValueType::Path,
    });
    file.insert("FileSize", ColumnInfo {
        description: "Size of the file in bytes",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    file.insert("Version", ColumnInfo {
        description: "Version string for versioned files (DLLs, EXEs). Empty for non-versioned files",
        foreign_key: None,
        value_type: ValueType::Version,
    });
    file.insert("Language", ColumnInfo {
        description: "Language ID for the file (e.g., 1033 for English US)",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    file.insert("Attributes", ColumnInfo {
        description: "File attributes: 1=Read only, 2=Hidden, 4=System, 512=Compressed, 4096=Patch added",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    file.insert("Sequence", ColumnInfo {
        description: "Sequence number for file order in cabinet. Used during installation",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    schema.insert("File", file);

    // Feature table
    let mut feature = HashMap::new();
    feature.insert("Feature", ColumnInfo {
        description: "Primary key - unique identifier for this feature",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    feature.insert("Feature_Parent", ColumnInfo {
        description: "Reference to parent feature for hierarchical feature tree",
        foreign_key: Some(("Feature", "Feature")),
        value_type: ValueType::Identifier,
    });
    feature.insert("Title", ColumnInfo {
        description: "Short description shown in feature selection dialog",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    feature.insert("Description", ColumnInfo {
        description: "Longer description shown when feature is selected",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    feature.insert("Display", ColumnInfo {
        description: "Display order. 0=hidden, odd=collapsed, even=expanded",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    feature.insert("Level", ColumnInfo {
        description: "Installation level. 0=disabled, 1=always install. Higher = optional",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    feature.insert("Directory_", ColumnInfo {
        description: "Default directory for the feature (can be changed by user)",
        foreign_key: Some(("Directory", "Directory")),
        value_type: ValueType::Identifier,
    });
    feature.insert("Attributes", ColumnInfo {
        description: "Feature attributes: 0=Favor local, 1=Favor source, 2=Follow parent, 8=Disallow absent",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    schema.insert("Feature", feature);

    // FeatureComponents table
    let mut feature_comp = HashMap::new();
    feature_comp.insert("Feature_", ColumnInfo {
        description: "Reference to the feature",
        foreign_key: Some(("Feature", "Feature")),
        value_type: ValueType::Identifier,
    });
    feature_comp.insert("Component_", ColumnInfo {
        description: "Reference to the component included in this feature",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    schema.insert("FeatureComponents", feature_comp);

    // Property table
    let mut property = HashMap::new();
    property.insert("Property", ColumnInfo {
        description: "Property name. PUBLIC properties are UPPERCASE. Private are mixed case",
        foreign_key: None,
        value_type: ValueType::Property,
    });
    property.insert("Value", ColumnInfo {
        description: "Property value. Can reference other properties with [PropertyName] syntax",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    schema.insert("Property", property);

    // Registry table
    let mut registry = HashMap::new();
    registry.insert("Registry", ColumnInfo {
        description: "Primary key - unique identifier for this registry entry",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    registry.insert("Root", ColumnInfo {
        description: "Registry root: -1=User/Machine based on ALLUSERS, 0=HKCR, 1=HKCU, 2=HKLM, 3=HKU",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    registry.insert("Key", ColumnInfo {
        description: "Registry key path (without root). Can contain [Property] references",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    registry.insert("Name", ColumnInfo {
        description: "Registry value name. Empty or null for default value. '+' prefix = create key only",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    registry.insert("Value", ColumnInfo {
        description: "Registry value. Prefix: #x=hex dword, #%=expandsz, [~]=null-delimited multi-sz",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    registry.insert("Component_", ColumnInfo {
        description: "Reference to the component that controls this registry entry",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    schema.insert("Registry", registry);

    // CustomAction table
    let mut custom_action = HashMap::new();
    custom_action.insert("Action", ColumnInfo {
        description: "Primary key - name of the custom action. Used in sequence tables",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    custom_action.insert("Type", ColumnInfo {
        description: "Custom action type (bit field). Low bits=source type, high bits=options",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    custom_action.insert("Source", ColumnInfo {
        description: "Source of the action: Binary table key, Directory, Property, or file path",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    custom_action.insert("Target", ColumnInfo {
        description: "Target: DLL entry point, EXE command line, VBScript/JScript code, or property value",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    custom_action.insert("ExtendedType", ColumnInfo {
        description: "Extended type flags for additional custom action options",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    schema.insert("CustomAction", custom_action);

    // InstallExecuteSequence
    let mut install_exec = HashMap::new();
    install_exec.insert("Action", ColumnInfo {
        description: "Action name - either a built-in action or reference to CustomAction table",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    install_exec.insert("Condition", ColumnInfo {
        description: "Condition for running this action. Empty means always run",
        foreign_key: None,
        value_type: ValueType::Condition,
    });
    install_exec.insert("Sequence", ColumnInfo {
        description: "Sequence number determining order of execution. Higher = later",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    schema.insert("InstallExecuteSequence", install_exec.clone());
    schema.insert("InstallUISequence", install_exec.clone());
    schema.insert("AdminExecuteSequence", install_exec.clone());
    schema.insert("AdminUISequence", install_exec.clone());
    schema.insert("AdvtExecuteSequence", install_exec);

    // LaunchCondition
    let mut launch = HashMap::new();
    launch.insert("Condition", ColumnInfo {
        description: "Condition that must be true for installation to proceed",
        foreign_key: None,
        value_type: ValueType::Condition,
    });
    launch.insert("Description", ColumnInfo {
        description: "Error message shown to user if condition is false",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    schema.insert("LaunchCondition", launch);

    // Shortcut table
    let mut shortcut = HashMap::new();
    shortcut.insert("Shortcut", ColumnInfo {
        description: "Primary key - unique identifier for this shortcut",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    shortcut.insert("Directory_", ColumnInfo {
        description: "Directory where shortcut is created (e.g., DesktopFolder, ProgramMenuFolder)",
        foreign_key: Some(("Directory", "Directory")),
        value_type: ValueType::Identifier,
    });
    shortcut.insert("Name", ColumnInfo {
        description: "Shortcut file name (without .lnk). Format: 'ShortName|LongName'",
        foreign_key: None,
        value_type: ValueType::Path,
    });
    shortcut.insert("Component_", ColumnInfo {
        description: "Reference to the component that controls this shortcut",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    shortcut.insert("Target", ColumnInfo {
        description: "Shortcut target: Feature name or [Property] with full path",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    shortcut.insert("Arguments", ColumnInfo {
        description: "Command line arguments for the shortcut target",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    shortcut.insert("Description", ColumnInfo {
        description: "Shortcut description shown as tooltip",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    shortcut.insert("Hotkey", ColumnInfo {
        description: "Keyboard shortcut (hotkey) to activate this shortcut",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    shortcut.insert("Icon_", ColumnInfo {
        description: "Reference to Icon table for shortcut icon",
        foreign_key: Some(("Icon", "Name")),
        value_type: ValueType::Identifier,
    });
    shortcut.insert("IconIndex", ColumnInfo {
        description: "Index of icon in the icon file (0-based)",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    shortcut.insert("ShowCmd", ColumnInfo {
        description: "Window state: 1=Normal, 3=Maximized, 7=Minimized",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    shortcut.insert("WkDir", ColumnInfo {
        description: "Working directory for the shortcut target",
        foreign_key: Some(("Directory", "Directory")),
        value_type: ValueType::Identifier,
    });
    schema.insert("Shortcut", shortcut);

    // ServiceInstall table
    let mut service = HashMap::new();
    service.insert("ServiceInstall", ColumnInfo {
        description: "Primary key - unique identifier for this service",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    service.insert("Name", ColumnInfo {
        description: "Internal service name used by Windows SCM",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    service.insert("DisplayName", ColumnInfo {
        description: "Display name shown in Services management console",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    service.insert("ServiceType", ColumnInfo {
        description: "Service type: 1=Kernel driver, 2=File system driver, 16=Own process, 32=Share process",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    service.insert("StartType", ColumnInfo {
        description: "Start type: 0=Boot, 1=System, 2=Automatic, 3=Manual, 4=Disabled",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    service.insert("ErrorControl", ColumnInfo {
        description: "Error control: 0=Ignore, 1=Normal, 2=Severe, 3=Critical",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    service.insert("Component_", ColumnInfo {
        description: "Reference to the component that installs this service",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    schema.insert("ServiceInstall", service);

    // Media table
    let mut media = HashMap::new();
    media.insert("DiskId", ColumnInfo {
        description: "Primary key - disk/cabinet identifier (1-based)",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    media.insert("LastSequence", ColumnInfo {
        description: "Last file sequence number on this disk",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    media.insert("DiskPrompt", ColumnInfo {
        description: "Prompt shown when disk is required",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    media.insert("Cabinet", ColumnInfo {
        description: "Cabinet file name. #name = embedded, name = external",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    media.insert("VolumeLabel", ColumnInfo {
        description: "Volume label of the disk (for verification)",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    media.insert("Source", ColumnInfo {
        description: "Path to cabinet or source files (relative to SourceDir)",
        foreign_key: None,
        value_type: ValueType::Path,
    });
    schema.insert("Media", media);

    // Binary table
    let mut binary = HashMap::new();
    binary.insert("Name", ColumnInfo {
        description: "Primary key - unique identifier for this binary data",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    binary.insert("Data", ColumnInfo {
        description: "Binary data stored in the MSI (icons, DLLs, scripts, etc.)",
        foreign_key: None,
        value_type: ValueType::Binary,
    });
    schema.insert("Binary", binary);

    // Icon table
    let mut icon = HashMap::new();
    icon.insert("Name", ColumnInfo {
        description: "Primary key - unique identifier for this icon",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    icon.insert("Data", ColumnInfo {
        description: "Icon file data (.ico format)",
        foreign_key: None,
        value_type: ValueType::Binary,
    });
    schema.insert("Icon", icon);

    // CreateFolder table
    let mut create_folder = HashMap::new();
    create_folder.insert("Directory_", ColumnInfo {
        description: "Reference to directory to create",
        foreign_key: Some(("Directory", "Directory")),
        value_type: ValueType::Identifier,
    });
    create_folder.insert("Component_", ColumnInfo {
        description: "Reference to component that controls folder creation",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    schema.insert("CreateFolder", create_folder);

    // RemoveFile table
    let mut remove_file = HashMap::new();
    remove_file.insert("FileKey", ColumnInfo {
        description: "Primary key - unique identifier for this remove action",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    remove_file.insert("Component_", ColumnInfo {
        description: "Reference to component that controls file removal",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    remove_file.insert("FileName", ColumnInfo {
        description: "File name or wildcard pattern to remove",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    remove_file.insert("DirProperty", ColumnInfo {
        description: "Directory or property containing path",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    remove_file.insert("InstallMode", ColumnInfo {
        description: "When to remove: 1=Install, 2=Uninstall, 3=Both",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    schema.insert("RemoveFile", remove_file);

    // Environment table
    let mut environment = HashMap::new();
    environment.insert("Environment", ColumnInfo {
        description: "Primary key - unique identifier for this environment variable",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    environment.insert("Name", ColumnInfo {
        description: "Environment variable name (prefix: =set, +append, -remove, !system, *both)",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    environment.insert("Value", ColumnInfo {
        description: "Value to set. Use [~] for separator in PATH-style variables",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    environment.insert("Component_", ColumnInfo {
        description: "Reference to component that controls this variable",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    schema.insert("Environment", environment);

    // ServiceControl table
    let mut service_control = HashMap::new();
    service_control.insert("ServiceControl", ColumnInfo {
        description: "Primary key - unique identifier for this service control",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    service_control.insert("Name", ColumnInfo {
        description: "Service name to control",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    service_control.insert("Event", ColumnInfo {
        description: "Events: 1=Start install, 2=Stop install, 8=Delete install, 16=Start uninstall, etc.",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    service_control.insert("Arguments", ColumnInfo {
        description: "Arguments to pass when starting service",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    service_control.insert("Wait", ColumnInfo {
        description: "Wait for service action: 0=No, 1=Yes",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    service_control.insert("Component_", ColumnInfo {
        description: "Reference to component that controls this service",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    schema.insert("ServiceControl", service_control);

    // Condition table
    let mut condition = HashMap::new();
    condition.insert("Feature_", ColumnInfo {
        description: "Reference to feature this condition applies to",
        foreign_key: Some(("Feature", "Feature")),
        value_type: ValueType::Identifier,
    });
    condition.insert("Level", ColumnInfo {
        description: "New feature level if condition is true (0 disables)",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    condition.insert("Condition", ColumnInfo {
        description: "Conditional expression to evaluate",
        foreign_key: None,
        value_type: ValueType::Condition,
    });
    schema.insert("Condition", condition);

    // MsiFileHash table
    let mut file_hash = HashMap::new();
    file_hash.insert("File_", ColumnInfo {
        description: "Reference to file in File table",
        foreign_key: Some(("File", "File")),
        value_type: ValueType::Identifier,
    });
    file_hash.insert("Options", ColumnInfo {
        description: "Hash algorithm options (must be 0)",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    file_hash.insert("HashPart1", ColumnInfo {
        description: "First 32 bits of MD5 hash",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    file_hash.insert("HashPart2", ColumnInfo {
        description: "Second 32 bits of MD5 hash",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    file_hash.insert("HashPart3", ColumnInfo {
        description: "Third 32 bits of MD5 hash",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    file_hash.insert("HashPart4", ColumnInfo {
        description: "Fourth 32 bits of MD5 hash",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    schema.insert("MsiFileHash", file_hash);

    // AppSearch table
    let mut app_search = HashMap::new();
    app_search.insert("Property", ColumnInfo {
        description: "Property to set with search result",
        foreign_key: None,
        value_type: ValueType::Property,
    });
    app_search.insert("Signature_", ColumnInfo {
        description: "Reference to signature defining what to search for",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    schema.insert("AppSearch", app_search);

    // RegLocator table
    let mut reg_locator = HashMap::new();
    reg_locator.insert("Signature_", ColumnInfo {
        description: "Primary key - signature for this search",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    reg_locator.insert("Root", ColumnInfo {
        description: "Registry root: 0=HKCR, 1=HKCU, 2=HKLM, 3=HKU",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    reg_locator.insert("Key", ColumnInfo {
        description: "Registry key path to search",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    reg_locator.insert("Name", ColumnInfo {
        description: "Registry value name (empty for default)",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    reg_locator.insert("Type", ColumnInfo {
        description: "Type: 0=Directory, 1=File, 2=Raw value",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    schema.insert("RegLocator", reg_locator);

    // CompLocator table
    let mut comp_locator = HashMap::new();
    comp_locator.insert("Signature_", ColumnInfo {
        description: "Primary key - signature for this search",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    comp_locator.insert("ComponentId", ColumnInfo {
        description: "GUID of component to locate",
        foreign_key: None,
        value_type: ValueType::Guid,
    });
    comp_locator.insert("Type", ColumnInfo {
        description: "Type: 0=Directory, 1=File name",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    schema.insert("CompLocator", comp_locator);

    // DrLocator table
    let mut dr_locator = HashMap::new();
    dr_locator.insert("Signature_", ColumnInfo {
        description: "Primary key - signature for this directory search",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    dr_locator.insert("Parent", ColumnInfo {
        description: "Parent signature or property for search root",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    dr_locator.insert("Path", ColumnInfo {
        description: "Directory path to search",
        foreign_key: None,
        value_type: ValueType::Path,
    });
    dr_locator.insert("Depth", ColumnInfo {
        description: "Depth to search subdirectories (0=this dir only)",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    schema.insert("DrLocator", dr_locator);

    // IniFile table
    let mut ini_file = HashMap::new();
    ini_file.insert("IniFile", ColumnInfo {
        description: "Primary key - unique identifier for this INI entry",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    ini_file.insert("FileName", ColumnInfo {
        description: "INI file name",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    ini_file.insert("DirProperty", ColumnInfo {
        description: "Directory or property containing INI file path",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    ini_file.insert("Section", ColumnInfo {
        description: "INI file section name",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    ini_file.insert("Key", ColumnInfo {
        description: "INI file key name",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    ini_file.insert("Value", ColumnInfo {
        description: "Value to write",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    ini_file.insert("Action", ColumnInfo {
        description: "Action: 0=Create, 1=Create line, 2=Add tag, 3=Remove tag",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    ini_file.insert("Component_", ColumnInfo {
        description: "Reference to controlling component",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    schema.insert("IniFile", ini_file);

    // TypeLib table
    let mut type_lib = HashMap::new();
    type_lib.insert("LibID", ColumnInfo {
        description: "GUID of the type library",
        foreign_key: None,
        value_type: ValueType::Guid,
    });
    type_lib.insert("Language", ColumnInfo {
        description: "Language ID of the type library",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    type_lib.insert("Component_", ColumnInfo {
        description: "Reference to component containing the type library",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    type_lib.insert("Version", ColumnInfo {
        description: "Version number (high word = major, low word = minor)",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    type_lib.insert("Description", ColumnInfo {
        description: "Description displayed in registry",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    type_lib.insert("Directory_", ColumnInfo {
        description: "Help file directory",
        foreign_key: Some(("Directory", "Directory")),
        value_type: ValueType::Identifier,
    });
    type_lib.insert("Feature_", ColumnInfo {
        description: "Reference to feature providing the type library",
        foreign_key: Some(("Feature", "Feature")),
        value_type: ValueType::Identifier,
    });
    schema.insert("TypeLib", type_lib);

    // Class table (COM registration)
    let mut class = HashMap::new();
    class.insert("CLSID", ColumnInfo {
        description: "Class ID (GUID) of the COM class",
        foreign_key: None,
        value_type: ValueType::Guid,
    });
    class.insert("Context", ColumnInfo {
        description: "Server context: LocalServer, LocalServer32, InprocServer, InprocServer32",
        foreign_key: None,
        value_type: ValueType::Identifier,
    });
    class.insert("Component_", ColumnInfo {
        description: "Reference to component containing the COM class",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    class.insert("ProgId_Default", ColumnInfo {
        description: "Default ProgId for this class",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    class.insert("Description", ColumnInfo {
        description: "Description of the COM class",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    class.insert("Feature_", ColumnInfo {
        description: "Reference to feature providing this class",
        foreign_key: Some(("Feature", "Feature")),
        value_type: ValueType::Identifier,
    });
    schema.insert("Class", class);

    // Extension table
    let mut extension = HashMap::new();
    extension.insert("Extension", ColumnInfo {
        description: "File extension (without dot)",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    extension.insert("Component_", ColumnInfo {
        description: "Reference to component registering this extension",
        foreign_key: Some(("Component", "Component")),
        value_type: ValueType::Identifier,
    });
    extension.insert("ProgId_", ColumnInfo {
        description: "ProgId to associate with extension",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    extension.insert("MIME_", ColumnInfo {
        description: "MIME type for this extension",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    extension.insert("Feature_", ColumnInfo {
        description: "Reference to feature providing this extension",
        foreign_key: Some(("Feature", "Feature")),
        value_type: ValueType::Identifier,
    });
    schema.insert("Extension", extension);

    // Verb table
    let mut verb = HashMap::new();
    verb.insert("Extension_", ColumnInfo {
        description: "Reference to extension this verb applies to",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    verb.insert("Verb", ColumnInfo {
        description: "Verb name (e.g., open, edit, print)",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    verb.insert("Sequence", ColumnInfo {
        description: "Display order (lower = higher priority)",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    verb.insert("Command", ColumnInfo {
        description: "Menu text for this verb",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    verb.insert("Argument", ColumnInfo {
        description: "Command line arguments (use %1 for document)",
        foreign_key: None,
        value_type: ValueType::Formatted,
    });
    schema.insert("Verb", verb);

    // Upgrade table
    let mut upgrade = HashMap::new();
    upgrade.insert("UpgradeCode", ColumnInfo {
        description: "GUID identifying the upgrade family - shared by all related products",
        foreign_key: None,
        value_type: ValueType::Guid,
    });
    upgrade.insert("VersionMin", ColumnInfo {
        description: "Minimum version to detect (inclusive unless Attributes excludes it)",
        foreign_key: None,
        value_type: ValueType::Version,
    });
    upgrade.insert("VersionMax", ColumnInfo {
        description: "Maximum version to detect (inclusive unless Attributes excludes it)",
        foreign_key: None,
        value_type: ValueType::Version,
    });
    upgrade.insert("Language", ColumnInfo {
        description: "Comma-separated language IDs to detect (empty = all languages)",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    upgrade.insert("Attributes", ColumnInfo {
        description: "Attributes: 256=Migrate features, 512=Only detect, 1024=Exclude min version",
        foreign_key: None,
        value_type: ValueType::Integer,
    });
    upgrade.insert("Remove", ColumnInfo {
        description: "Comma-separated features to remove from detected products",
        foreign_key: None,
        value_type: ValueType::Text,
    });
    upgrade.insert("ActionProperty", ColumnInfo {
        description: "Property set to list of detected product codes",
        foreign_key: None,
        value_type: ValueType::Property,
    });
    schema.insert("Upgrade", upgrade);

    schema
}

/// Well-known property descriptions
pub fn get_property_description(name: &str) -> Option<&'static str> {
    match name {
        "ProductCode" => Some("GUID uniquely identifying this product version"),
        "ProductName" => Some("Display name of the product"),
        "ProductVersion" => Some("Product version in format major.minor.build"),
        "Manufacturer" => Some("Company or individual who created the product"),
        "UpgradeCode" => Some("GUID identifying the upgrade family - shared across versions"),
        "TARGETDIR" => Some("Root installation directory (defaults to ROOTDRIVE)"),
        "ProgramFilesFolder" => Some("Program Files directory (32-bit or 64-bit based on package)"),
        "ProgramFiles64Folder" => Some("64-bit Program Files directory"),
        "CommonFilesFolder" => Some("Common Files directory"),
        "SystemFolder" => Some("Windows\\System32 directory"),
        "WindowsFolder" => Some("Windows directory"),
        "TempFolder" => Some("User's temporary directory"),
        "DesktopFolder" => Some("User's desktop directory"),
        "ProgramMenuFolder" => Some("Start Menu\\Programs directory"),
        "StartupFolder" => Some("Start Menu\\Programs\\Startup directory"),
        "ALLUSERS" => Some("1 = per-machine install, empty = per-user install"),
        "REINSTALLMODE" => Some("Reinstall mode flags: o=older, e=equal, d=different, s=same version"),
        "INSTALLLEVEL" => Some("Installation level (1-32767). Features with level <= this are installed"),
        "REBOOT" => Some("Controls reboot behavior: Force, Suppress, ReallySuppress"),
        "ARPNOREMOVE" => Some("1 = Hide Remove button in Add/Remove Programs"),
        "ARPNOMODIFY" => Some("1 = Hide Modify button in Add/Remove Programs"),
        "ARPNOREPAIR" => Some("1 = Hide Repair option in Add/Remove Programs"),
        "ARPSYSTEMCOMPONENT" => Some("1 = Hide from Add/Remove Programs entirely"),
        _ => None,
    }
}

/// Standard directory descriptions
pub fn get_directory_description(name: &str) -> Option<&'static str> {
    match name {
        "TARGETDIR" => Some("Root destination directory - usually C:\\"),
        "SourceDir" => Some("Root source directory - where MSI is located"),
        "ProgramFilesFolder" => Some("C:\\Program Files\\ (or Program Files (x86) on 64-bit)"),
        "ProgramFiles64Folder" => Some("C:\\Program Files\\ on 64-bit Windows"),
        "CommonFilesFolder" => Some("C:\\Program Files\\Common Files\\"),
        "CommonFiles64Folder" => Some("C:\\Program Files\\Common Files\\ (64-bit)"),
        "SystemFolder" => Some("C:\\Windows\\System32\\"),
        "System64Folder" => Some("C:\\Windows\\System32\\ (native 64-bit)"),
        "WindowsFolder" => Some("C:\\Windows\\"),
        "TempFolder" => Some("User's temporary folder"),
        "LocalAppDataFolder" => Some("C:\\Users\\<user>\\AppData\\Local\\"),
        "AppDataFolder" => Some("C:\\Users\\<user>\\AppData\\Roaming\\"),
        "PersonalFolder" => Some("C:\\Users\\<user>\\Documents\\"),
        "DesktopFolder" => Some("User's desktop"),
        "StartMenuFolder" => Some("User's Start Menu"),
        "ProgramMenuFolder" => Some("Start Menu\\Programs"),
        "StartupFolder" => Some("Start Menu\\Programs\\Startup"),
        "SendToFolder" => Some("SendTo folder"),
        "FavoritesFolder" => Some("User's Favorites"),
        "FontsFolder" => Some("C:\\Windows\\Fonts\\"),
        "AdminToolsFolder" => Some("Administrative Tools folder"),
        "NetHoodFolder" => Some("Network Neighborhood folder"),
        "PrintHoodFolder" => Some("Printers folder"),
        "RecentFolder" => Some("Recent documents folder"),
        "TemplateFolder" => Some("Templates folder"),
        _ => None,
    }
}

/// Detect value type from content
pub fn detect_value_type(value: &str) -> Option<DetectedValue> {
    // GUID pattern
    if value.starts_with('{') && value.ends_with('}') && value.len() == 38 {
        return Some(DetectedValue::Guid);
    }

    // Property reference [PropertyName]
    if value.contains('[') && value.contains(']') {
        return Some(DetectedValue::PropertyRef);
    }

    // Path-like values
    if value.contains('\\') || value.contains('/') {
        return Some(DetectedValue::Path);
    }

    // Version-like
    if value.chars().all(|c| c.is_ascii_digit() || c == '.') && value.contains('.') {
        let parts: Vec<&str> = value.split('.').collect();
        if parts.len() >= 2 && parts.len() <= 4 {
            return Some(DetectedValue::Version);
        }
    }

    None
}

#[derive(Clone, Copy)]
pub enum DetectedValue {
    Guid,
    PropertyRef,
    Path,
    Version,
}

impl DetectedValue {
    pub fn icon(&self) -> &'static str {
        match self {
            DetectedValue::Guid => "◈",
            DetectedValue::PropertyRef => "◇",
            DetectedValue::Path => "◨",
            DetectedValue::Version => "◎",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DetectedValue::Guid => "GUID - Globally Unique Identifier",
            DetectedValue::PropertyRef => "Contains property references [PropertyName]",
            DetectedValue::Path => "File system path",
            DetectedValue::Version => "Version number",
        }
    }
}

/// Decode an attribute value for a specific table/column
pub fn decode_attributes(table: &str, column: &str, value: i32) -> Option<String> {
    match (table, column) {
        ("Component", "Attributes") => Some(get_component_attr_desc(value)),
        ("File", "Attributes") => Some(get_file_attr_desc(value)),
        ("CustomAction", "Type") => Some(get_custom_action_type_desc(value)),
        ("Feature", "Attributes") => Some(get_feature_attr_desc(value)),
        ("ServiceInstall", "ServiceType") => Some(get_service_type_desc(value)),
        ("ServiceInstall", "StartType") => Some(get_start_type_desc(value)),
        ("ServiceInstall", "ErrorControl") => Some(get_error_control_desc(value)),
        ("Registry", "Root") => Some(get_registry_root_desc(value)),
        ("Shortcut", "ShowCmd") => Some(get_show_cmd_desc(value)),
        _ => None,
    }
}

/// Get description for Registry Root
pub fn get_registry_root_desc(root: i32) -> String {
    match root {
        -1 => "HKCU or HKLM (based on ALLUSERS)".to_string(),
        0 => "HKEY_CLASSES_ROOT".to_string(),
        1 => "HKEY_CURRENT_USER".to_string(),
        2 => "HKEY_LOCAL_MACHINE".to_string(),
        3 => "HKEY_USERS".to_string(),
        _ => format!("Unknown ({})", root),
    }
}

/// Get description for Feature attributes
pub fn get_feature_attr_desc(attr: i32) -> String {
    let mut flags = Vec::new();
    if attr & 0x01 != 0 { flags.push("Favor source"); }
    if attr & 0x02 != 0 { flags.push("Follow parent"); }
    if attr & 0x04 != 0 { flags.push("Favor advertise"); }
    if attr & 0x08 != 0 { flags.push("Disallow absent"); }
    if attr & 0x10 != 0 { flags.push("Disallow advertise"); }
    if attr & 0x20 != 0 { flags.push("UI disallow absent"); }
    if attr & 0x40 != 0 { flags.push("No unsupported advertise"); }

    if flags.is_empty() {
        "Favor local (default)".to_string()
    } else {
        flags.join(", ")
    }
}

/// Get description for Service Type
pub fn get_service_type_desc(svc_type: i32) -> String {
    match svc_type & 0xFF {
        0x01 => "Kernel driver".to_string(),
        0x02 => "File system driver".to_string(),
        0x10 => "Own process".to_string(),
        0x20 => "Share process".to_string(),
        0x110 => "Own process (interactive)".to_string(),
        0x120 => "Share process (interactive)".to_string(),
        _ => format!("Unknown ({})", svc_type),
    }
}

/// Get description for Start Type
pub fn get_start_type_desc(start: i32) -> String {
    match start {
        0 => "Boot (loaded by kernel loader)".to_string(),
        1 => "System (loaded by I/O subsystem)".to_string(),
        2 => "Automatic".to_string(),
        3 => "Manual".to_string(),
        4 => "Disabled".to_string(),
        _ => format!("Unknown ({})", start),
    }
}

/// Get description for Error Control
pub fn get_error_control_desc(err: i32) -> String {
    match err {
        0 => "Ignore errors".to_string(),
        1 => "Normal - log error".to_string(),
        2 => "Severe - switch to LastKnownGood".to_string(),
        3 => "Critical - fail boot".to_string(),
        _ => format!("Unknown ({})", err),
    }
}

/// Get description for ShowCmd (Shortcut)
pub fn get_show_cmd_desc(cmd: i32) -> String {
    match cmd {
        1 => "Normal window".to_string(),
        3 => "Maximized".to_string(),
        7 => "Minimized".to_string(),
        _ => format!("Unknown ({})", cmd),
    }
}

/// Get description for Custom Action type
pub fn get_custom_action_type_desc(type_val: i32) -> String {
    let source = match type_val & 0x3F {
        1 => "DLL in Binary table",
        2 => "EXE in Binary table",
        5 => "JScript in Binary table",
        6 => "VBScript in Binary table",
        17 => "DLL from installed file",
        18 => "EXE from installed file",
        19 => "Display error and abort",
        21 => "JScript from installed file",
        22 => "VBScript from installed file",
        34 => "EXE with working dir",
        35 => "Set directory",
        37 => "JScript inline",
        38 => "VBScript inline",
        50 => "EXE with command line",
        51 => "Set property",
        53 => "JScript from property",
        54 => "VBScript from property",
        _ => "Unknown type",
    };

    let mut flags = Vec::new();
    if type_val & 0x40 != 0 { flags.push("Continue on error"); }
    if type_val & 0x80 != 0 { flags.push("Async"); }
    if type_val & 0x100 != 0 { flags.push("First sequence"); }
    if type_val & 0x200 != 0 { flags.push("Once per process"); }
    if type_val & 0x400 != 0 { flags.push("Client repeat"); }
    if type_val & 0x800 != 0 { flags.push("In script"); }
    if type_val & 0x1000 != 0 { flags.push("Rollback"); }
    if type_val & 0x2000 != 0 { flags.push("Commit"); }

    if flags.is_empty() {
        source.to_string()
    } else {
        format!("{} ({})", source, flags.join(", "))
    }
}

/// Get description for Component attributes
pub fn get_component_attr_desc(attr: i32) -> String {
    let mut flags = Vec::new();
    if attr & 0x01 != 0 { flags.push("Local only"); }
    if attr & 0x02 != 0 { flags.push("Source only"); }
    if attr & 0x04 != 0 { flags.push("Optional"); }
    if attr & 0x10 != 0 { flags.push("Registry key path"); }
    if attr & 0x20 != 0 { flags.push("Shared DLL ref count"); }
    if attr & 0x40 != 0 { flags.push("Permanent"); }
    if attr & 0x80 != 0 { flags.push("ODBC data source"); }
    if attr & 0x100 != 0 { flags.push("Transitive"); }
    if attr & 0x200 != 0 { flags.push("Never overwrite"); }
    if attr & 0x1000 != 0 { flags.push("64-bit"); }
    if attr & 0x2000 != 0 { flags.push("Disable registry reflection"); }
    if attr & 0x4000 != 0 { flags.push("Uninstall on supersedence"); }
    if attr & 0x8000 != 0 { flags.push("Shared among packages"); }

    if flags.is_empty() {
        "None".to_string()
    } else {
        flags.join(", ")
    }
}

/// Get description for File attributes
pub fn get_file_attr_desc(attr: i32) -> String {
    let mut flags = Vec::new();
    if attr & 0x01 != 0 { flags.push("Read only"); }
    if attr & 0x02 != 0 { flags.push("Hidden"); }
    if attr & 0x04 != 0 { flags.push("System"); }
    if attr & 0x100 != 0 { flags.push("Vital"); }
    if attr & 0x200 != 0 { flags.push("Checksum"); }
    if attr & 0x400 != 0 { flags.push("Patch added"); }
    if attr & 0x800 != 0 { flags.push("Non-compressed"); }
    if attr & 0x1000 != 0 { flags.push("Compressed"); }

    if flags.is_empty() {
        "None".to_string()
    } else {
        flags.join(", ")
    }
}
