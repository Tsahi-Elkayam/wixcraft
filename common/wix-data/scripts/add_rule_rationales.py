#!/usr/bin/env python3
"""Add rationale and fix_suggestion to all rules in the database."""

import sqlite3
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"

# Rule rationales and fix suggestions grouped by category
RULE_DATA = {
    # Bundle rules
    "BNDL001": {
        "rationale": "The UpgradeCode uniquely identifies the bundle family. Without it, Burn cannot detect existing installations and perform upgrades, leading to multiple side-by-side installations.",
        "fix_suggestion": "Add UpgradeCode attribute to the Bundle element: <Bundle UpgradeCode=\"{GUID}\">",
        "auto_fixable": 0
    },
    "BNDL002": {
        "rationale": "A bundle without packages serves no purpose. The Chain element must contain at least one package (MsiPackage, ExePackage, etc.) to install.",
        "fix_suggestion": "Add at least one package element inside the Chain element: <Chain><MsiPackage .../></Chain>",
        "auto_fixable": 0
    },
    "BNDL003": {
        "rationale": "DetectCondition tells Burn how to check if a package is already installed. Without it, Burn may reinstall packages unnecessarily or fail to detect upgrades.",
        "fix_suggestion": "Add DetectCondition attribute with a registry or file check: DetectCondition=\"EXISTS(path) OR REGISTRY_VALUE(...)\"",
        "auto_fixable": 0
    },
    "BNDL004": {
        "rationale": "Burn has built-in variables like WixBundleVersion, WixBundleName, etc. Defining variables with the same names causes conflicts and unexpected behavior.",
        "fix_suggestion": "Rename the variable to avoid conflict with built-in Burn variables. Use a unique prefix for your variables.",
        "auto_fixable": 0
    },
    "BNDL005": {
        "rationale": "When Vital=no, package installation failures are ignored. This may leave the system in a partially installed state without user awareness.",
        "fix_suggestion": "Set Vital=\"yes\" (default) or ensure non-vital packages are truly optional and their failure is acceptable.",
        "auto_fixable": 0
    },
    "BNDL006": {
        "rationale": "Bundle payloads must exist at build time. Missing files cause build failures or runtime installation errors.",
        "fix_suggestion": "Ensure the payload file exists at the specified SourceFile path, or update the path to the correct location.",
        "auto_fixable": 0
    },
    "BNDL007": {
        "rationale": "Remote payloads downloaded without hash verification can be tampered with. Hash ensures payload integrity and protects against man-in-the-middle attacks.",
        "fix_suggestion": "Add Hash attribute with SHA-512 hash of the payload: Hash=\"base64-encoded-hash\"",
        "auto_fixable": 0
    },
    "BNDL008": {
        "rationale": "RollbackBoundary controls which packages roll back on failure. Incorrect placement can cause unintended packages to be uninstalled on failure.",
        "fix_suggestion": "Review RollbackBoundary placement. Place before package groups that should roll back together.",
        "auto_fixable": 0
    },
    "BNDL009": {
        "rationale": "ExePackage without RepairCommand cannot participate in repair scenarios. Users cannot repair such packages without full reinstall.",
        "fix_suggestion": "Add RepairCommand attribute with appropriate repair arguments for the executable.",
        "auto_fixable": 0
    },
    "BNDL010": {
        "rationale": "MSI packages in bundles run silently. Without proper silent install support, UI may appear or installation may hang.",
        "fix_suggestion": "Ensure the MSI supports silent installation. Burn passes MSIFASTINSTALL and ARPSYSTEMCOMPONENT properties automatically.",
        "auto_fixable": 0
    },
    "BNDL011": {
        "rationale": "MSU packages contain Windows updates. Including the KB number helps users and administrators identify what update is being installed.",
        "fix_suggestion": "Add the KB number to the package description or Id: Id=\"KB1234567\"",
        "auto_fixable": 0
    },
    "BNDL012": {
        "rationale": "Circular references in PackageGroup cause infinite loops during resolution. Burn cannot determine installation order.",
        "fix_suggestion": "Review PackageGroupRef elements and remove the circular dependency. Restructure package groups if needed.",
        "auto_fixable": 0
    },
    "BUNDLE001": {
        "rationale": "MSI elements like Component, Directory, File belong in MSI packages, not directly in bundles. Bundles orchestrate packages but don't contain MSI tables.",
        "fix_suggestion": "Move MSI elements into an MsiPackage or create a separate .wxs file compiled as an MSI.",
        "auto_fixable": 0
    },
    "BUNDLE002": {
        "rationale": "ExePackage needs DetectCondition to know if already installed, and UninstallArguments for clean removal. Without these, upgrades and uninstalls fail.",
        "fix_suggestion": "Add DetectCondition and UninstallArguments attributes, or set Permanent=\"yes\" if the package should never be uninstalled.",
        "auto_fixable": 0
    },
    "BUNDLE003": {
        "rationale": "VC++ redistributables support the Burn protocol for better progress reporting and silent handling. Using Protocol=burn improves user experience.",
        "fix_suggestion": "Add Protocol=\"burn\" to ExePackage for VC++ redistributables that support it.",
        "auto_fixable": 0
    },

    # Component rules
    "COMP001": {
        "rationale": "Component GUIDs identify components across installations and upgrades. Without a GUID, Windows Installer cannot track the component, breaking repair and upgrade.",
        "fix_suggestion": "Add Guid=\"*\" for auto-generation based on component contents, or specify an explicit GUID.",
        "auto_fixable": 1
    },
    "COMP002": {
        "rationale": "Components with multiple files complicate repair and patching. If any file is damaged, all files must be restored. One file per component follows ICE guidelines.",
        "fix_suggestion": "Split into multiple components with one file each. Use ComponentGroup to keep them organized.",
        "auto_fixable": 0
    },
    "COMP003": {
        "rationale": "KeyPath identifies the component for installation state. Without it, Windows Installer cannot determine if the component is installed, breaking repair.",
        "fix_suggestion": "Add KeyPath=\"yes\" to the primary File element, or set a RegistryValue as the KeyPath.",
        "auto_fixable": 1
    },
    "COMP004": {
        "rationale": "Matching Component and File Ids simplifies maintenance and makes the relationship clear. This is a best practice for single-file components.",
        "fix_suggestion": "Change File Id to match Component Id, or vice versa.",
        "auto_fixable": 1
    },
    "COMP005": {
        "rationale": "ICE57: Components must install all resources to one directory. Multi-directory components cause installation tracking failures.",
        "fix_suggestion": "Split the component into separate components, one for each target directory.",
        "auto_fixable": 0
    },
    "COMP006": {
        "rationale": "ICE92: Permanent components (NeverOverwrite) must have stable GUIDs for reference counting. Auto-generated GUIDs change when contents change.",
        "fix_suggestion": "Add an explicit Guid attribute with a stable GUID value.",
        "auto_fixable": 0
    },
    "COMP007": {
        "rationale": "ICE57: Components cannot mix per-user (HKCU) and per-machine (HKLM) resources. Windows Installer tracks them differently.",
        "fix_suggestion": "Create separate components for per-user and per-machine resources.",
        "auto_fixable": 0
    },
    "COMP008": {
        "rationale": "ICE30: Same file path in multiple components causes conflicts. Only one component should own each file path.",
        "fix_suggestion": "Remove duplicate File elements or use different target paths.",
        "auto_fixable": 0
    },
    "COMP009": {
        "rationale": "ICE38: Components in user profile directories should use HKCU registry as KeyPath. File-based KeyPath in roaming profiles causes issues.",
        "fix_suggestion": "Add a RegistryValue with KeyPath=\"yes\" under HKCU for the component.",
        "auto_fixable": 0
    },
    "COMP010": {
        "rationale": "Components not referenced by any Feature will never be installed. They waste space in the package and may indicate missing FeatureRefs.",
        "fix_suggestion": "Add ComponentRef to a Feature, or remove the orphaned component if it's not needed.",
        "auto_fixable": 0
    },
    "COMP011": {
        "rationale": "Component directory attribute must reference a valid Directory or DirectoryRef. Invalid references cause build or installation failures.",
        "fix_suggestion": "Correct the Directory attribute to reference a valid directory Id.",
        "auto_fixable": 0
    },
    "COMP012": {
        "rationale": "Empty components serve no purpose and waste package space. Components should contain at least one resource.",
        "fix_suggestion": "Add files, registry values, or other resources to the component, or remove it entirely.",
        "auto_fixable": 0
    },
    "COMP013": {
        "rationale": "64-bit components in 32-bit packages may fail to install on 32-bit systems or cause registry redirection issues.",
        "fix_suggestion": "Create a 64-bit package for 64-bit components, or use conditional components with architecture checks.",
        "auto_fixable": 0
    },
    "COMP014": {
        "rationale": "Windows Installer limits identifiers to 72 characters. Longer Ids cause build failures or truncation issues.",
        "fix_suggestion": "Shorten the Component Id to 72 characters or less.",
        "auto_fixable": 0
    },
    "COMP015": {
        "rationale": "Component conditions referencing other components create implicit dependencies. This can cause installation order issues.",
        "fix_suggestion": "Use Feature conditions or explicit Property references instead of component-based conditions.",
        "auto_fixable": 0
    },

    # Custom action rules
    "CA001": {
        "rationale": "Immediate custom actions run before installation changes. Actions modifying system state should run deferred with elevated privileges.",
        "fix_suggestion": "Change Execute=\"immediate\" to Execute=\"deferred\" for system-modifying actions.",
        "auto_fixable": 1
    },
    "CA002": {
        "rationale": "Execute attribute specifies when the action runs. Without it, the default may not match the intended behavior.",
        "fix_suggestion": "Add Execute=\"immediate\" or Execute=\"deferred\" based on when the action should run.",
        "auto_fixable": 0
    },
    "CA003": {
        "rationale": "Deferred actions that modify system state should have rollback actions. Without rollback, failed installations leave partial changes.",
        "fix_suggestion": "Create a corresponding rollback custom action and schedule it before the deferred action.",
        "auto_fixable": 0
    },
    "CA004": {
        "rationale": "VBScript/JScript custom actions are slower, less reliable, and blocked by some security policies. Native DLL or C# actions are preferred.",
        "fix_suggestion": "Rewrite the script as a native DLL custom action or use WiX built-in elements when possible.",
        "auto_fixable": 0
    },
    "CA005": {
        "rationale": "DLL custom actions must specify the function entry point. Without it, Windows Installer cannot locate and call the function.",
        "fix_suggestion": "Add DllEntry attribute with the exported function name: DllEntry=\"FunctionName\"",
        "auto_fixable": 0
    },
    "CA006": {
        "rationale": "Deferred actions with Impersonate=yes run as the installing user, not SYSTEM. This may lack privileges for system modifications.",
        "fix_suggestion": "Set Impersonate=\"no\" for deferred actions that need SYSTEM privileges.",
        "auto_fixable": 1
    },
    "CA007": {
        "rationale": "Custom actions referencing binaries must have matching Binary elements. Missing binaries cause build or runtime failures.",
        "fix_suggestion": "Add a Binary element with matching Id, or correct the BinaryRef attribute.",
        "auto_fixable": 0
    },
    "CA008": {
        "rationale": "Custom actions must be scheduled in a sequence table to execute. Unscheduled actions are never called.",
        "fix_suggestion": "Add InstallExecuteSequence or InstallUISequence element to schedule the action.",
        "auto_fixable": 0
    },
    "CA009": {
        "rationale": "Type 1 (DLL) custom actions need files extracted. Running before InstallFiles means the DLL may not exist yet.",
        "fix_suggestion": "Schedule the action after InstallFiles, or use Type 17/21 for embedded binaries.",
        "auto_fixable": 0
    },
    "CA010": {
        "rationale": "MSI custom actions must return UINT (ERROR_SUCCESS, ERROR_INSTALL_FAILURE, etc.). Wrong return types cause undefined behavior.",
        "fix_suggestion": "Ensure the custom action function returns UINT and uses proper MSI return codes.",
        "auto_fixable": 0
    },
    "CA011": {
        "rationale": "Async custom actions (Continue=yes) don't wait for completion. This can cause timing issues if later actions depend on their results.",
        "fix_suggestion": "Remove Continue=yes unless truly parallel execution is needed. Ensure no dependencies on async results.",
        "auto_fixable": 0
    },
    "CA012": {
        "rationale": "Custom actions have different capabilities in different sequences. UI sequence runs before install; Execute sequence has full privileges.",
        "fix_suggestion": "Move the action to the appropriate sequence: UI for user interaction, Execute for system changes.",
        "auto_fixable": 0
    },
    "CA013": {
        "rationale": "Type 51 custom actions set properties. Without a Value attribute, the property is cleared, which may not be intended.",
        "fix_suggestion": "Add Value attribute with the property value to set: Value=\"[INSTALLDIR]\"",
        "auto_fixable": 0
    },
    "CA014": {
        "rationale": "Deferred custom actions cannot access session properties directly. Properties must be passed via CustomActionData.",
        "fix_suggestion": "Create a Type 51 immediate action to set CustomActionData property before the deferred action.",
        "auto_fixable": 0
    },
    "CA015": {
        "rationale": "PowerShell custom actions require the PowerShell runtime and may be blocked by execution policies. Native actions are faster and more reliable.",
        "fix_suggestion": "Consider rewriting as a native DLL action, or ensure PowerShell prerequisites are met.",
        "auto_fixable": 0
    },

    # Deprecated rules
    "DEPR001": {
        "rationale": "Deprecated elements may be removed in future WiX versions. Using current elements ensures forward compatibility.",
        "fix_suggestion": "Replace with the recommended WiX v4/v5 element. Check migration documentation for the equivalent.",
        "auto_fixable": 0
    },
    "DEPR002": {
        "rationale": "Deprecated attributes may be removed in future versions. Using current attributes ensures forward compatibility.",
        "fix_suggestion": "Replace with the recommended WiX v4/v5 attribute. Check documentation for the equivalent.",
        "auto_fixable": 0
    },
    "DEPR003": {
        "rationale": "WiX v3 schema uses different namespace and element names. Updating to v4 schema provides new features and better tooling support.",
        "fix_suggestion": "Run 'wix convert' to automatically migrate WiX v3 files to v4/v5 format.",
        "auto_fixable": 0
    },
    "DEPR004": {
        "rationale": "Extension namespaces changed between WiX versions. Updated namespaces are required for v4/v5 compatibility.",
        "fix_suggestion": "Update namespace URI from WixToolset.XXX/v3 to appropriate v4/v5 namespace.",
        "auto_fixable": 1
    },
    "DEPR005": {
        "rationale": "Product element is WiX v3 style. Package element is the WiX v4/v5 equivalent with cleaner syntax.",
        "fix_suggestion": "Replace <Product> with <Package> and update child elements accordingly.",
        "auto_fixable": 0
    },
    "DEPR006": {
        "rationale": "DirectoryRef for standard locations is verbose. StandardDirectory provides a cleaner, self-documenting reference.",
        "fix_suggestion": "Replace <DirectoryRef Id=\"ProgramFilesFolder\"> with <StandardDirectory Id=\"ProgramFilesFolder\">",
        "auto_fixable": 1
    },

    # Directory rules
    "DIR001": {
        "rationale": "Directory elements need Name to create the folder. Without Name, the directory structure is incomplete.",
        "fix_suggestion": "Add Name attribute with the folder name: Name=\"MyFolder\"",
        "auto_fixable": 0
    },
    "DIR002": {
        "rationale": "Hardcoded paths like C:\\Program Files break on systems with different configurations. Standard directories adapt automatically.",
        "fix_suggestion": "Use StandardDirectory references like ProgramFilesFolder, or properties like [ProgramFilesFolder].",
        "auto_fixable": 0
    },
    "DIR003": {
        "rationale": "Deep nesting (>5 levels) creates long paths that may exceed Windows MAX_PATH limit, especially with long file names.",
        "fix_suggestion": "Flatten the directory structure. Consider using fewer nested folders.",
        "auto_fixable": 0
    },
    "DIR004": {
        "rationale": "Windows Installer limits identifiers to 72 characters. Longer Ids cause build failures.",
        "fix_suggestion": "Shorten the Directory Id to 72 characters or less.",
        "auto_fixable": 0
    },
    "DIR005": {
        "rationale": "Circular directory references (A -> B -> A) create infinite loops and cause build failures.",
        "fix_suggestion": "Review directory parent references and remove the circular dependency.",
        "auto_fixable": 0
    },
    "DIR006": {
        "rationale": "Spaces in directory names can cause issues with some tools and scripts that don't handle quoting properly.",
        "fix_suggestion": "Use CamelCase or underscores instead of spaces: MyFolder or My_Folder",
        "auto_fixable": 0
    },
    "DIR007": {
        "rationale": "Users expect applications in standard locations (Program Files). Non-standard locations may confuse users or trigger security warnings.",
        "fix_suggestion": "Install to ProgramFilesFolder or LocalAppDataFolder for standard user expectations.",
        "auto_fixable": 0
    },
    "DIR008": {
        "rationale": "Empty directories are not created by default. CreateFolder element explicitly creates the directory during installation.",
        "fix_suggestion": "Add <CreateFolder/> inside a Component to create the empty directory.",
        "auto_fixable": 0
    },
    "DIR009": {
        "rationale": "Standard directory IDs are predefined by Windows Installer. Invalid IDs are not recognized and cause installation failures.",
        "fix_suggestion": "Use valid StandardDirectory IDs like ProgramFilesFolder, CommonAppDataFolder, etc.",
        "auto_fixable": 0
    },
    "DIR010": {
        "rationale": "Directories with RemoveFolder only during uninstall may leave empty folders after repair or minor upgrade.",
        "fix_suggestion": "Consider adding RemoveFolder with On=\"both\" or On=\"install\" for cleanup during repairs.",
        "auto_fixable": 0
    },

    # Extension rules
    "EXT001": {
        "rationale": "IIS websites need bindings to accept connections. Without bindings, the website cannot respond to requests.",
        "fix_suggestion": "Add WebAddress child element with port and optionally IP and hostname bindings.",
        "auto_fixable": 0
    },
    "EXT002": {
        "rationale": "ApplicationPoolIdentity is the recommended identity providing least-privilege security. Custom accounts may have excessive privileges.",
        "fix_suggestion": "Use Identity=\"applicationPoolIdentity\" instead of custom accounts where possible.",
        "auto_fixable": 0
    },
    "EXT003": {
        "rationale": "SQL operations can fail mid-installation. Without rollback scripts, the database may be left in an inconsistent state.",
        "fix_suggestion": "Add RollbackScript attribute with SQL to undo changes on installation failure.",
        "auto_fixable": 0
    },
    "EXT004": {
        "rationale": "Domain user accounts may not exist on standalone machines or in different domains. This causes installation failures.",
        "fix_suggestion": "Use local accounts or add conditions to check domain availability before creating domain users.",
        "auto_fixable": 0
    },
    "EXT005": {
        "rationale": "Target machines may not have the required .NET version installed. Installation may fail or the application may not run.",
        "fix_suggestion": "Add DotNetCompatibilityCheck or prerequisite bundles to ensure .NET availability.",
        "auto_fixable": 0
    },
    "EXT006": {
        "rationale": "Firewall rules without protocol specification may allow unintended traffic. Always specify TCP, UDP, or specific protocol.",
        "fix_suggestion": "Add Protocol attribute: Protocol=\"tcp\" or Protocol=\"udp\"",
        "auto_fixable": 0
    },
    "EXT007": {
        "rationale": "XmlFile transformations are complex. Changes may have unintended effects on existing configurations. Thorough testing is essential.",
        "fix_suggestion": "Test XML transformations on various existing configurations. Consider using Permanent=\"yes\" for user settings.",
        "auto_fixable": 0
    },
    "EXT008": {
        "rationale": "PATH modifications need proper semicolon delimiters. Missing delimiters can corrupt the PATH, breaking other applications.",
        "fix_suggestion": "Use Action=\"set\" with Part=\"last\" to properly append to PATH with correct delimiter handling.",
        "auto_fixable": 0
    },

    # Feature rules
    "FEAT001": {
        "rationale": "Feature Title is displayed in the installation UI. Without it, users see blank entries in feature selection.",
        "fix_suggestion": "Add Title attribute with user-friendly text: Title=\"Main Application\"",
        "auto_fixable": 0
    },
    "FEAT002": {
        "rationale": "Features without components don't install anything. They waste UI space unless they're parent containers for sub-features.",
        "fix_suggestion": "Add ComponentRef or ComponentGroupRef elements, or verify this is intentionally a parent-only feature.",
        "auto_fixable": 0
    },
    "FEAT003": {
        "rationale": "Level=\"0\" means the feature is disabled and won't install by default. Users must explicitly select it in custom UI.",
        "fix_suggestion": "Set Level=\"1\" or higher for features that should install by default.",
        "auto_fixable": 0
    },
    "FEAT004": {
        "rationale": "Feature Ids are limited to 38 characters by Windows Installer. Longer Ids cause build failures.",
        "fix_suggestion": "Shorten the Feature Id to 38 characters or less.",
        "auto_fixable": 0
    },
    "FEAT005": {
        "rationale": "Circular feature references (A -> B -> A) create infinite loops during installation costing.",
        "fix_suggestion": "Review Feature parent references and remove the circular dependency.",
        "auto_fixable": 0
    },
    "FEAT006": {
        "rationale": "Deep feature hierarchies (>16 levels) slow down installation costing and may cause UI display issues.",
        "fix_suggestion": "Flatten the feature hierarchy. Consider grouping related features differently.",
        "auto_fixable": 0
    },
    "FEAT007": {
        "rationale": "Windows Installer has a practical limit around 32000 features. Exceeding this causes installation failures.",
        "fix_suggestion": "Reduce feature count by combining related features or using component groups.",
        "auto_fixable": 0
    },
    "FEAT008": {
        "rationale": "Feature Description appears in UI when users hover or select features. It helps users understand what they're installing.",
        "fix_suggestion": "Add Description attribute with helpful text explaining the feature.",
        "auto_fixable": 0
    },
    "FEAT009": {
        "rationale": "Features with Level=\"0\" are hidden and not installed by default. Components in such features won't be installed unless explicitly selected.",
        "fix_suggestion": "Verify the hidden feature is intentional. Set Level=\"1\" or higher if it should install by default.",
        "auto_fixable": 0
    },
    "FEAT010": {
        "rationale": "At least one root (top-level) feature must exist for installation to work. Nested-only features have no installation path.",
        "fix_suggestion": "Ensure at least one Feature element exists without a parent Feature reference.",
        "auto_fixable": 0
    },

    # File rules
    "FILE001": {
        "rationale": "Source attribute specifies the file to include in the package. Without it, WiX doesn't know what file to package.",
        "fix_suggestion": "Add Source attribute with path to the source file: Source=\"bin\\MyApp.exe\"",
        "auto_fixable": 0
    },
    "FILE002": {
        "rationale": "File Ids are used in database tables and may be exposed in logs. Invalid characters can cause database or parsing errors.",
        "fix_suggestion": "Use only alphanumeric characters, underscores, and periods. Remove spaces and special characters.",
        "auto_fixable": 1
    },
    "FILE003": {
        "rationale": "8.3 format (8 char name, 3 char extension) is legacy. Long names work on modern systems but log messages may be about this.",
        "fix_suggestion": "This is informational. Long file names are fine for modern Windows systems.",
        "auto_fixable": 0
    },
    "FILE004": {
        "rationale": "Source file must exist at build time. Missing files cause build failures.",
        "fix_suggestion": "Verify the source file path is correct. Ensure the file exists before building.",
        "auto_fixable": 0
    },
    "FILE005": {
        "rationale": "Version information helps Windows Installer track file state. Unversioned files may be unnecessarily replaced during repair.",
        "fix_suggestion": "Add version info to executables and DLLs. For non-executable files, this is often acceptable.",
        "auto_fixable": 0
    },
    "FILE006": {
        "rationale": "Duplicate File Ids cause database conflicts. Each file must have a unique identifier.",
        "fix_suggestion": "Change one of the duplicate File Ids to a unique value.",
        "auto_fixable": 0
    },
    "FILE007": {
        "rationale": "Win32 assemblies with SxS manifests need the manifest for proper DLL isolation. Missing manifests cause application errors.",
        "fix_suggestion": "Include the associated manifest file in the same component.",
        "auto_fixable": 0
    },
    "FILE008": {
        "rationale": "DLLs without version info cannot be properly tracked by Windows Installer. This affects repair and patching scenarios.",
        "fix_suggestion": "Add version information to DLL files using resource editors or build-time versioning.",
        "auto_fixable": 0
    },
    "FILE009": {
        "rationale": "Windows file names cannot contain: \\ / : * ? \" < > |. Such files cannot be created on the target system.",
        "fix_suggestion": "Rename the file to remove invalid characters.",
        "auto_fixable": 0
    },
    "FILE010": {
        "rationale": "Windows MAX_PATH is 260 characters. Deep paths with long names may exceed this, causing installation failures.",
        "fix_suggestion": "Use shorter directory or file names. Consider flattening the directory structure.",
        "auto_fixable": 0
    },
    "FILE011": {
        "rationale": "PE checksums help detect file corruption. Executables without checksums may not be detected as corrupt.",
        "fix_suggestion": "Use build tools that add checksums to executables. Visual C++ linker adds checksums by default.",
        "auto_fixable": 0
    },
    "FILE012": {
        "rationale": "Windows expects fonts in the Fonts folder (FontsFolder). Fonts elsewhere may not be available system-wide.",
        "fix_suggestion": "Install font files to FontsFolder StandardDirectory.",
        "auto_fixable": 0
    },
    "FILE013": {
        "rationale": "Very large files (>100MB) significantly impact download and installation time. Consider compression or on-demand download.",
        "fix_suggestion": "This is informational. Consider compressing large files or using external cabinet files.",
        "auto_fixable": 0
    },
    "FILE014": {
        "rationale": "PDB files contain debug symbols and are large. They should not be included in release packages.",
        "fix_suggestion": "Remove .pdb files from Source paths, or exclude them from the file pattern.",
        "auto_fixable": 0
    },
    "FILE015": {
        "rationale": "Temporary files (.tmp, ~files, backup files) should not be deployed. They indicate build artifacts accidentally included.",
        "fix_suggestion": "Remove temporary files from source directories before building.",
        "auto_fixable": 0
    },

    # Localization rules
    "LOC001": {
        "rationale": "Hardcoded UI strings cannot be translated. Using localization variables enables multi-language support.",
        "fix_suggestion": "Replace hardcoded text with !(loc.StringId) references and define strings in .wxl files.",
        "auto_fixable": 0
    },
    "LOC002": {
        "rationale": "Missing translations result in fallback to default language, which may confuse non-English users.",
        "fix_suggestion": "Add the missing string to the appropriate language .wxl file.",
        "auto_fixable": 0
    },
    "LOC003": {
        "rationale": "Consistent naming conventions for localization strings improve maintainability and reduce translation errors.",
        "fix_suggestion": "Follow naming pattern: Category_Element_Purpose, e.g., Dialog_Welcome_Title",
        "auto_fixable": 0
    },
    "LOC004": {
        "rationale": "Unused localization strings waste translator effort and clutter the .wxl files.",
        "fix_suggestion": "Remove the unused string from the .wxl file.",
        "auto_fixable": 0
    },

    # Naming rules
    "NAME001": {
        "rationale": "Consistent naming conventions improve code readability and maintenance. Mixed conventions create confusion.",
        "fix_suggestion": "Establish and follow a naming convention: PascalCase, camelCase, or UPPER_SNAKE_CASE.",
        "auto_fixable": 0
    },
    "NAME002": {
        "rationale": "Reserved words (like Action, Error, Condition) conflict with Windows Installer table names or property names.",
        "fix_suggestion": "Prefix or suffix the Id to avoid conflict: MyAction, ErrorProperty.",
        "auto_fixable": 0
    },
    "NAME003": {
        "rationale": "Generated Ids like Component1, File_abc123 make maintenance difficult. Descriptive Ids are self-documenting.",
        "fix_suggestion": "Use descriptive Ids that indicate purpose: MainAppComponent, ConfigFile.",
        "auto_fixable": 0
    },
    "NAME004": {
        "rationale": "Some parsers and tools may have issues with Ids starting with numbers. Letters-first is more compatible.",
        "fix_suggestion": "Prefix with a letter: _123Feature becomes Feature_123 or F123.",
        "auto_fixable": 0
    },
    "NAME005": {
        "rationale": "Mixed casing (myId, MyId, MYID) reduces readability. Consistent casing is easier to maintain.",
        "fix_suggestion": "Use consistent casing throughout. PascalCase is common for WiX Ids.",
        "auto_fixable": 0
    },

    # Package rules
    "64BIT001": {
        "rationale": "64-bit packages must have correct Template (x64;1033) and Page Count values for Windows Installer to recognize them.",
        "fix_suggestion": "Set InstallerVersion=\"500\" and Platform=\"x64\" for 64-bit packages.",
        "auto_fixable": 0
    },
    "MEDIA001": {
        "rationale": "Every package needs a Media entry with DiskId=1. This is where files are stored.",
        "fix_suggestion": "Add MediaTemplate or Media element with DiskId=\"1\".",
        "auto_fixable": 1
    },
    "MEDIA002": {
        "rationale": "Excessive media entries (>80) may cause UI issues in disk prompts and slow installation setup.",
        "fix_suggestion": "Consolidate files into fewer cabinets. Use MediaTemplate for automatic cabinet handling.",
        "auto_fixable": 0
    },
    "PKG001": {
        "rationale": "UpgradeCode identifies the product family. Without it, major upgrades cannot detect and remove previous versions.",
        "fix_suggestion": "Add UpgradeCode attribute to Package element: UpgradeCode=\"{GUID}\"",
        "auto_fixable": 0
    },
    "PKG002": {
        "rationale": "MajorUpgrade element handles version upgrades automatically. Without it, old versions remain installed alongside new ones.",
        "fix_suggestion": "Add <MajorUpgrade DowngradeErrorMessage=\"...\"/> inside the Package element.",
        "auto_fixable": 1
    },
    "PKG003": {
        "rationale": "Semantic versioning (major.minor.patch) clearly communicates version changes and is widely understood.",
        "fix_suggestion": "Use version format A.B.C where A=major, B=minor, C=patch/build.",
        "auto_fixable": 0
    },
    "PKG004": {
        "rationale": "Hardcoded ProductCode prevents Windows Installer from detecting the package as a new version. Use * for auto-generation.",
        "fix_suggestion": "Replace hardcoded GUID with Guid=\"*\" for auto-generation.",
        "auto_fixable": 1
    },
    "PKG005": {
        "rationale": "Manufacturer appears in Add/Remove Programs. Missing it shows blank or unknown in the programs list.",
        "fix_suggestion": "Add Manufacturer attribute: Manufacturer=\"Your Company Name\"",
        "auto_fixable": 0
    },
    "PKG006": {
        "rationale": "ProductName appears in Add/Remove Programs and installation UI. Users need to identify what they're installing.",
        "fix_suggestion": "Add Name attribute to Package: Name=\"Your Product Name\"",
        "auto_fixable": 0
    },
    "PKG007": {
        "rationale": "InstallerVersion 500 (Windows Installer 5.0) is recommended for WiX v4 features. Lower versions lack newer capabilities.",
        "fix_suggestion": "Set InstallerVersion=\"500\" in Package element.",
        "auto_fixable": 1
    },
    "PKG008": {
        "rationale": "Uncompressed packages are larger to download but faster to install. Compressed is generally preferred.",
        "fix_suggestion": "Set Compressed=\"yes\" or use MediaTemplate which compresses by default.",
        "auto_fixable": 1
    },
    "PKG009": {
        "rationale": "ProductLanguage (LCID) enables proper localization detection and language-specific installation behavior.",
        "fix_suggestion": "Add Language attribute: Language=\"1033\" for English (US).",
        "auto_fixable": 0
    },
    "PKG010": {
        "rationale": "GUIDs must follow the standard format: {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX} with uppercase hex.",
        "fix_suggestion": "Correct the GUID format. Generate a new one with guidgen or uuidgen.",
        "auto_fixable": 0
    },
    "PKG011": {
        "rationale": "UpgradeCode identifies the product family; ProductCode identifies specific versions. Same value breaks upgrade detection.",
        "fix_suggestion": "Generate a new GUID for UpgradeCode, different from ProductCode.",
        "auto_fixable": 0
    },
    "PKG012": {
        "rationale": "Windows Installer ignores the fourth version field. Using it for meaningful versioning causes upgrade detection issues.",
        "fix_suggestion": "Use only three version components: major.minor.build.",
        "auto_fixable": 0
    },
    "PKG013": {
        "rationale": "Per-user installs don't require elevation but have limitations: no HKLM registry, limited to user profile directories.",
        "fix_suggestion": "This is informational. Verify per-user limitations are acceptable for your use case.",
        "auto_fixable": 0
    },
    "PKG014": {
        "rationale": "Codepage affects how non-ASCII characters are stored in the MSI database. UTF-8 (65001) is recommended.",
        "fix_suggestion": "Add Codepage=\"65001\" for UTF-8 support.",
        "auto_fixable": 1
    },
    "PKG015": {
        "rationale": "AdminImage creates an administrative install point for network deployment, not a regular user installation.",
        "fix_suggestion": "Remove AdminImage=\"yes\" unless creating an admin install point intentionally.",
        "auto_fixable": 0
    },
    "SUMMARY001": {
        "rationale": "ProductName is stored in Summary Information stream with a 63-character limit. Longer names are truncated.",
        "fix_suggestion": "Shorten the product name to 63 characters or less.",
        "auto_fixable": 0
    },
    "SUMMARY002": {
        "rationale": "Summary Information stream validation errors indicate MSI database corruption or invalid metadata.",
        "fix_suggestion": "Review package metadata. Rebuild the MSI if validation errors persist.",
        "auto_fixable": 0
    },
    "UPGRADE001": {
        "rationale": "RemoveExistingProducts sequencing affects when old version is removed. Wrong timing can cause file conflicts.",
        "fix_suggestion": "Use MajorUpgrade element which handles sequencing automatically, or place after InstallExecute.",
        "auto_fixable": 0
    },
    "UPGRADE002": {
        "rationale": "Upgrade table validation errors indicate malformed upgrade entries that will break major upgrades.",
        "fix_suggestion": "Review Upgrade element attributes. Ensure version ranges are valid.",
        "auto_fixable": 0
    },

    # Performance rules
    "PERF001": {
        "rationale": "Large component counts (>1000) increase costing time, database size, and may slow installation UI responsiveness.",
        "fix_suggestion": "This is informational. Consider ComponentGroups for organization. Performance impact is usually minor.",
        "auto_fixable": 0
    },
    "PERF002": {
        "rationale": "Very large embedded cabinets (>100MB) must be fully loaded into memory, potentially causing out-of-memory on low-resource systems.",
        "fix_suggestion": "Split into multiple smaller cabinets or use external cabinet files.",
        "auto_fixable": 0
    },
    "PERF003": {
        "rationale": "One file per component improves patching granularity. Only changed files need to be included in patches.",
        "fix_suggestion": "Split components to contain single files where practical.",
        "auto_fixable": 0
    },
    "PERF004": {
        "rationale": "Uncompressed cabinets are faster to install but larger to download. Trade-off depends on deployment scenario.",
        "fix_suggestion": "This is informational. Use compressed for web downloads, uncompressed for fast network deployment.",
        "auto_fixable": 0
    },
    "PERF005": {
        "rationale": "Many custom actions slow down installation phases. Each action adds execution time and potential failure points.",
        "fix_suggestion": "Consolidate related actions. Use built-in WiX elements instead of custom actions where possible.",
        "auto_fixable": 0
    },
    "PERF006": {
        "rationale": "Synchronous custom actions in UI sequence freeze the installer UI. Users may think the installer has hung.",
        "fix_suggestion": "Move long-running actions to Execute sequence, or use async with progress updates.",
        "auto_fixable": 0
    },
    "PERF007": {
        "rationale": "Deep feature hierarchies require more costing calculations. Very deep trees may noticeably slow the UI.",
        "fix_suggestion": "Flatten feature hierarchy where possible. Keep depth under 16 levels.",
        "auto_fixable": 0
    },
    "PERF008": {
        "rationale": "Large Binary table entries (custom action DLLs, icons) increase MSI size and memory usage during installation.",
        "fix_suggestion": "Optimize binary sizes. Consider external files for very large binaries.",
        "auto_fixable": 0
    },

    # Property rules
    "PROP001": {
        "rationale": "Windows Installer convention: PUBLIC properties are UPPERCASE, private properties are lowercase. Public properties can be set from command line.",
        "fix_suggestion": "Rename public properties to UPPERCASE, private properties to lowercase or PascalCase.",
        "auto_fixable": 0
    },
    "PROP002": {
        "rationale": "Built-in properties (INSTALLDIR, ProductCode, etc.) have special meaning. Overriding them causes unexpected behavior.",
        "fix_suggestion": "Rename the property to avoid conflict with built-in names.",
        "auto_fixable": 0
    },
    "PROP003": {
        "rationale": "Deferred custom actions cannot read session properties. Properties must be in SecureCustomProperties to be passed.",
        "fix_suggestion": "Add property to SecureCustomProperties or use CustomActionData to pass values.",
        "auto_fixable": 0
    },
    "PROP004": {
        "rationale": "Property values in MSI database are limited to 255 characters in some contexts. Longer values may be truncated.",
        "fix_suggestion": "Split long values into multiple properties or use external configuration files.",
        "auto_fixable": 0
    },
    "PROP005": {
        "rationale": "Unused properties waste database space and may indicate incomplete implementation or dead code.",
        "fix_suggestion": "Remove the unused property, or add references if it should be used.",
        "auto_fixable": 0
    },
    "PROP006": {
        "rationale": "Property Ids must be valid identifiers. Invalid characters cause database or parsing errors.",
        "fix_suggestion": "Use only alphanumeric characters and underscores. Start with a letter.",
        "auto_fixable": 0
    },
    "PROP007": {
        "rationale": "Restricted properties (SystemFolder, WindowsFolder, etc.) are set by Windows Installer. Modifying them may cause failures.",
        "fix_suggestion": "Don't modify restricted properties. Create custom properties for custom values.",
        "auto_fixable": 0
    },
    "PROP008": {
        "rationale": "Admin property values differ from normal install. Inconsistency may cause different behavior in admin vs normal installs.",
        "fix_suggestion": "Ensure Admin property values match expected behavior for administrative installations.",
        "auto_fixable": 0
    },
    "PROP009": {
        "rationale": "Properties named PASSWORD, KEY, SECRET, TOKEN may contain sensitive data. They should be marked Hidden and Secure.",
        "fix_suggestion": "Add Hidden=\"yes\" and ensure the property is in SecureCustomProperties.",
        "auto_fixable": 0
    },
    "PROP010": {
        "rationale": "Conditions referencing undefined properties always evaluate to false. This may cause unexpected installation behavior.",
        "fix_suggestion": "Define the property or correct the condition to reference an existing property.",
        "auto_fixable": 0
    },

    # Registry rules
    "REG001": {
        "rationale": "HKEY_LOCAL_MACHINE requires administrator privileges. Per-user installs cannot write to HKLM.",
        "fix_suggestion": "Use HKCU for per-user installs, or require elevation for HKLM access.",
        "auto_fixable": 0
    },
    "REG002": {
        "rationale": "Root attribute specifies the registry hive. Without it, the registry location is undefined.",
        "fix_suggestion": "Add Root attribute: Root=\"HKLM\" or Root=\"HKCU\"",
        "auto_fixable": 0
    },
    "REG003": {
        "rationale": "HKCU entries in per-machine installs only affect the installing user. Other users won't see these settings.",
        "fix_suggestion": "Use HKLM for shared settings, or document that HKCU settings are per-user.",
        "auto_fixable": 0
    },
    "REG004": {
        "rationale": "Registry key paths have maximum length limits. Very long paths may cause errors on some systems.",
        "fix_suggestion": "Shorten the registry key path. Consider restructuring the registry hierarchy.",
        "auto_fixable": 0
    },
    "REG005": {
        "rationale": "Backslash in value name suggests the key path is incomplete. Value names rarely contain backslashes.",
        "fix_suggestion": "Move the backslash-prefixed portion to the Key path.",
        "auto_fixable": 0
    },
    "REG006": {
        "rationale": "HKCR is a virtual hive. Writes go to HKLM\\Software\\Classes or HKCU\\Software\\Classes depending on context.",
        "fix_suggestion": "Use explicit HKLM or HKCU with Software\\Classes path for predictable behavior.",
        "auto_fixable": 0
    },
    "REG007": {
        "rationale": "REG_EXPAND_SZ automatically expands environment variables. Without variables, REG_SZ is more efficient.",
        "fix_suggestion": "Change Type to String (REG_SZ) if no environment variable expansion is needed.",
        "auto_fixable": 1
    },
    "REG008": {
        "rationale": "ForceCreateOnInstall overwrites existing values. User customizations may be lost on reinstall or repair.",
        "fix_suggestion": "Remove ForceCreateOnInstall to preserve existing values, or document the overwrite behavior.",
        "auto_fixable": 0
    },
    "REG009": {
        "rationale": "Valid registry types: String, ExpandString, MultiString, Integer, Binary. Invalid types cause errors.",
        "fix_suggestion": "Use a valid Type value: string, expandString, multiString, integer, or binary.",
        "auto_fixable": 0
    },
    "REG010": {
        "rationale": "System-protected locations (HKLM\\SYSTEM, HKLM\\SAM) require special privileges and may be blocked by security software.",
        "fix_suggestion": "Avoid system-protected registry locations. Use HKLM\\SOFTWARE for application settings.",
        "auto_fixable": 0
    },

    # Security rules
    "SEC001": {
        "rationale": "Program Files is protected by default. Installing there without elevation fails on standard user accounts.",
        "fix_suggestion": "Request elevation (InstallScope=\"perMachine\") or install to LocalAppDataFolder.",
        "auto_fixable": 0
    },
    "SEC002": {
        "rationale": "World-writable files can be modified by any user, potentially allowing privilege escalation or malware injection.",
        "fix_suggestion": "Remove write permissions for Everyone. Grant write only to administrators or specific users.",
        "auto_fixable": 0
    },
    "SEC003": {
        "rationale": "Elevated custom actions run with SYSTEM privileges. Unvalidated input could allow privilege escalation attacks.",
        "fix_suggestion": "Validate all input parameters. Use parameterized queries for databases. Avoid shell execution.",
        "auto_fixable": 0
    },
    "SEC004": {
        "rationale": "Hardcoded passwords in MSI packages are visible to anyone with package access. Credentials should never be in installers.",
        "fix_suggestion": "Remove hardcoded credentials. Prompt users at install time or use secure credential storage.",
        "auto_fixable": 0
    },
    "SEC005": {
        "rationale": "DLLs loaded from unqualified paths can be hijacked. Attackers can place malicious DLLs in the search path.",
        "fix_suggestion": "Use fully qualified paths for DLL custom actions. Store DLLs in Binary table.",
        "auto_fixable": 0
    },
    "SEC006": {
        "rationale": "HTTP downloads can be intercepted and modified. HTTPS ensures integrity and authenticity of downloaded content.",
        "fix_suggestion": "Change download URLs from http:// to https://",
        "auto_fixable": 1
    },
    "SEC007": {
        "rationale": "Overly permissive registry ACLs allow unauthorized modifications to application settings or configuration.",
        "fix_suggestion": "Restrict permissions to administrators and the application's identity. Remove Everyone write access.",
        "auto_fixable": 0
    },
    "SEC008": {
        "rationale": "Services with admin rights can be exploited for privilege escalation. Follow least-privilege principle.",
        "fix_suggestion": "Use LocalService or NetworkService. Create custom accounts with minimal required permissions.",
        "auto_fixable": 0
    },
    "SEC009": {
        "rationale": "Broad firewall rules (all ports, all addresses) expose the system to unnecessary network risks.",
        "fix_suggestion": "Limit to specific ports and protocols. Use application-based rules instead of port-based.",
        "auto_fixable": 0
    },
    "SEC010": {
        "rationale": "Unsigned binaries may trigger security warnings and cannot be verified as unmodified from the publisher.",
        "fix_suggestion": "Sign all executable files with a code signing certificate.",
        "auto_fixable": 0
    },
    "SEC011": {
        "rationale": "Script execution may be blocked by Group Policy or security software. Script-based custom actions may fail silently.",
        "fix_suggestion": "Use compiled custom actions, or ensure script execution prerequisites are documented.",
        "auto_fixable": 0
    },
    "SEC012": {
        "rationale": "World-writable ProgramData allows any user to modify shared application data, potentially corrupting or hijacking it.",
        "fix_suggestion": "Set appropriate ACLs on ProgramData directories. Restrict write access to administrators.",
        "auto_fixable": 0
    },

    # Service rules
    "SVC001": {
        "rationale": "ServiceInstall requires Name (service name), Start (start type), ErrorControl (failure behavior), and Type (service type).",
        "fix_suggestion": "Add missing required attributes: Name, Start, ErrorControl, and Type.",
        "auto_fixable": 0
    },
    "SVC002": {
        "rationale": "ServiceControl elements start/stop services during install/uninstall. Without them, services may not start or leave orphan processes.",
        "fix_suggestion": "Add ServiceControl with Start=\"install\" and Stop=\"both\" for proper lifecycle management.",
        "auto_fixable": 1
    },
    "SVC003": {
        "rationale": "Services starting before files are installed will fail because the executable doesn't exist yet.",
        "fix_suggestion": "Schedule service start after InstallFiles and before InstallFinalize.",
        "auto_fixable": 0
    },
    "SVC004": {
        "rationale": "LocalSystem has full system privileges. Compromised services running as LocalSystem can control the entire machine.",
        "fix_suggestion": "Use LocalService or NetworkService. Create a dedicated service account with minimal rights.",
        "auto_fixable": 0
    },
    "SVC005": {
        "rationale": "Services need an executable to run. ServiceInstall without a File in the component cannot function.",
        "fix_suggestion": "Add the service executable as a File in the same Component as ServiceInstall.",
        "auto_fixable": 0
    },
    "SVC006": {
        "rationale": "Service Description appears in Services control panel. Without it, administrators see blank descriptions.",
        "fix_suggestion": "Add Description attribute with text explaining the service purpose.",
        "auto_fixable": 0
    },
    "SVC007": {
        "rationale": "Circular service dependencies prevent any of the services from starting. The system cannot resolve the dependency order.",
        "fix_suggestion": "Remove the circular dependency. Restructure services if needed.",
        "auto_fixable": 0
    },
    "SVC008": {
        "rationale": "Auto-start services with no recovery options stay stopped after failures. Users may not notice the service is down.",
        "fix_suggestion": "Add ServiceConfig with OnFailure actions: restart, reboot, or run program.",
        "auto_fixable": 0
    },
    "SVC009": {
        "rationale": "Hardcoded service accounts (specific usernames) may not exist on target machines, causing installation failures.",
        "fix_suggestion": "Use built-in accounts (LocalService, NetworkService) or create the account during installation.",
        "auto_fixable": 0
    },
    "SVC010": {
        "rationale": "Interactive services that display UI on the desktop are deprecated in Windows Vista and later. They don't work in Session 0.",
        "fix_suggestion": "Remove Interactive=\"yes\". Use other IPC mechanisms for service-to-user communication.",
        "auto_fixable": 0
    },

    # Shortcut rules
    "SHORT001": {
        "rationale": "Non-advertised shortcut targets must be in the same component as the shortcut for proper reference counting.",
        "fix_suggestion": "Move the shortcut to the same component as its target, or use advertised shortcuts.",
        "auto_fixable": 0
    },
    "SHORT002": {
        "rationale": "GAC assemblies don't have stable file paths. Advertised shortcuts use MSI resolution to find the correct path.",
        "fix_suggestion": "Set Advertise=\"yes\" for shortcuts pointing to GAC-installed assemblies.",
        "auto_fixable": 0
    },
    "SHORT003": {
        "rationale": "Public properties can be modified from command line. Using them for shortcut directories is a potential security risk.",
        "fix_suggestion": "Use Directory references instead of public properties for shortcut locations.",
        "auto_fixable": 0
    },
    "SHRT001": {
        "rationale": "Shortcuts must point to something. Without Target, the shortcut has no destination.",
        "fix_suggestion": "Add Target attribute with FileRef, DirectoryRef, or property reference: Target=\"[#FileId]\"",
        "auto_fixable": 0
    },
    "SHRT002": {
        "rationale": "Shortcuts in non-standard locations (not Start Menu or Desktop) may be difficult for users to find.",
        "fix_suggestion": "Consider using ProgramMenuFolder or DesktopFolder for discoverability.",
        "auto_fixable": 0
    },
    "SHRT003": {
        "rationale": "Shortcuts without icons use generic system icons. Custom icons improve user experience and branding.",
        "fix_suggestion": "Add Icon attribute referencing an Icon element: Icon=\"IconId\"",
        "auto_fixable": 0
    },
    "SHRT004": {
        "rationale": "WorkingDirectory must reference a valid Directory. Invalid references cause the application to start in wrong location.",
        "fix_suggestion": "Correct WorkingDirectory to reference a valid Directory Id.",
        "auto_fixable": 0
    },
    "SHRT005": {
        "rationale": "Arguments in Target are harder to maintain. The Arguments attribute separates concerns and is easier to read.",
        "fix_suggestion": "Move command-line arguments from Target to the Arguments attribute.",
        "auto_fixable": 0
    },
    "SHRT006": {
        "rationale": "Advertised shortcuts work differently per-user vs per-machine. Per-user advertised shortcuts may not self-repair correctly.",
        "fix_suggestion": "Test advertised shortcuts in per-user scenarios. Consider non-advertised for per-user installs.",
        "auto_fixable": 0
    },
    "SHRT007": {
        "rationale": "Multiple shortcuts to the same executable may indicate duplication. Verify each shortcut is intentional.",
        "fix_suggestion": "This is informational. Verify each shortcut serves a distinct purpose.",
        "auto_fixable": 0
    },
    "SHRT008": {
        "rationale": "Shortcut names become file names. Characters invalid in file names cause shortcut creation to fail.",
        "fix_suggestion": "Remove invalid characters: \\ / : * ? \" < > | from shortcut Name.",
        "auto_fixable": 0
    },
    "SHRT009": {
        "rationale": "Desktop shortcuts are visible to users. Automatically creating them may clutter desktops and annoy users.",
        "fix_suggestion": "Make desktop shortcuts optional via feature selection or installer checkbox.",
        "auto_fixable": 0
    },
    "SHRT010": {
        "rationale": "URL shortcuts without icons display generic browser icons. Custom icons improve recognition.",
        "fix_suggestion": "Add IconFile attribute pointing to an .ico file or executable with icon.",
        "auto_fixable": 0
    },

    # UI rules
    "UI001": {
        "rationale": "Custom UIs without error handling leave users confused when errors occur. Standard error dialogs should be included.",
        "fix_suggestion": "Include WixUI_ErrorProgressText or add custom Error dialog handling.",
        "auto_fixable": 0
    },
    "UI002": {
        "rationale": "Many software licenses require acceptance before installation. Missing license dialog may violate licensing terms.",
        "fix_suggestion": "Use WixUIExtension with license dialog, or add custom LicenseAgreementDlg.",
        "auto_fixable": 0
    },
    "UI003": {
        "rationale": "Overlapping controls hide each other. Users cannot see or interact with hidden controls.",
        "fix_suggestion": "Adjust control positions and sizes to prevent overlap.",
        "auto_fixable": 0
    },
    "UI004": {
        "rationale": "Users expect to cancel installation at any time. Dialogs without cancel frustrate users.",
        "fix_suggestion": "Add a Cancel button with DoAction EndDialog=Exit.",
        "auto_fixable": 0
    },
    "UI005": {
        "rationale": "Control too small for its text content causes truncation. Users cannot read the full message.",
        "fix_suggestion": "Increase control width/height, or reduce text length.",
        "auto_fixable": 0
    },
    "UI006": {
        "rationale": "Dialog actions referencing non-existent dialogs cause runtime errors when triggered.",
        "fix_suggestion": "Correct the dialog reference to an existing dialog Id.",
        "auto_fixable": 0
    },
    "UI007": {
        "rationale": "Missing bitmap files cause dialogs to display incorrectly or show error placeholders.",
        "fix_suggestion": "Ensure all referenced bitmap files exist at the specified paths.",
        "auto_fixable": 0
    },
    "UI008": {
        "rationale": "Large bitmaps (>100KB) significantly increase MSI package size. Consider optimization.",
        "fix_suggestion": "Compress or resize bitmaps. Use appropriate color depth.",
        "auto_fixable": 0
    },
    "UI009": {
        "rationale": "UIRef references UI elements from libraries. Without corresponding elements, the reference fails.",
        "fix_suggestion": "Include WixUIExtension or ensure referenced UI elements are defined.",
        "auto_fixable": 0
    },
    "UI010": {
        "rationale": "ActionText provides progress dialog messages. Missing text leaves users without installation status.",
        "fix_suggestion": "Add ActionText elements for custom actions and standard actions.",
        "auto_fixable": 0
    },

    # Upgrade rules
    "UPG001": {
        "rationale": "MajorUpgrade version range may include newer versions if not constrained. This could unintall a newer version.",
        "fix_suggestion": "Set MigrateFeatures=\"yes\" and verify version range doesn't include newer versions.",
        "auto_fixable": 0
    },
    "UPG002": {
        "rationale": "Downgrades may lose data or cause incompatibility. Preventing them protects users from accidental data loss.",
        "fix_suggestion": "Add MajorUpgrade with DowngradeErrorMessage to block downgrades.",
        "auto_fixable": 1
    },
    "UPG003": {
        "rationale": "Version gaps in upgrade ranges leave some versions unable to upgrade. They become orphaned installations.",
        "fix_suggestion": "Ensure upgrade ranges are continuous with no gaps between versions.",
        "auto_fixable": 0
    },
    "UPG004": {
        "rationale": "RemoveExistingProducts late in the sequence may leave old files. Early removal is cleaner but slower.",
        "fix_suggestion": "Review upgrade schedule. Early removal ensures clean upgrade but takes longer.",
        "auto_fixable": 0
    },
    "UPG005": {
        "rationale": "Small updates without version bumps may not trigger proper upgrade behavior. Version should change for any update.",
        "fix_suggestion": "Increment at least the build number for any package changes.",
        "auto_fixable": 0
    },
    "UPG006": {
        "rationale": "Changing UpgradeCode creates a new product family. Old versions won't be detected or upgraded.",
        "fix_suggestion": "Keep UpgradeCode constant across versions. Only change for truly separate product lines.",
        "auto_fixable": 0
    },
}

def main():
    """Update rules in the database with rationale and fix_suggestion."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    updated = 0
    for rule_id, data in RULE_DATA.items():
        cursor.execute("""
            UPDATE rules
            SET rationale = ?, fix_suggestion = ?, auto_fixable = ?
            WHERE rule_id = ?
        """, (data["rationale"], data["fix_suggestion"], data.get("auto_fixable", 0), rule_id))
        if cursor.rowcount > 0:
            updated += 1

    conn.commit()

    # Verify
    cursor.execute("SELECT COUNT(*) FROM rules WHERE rationale IS NOT NULL AND rationale != ''")
    with_rationale = cursor.fetchone()[0]
    cursor.execute("SELECT COUNT(*) FROM rules WHERE fix_suggestion IS NOT NULL AND fix_suggestion != ''")
    with_fix = cursor.fetchone()[0]
    cursor.execute("SELECT COUNT(*) FROM rules WHERE auto_fixable = 1")
    auto_fixable = cursor.fetchone()[0]

    print(f"Updated {updated} rules")
    print(f"Rules with rationale: {with_rationale}")
    print(f"Rules with fix_suggestion: {with_fix}")
    print(f"Auto-fixable rules: {auto_fixable}")

    conn.close()

if __name__ == "__main__":
    main()
