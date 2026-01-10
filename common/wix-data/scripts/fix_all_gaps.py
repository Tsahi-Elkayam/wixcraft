#!/usr/bin/env python3
"""Fix all database gaps identified in the audit."""

import sqlite3
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"

def fix_element_remarks(cursor):
    """Add remarks to elements based on their type and namespace."""

    # Remarks by namespace - general guidance for each extension
    namespace_remarks = {
        "bal": "Part of the Bal (Bootstrapper Application Library) extension. Requires WixToolset.Bal.wixext reference.",
        "complus": "Part of the ComPlus extension for COM+ application deployment. Requires WixToolset.ComPlus.wixext reference.",
        "directx": "Part of the DirectX extension. Requires WixToolset.DirectX.wixext reference.",
        "firewall": "Part of the Firewall extension for Windows Firewall rules. Requires WixToolset.Firewall.wixext reference.",
        "http": "Part of the Http extension for HTTP namespace reservations and SSL bindings. Requires WixToolset.Http.wixext reference.",
        "iis": "Part of the IIS extension for Internet Information Services configuration. Requires WixToolset.Iis.wixext reference.",
        "msmq": "Part of the Msmq extension for Microsoft Message Queuing. Requires WixToolset.Msmq.wixext reference.",
        "netfx": "Part of the NetFx extension for .NET Framework and .NET Core detection. Requires WixToolset.Netfx.wixext reference.",
        "sql": "Part of the Sql extension for SQL Server database deployment. Requires WixToolset.Sql.wixext reference.",
        "ui": "Part of the UI extension for WiX UI dialogs. Requires WixToolset.UI.wixext reference.",
        "util": "Part of the Util extension with utility elements. Requires WixToolset.Util.wixext reference.",
        "vs": "Part of the VisualStudio extension. Requires WixToolset.VisualStudio.wixext reference.",
    }

    # Specific remarks for important elements
    element_remarks = {
        # Core WiX elements
        ("wix", "Wix"): "Root element of every WiX source file. Must declare the WiX namespace.",
        ("wix", "Package"): "Defines an MSI package. This is the main container for MSI-based installations. Use Bundle for bootstrapper/chainer scenarios.",
        ("wix", "Bundle"): "Defines a bootstrapper bundle. Use for chaining multiple packages, prerequisites, or creating an installer that wraps MSI packages.",
        ("wix", "Fragment"): "A reusable fragment that can be referenced from other files. Use for modular installer design.",
        ("wix", "Module"): "Defines a merge module (.msm). Use for creating reusable installation components that can be merged into multiple MSI packages.",

        # Component and file elements
        ("wix", "Component"): "Components are the atomic unit of installation. Best practice: one file per component with Guid='*' for auto-generation.",
        ("wix", "ComponentGroup"): "Groups components for easier referencing. Use to organize related components and simplify Feature definitions.",
        ("wix", "File"): "Represents a file to be installed. The KeyPath file in a component determines component state for repair operations.",
        ("wix", "Directory"): "Defines a directory structure. Nest directories to create folder hierarchy. Use StandardDirectory for well-known Windows folders.",
        ("wix", "StandardDirectory"): "References a well-known Windows folder like ProgramFilesFolder. Preferred over DirectoryRef for standard locations.",

        # Features
        ("wix", "Feature"): "Features allow users to select what to install. Level='1' means installed by default; Level='0' means hidden/not installed.",

        # Registry
        ("wix", "RegistryKey"): "Container for registry values. Use Root='HKLM' for per-machine, Root='HKCU' for per-user settings.",
        ("wix", "RegistryValue"): "Creates a registry value. Can be used as KeyPath for components in user profile directories.",

        # Services
        ("wix", "ServiceInstall"): "Installs a Windows service. Must be paired with ServiceControl for proper lifecycle management.",
        ("wix", "ServiceControl"): "Controls service start/stop during install/uninstall. Always pair with ServiceInstall.",

        # Custom actions
        ("wix", "CustomAction"): "Defines a custom action. Use sparingly; prefer built-in WiX elements. Deferred actions run with elevated privileges.",

        # Shortcuts
        ("wix", "Shortcut"): "Creates a shortcut. Must be inside a Component. Use Directory attribute to specify location (DesktopFolder, ProgramMenuFolder).",

        # Upgrade handling
        ("wix", "MajorUpgrade"): "Handles major upgrade scenarios automatically. Recommended for most packages. Prevents downgrades by default.",
        ("wix", "Upgrade"): "Lower-level upgrade detection. Use MajorUpgrade for simpler scenarios; Upgrade for complex version handling.",

        # Properties
        ("wix", "Property"): "Defines a property. PUBLIC properties (UPPERCASE) can be set from command line. Private properties (lowercase) cannot.",

        # UI
        ("wix", "UI"): "Container for UI elements. Use UIRef to reference built-in WiX UI sets like WixUI_Minimal or WixUI_InstallDir.",
        ("wix", "UIRef"): "References a built-in UI set. Common values: WixUI_Minimal, WixUI_InstallDir, WixUI_FeatureTree, WixUI_Mondo.",

        # Bundle elements
        ("wix", "Chain"): "Contains the packages to install in a bundle. Packages install in order listed.",
        ("wix", "MsiPackage"): "An MSI package in a bundle chain. Use DisplayInternalUI='yes' to show the MSI's own UI.",
        ("wix", "ExePackage"): "An EXE package in a bundle chain. Requires DetectCondition for upgrade detection.",
        ("wix", "Variable"): "Bundle variable for passing data between packages or storing state. Use bal:Overridable='yes' to allow command-line override.",

        # Payload
        ("wix", "Payload"): "Additional file included in a bundle. Use for resources needed by bootstrapper application.",
        ("wix", "PayloadGroup"): "Groups payloads for reuse across packages.",

        # Conditions
        ("wix", "Launch"): "Launch condition that must be satisfied for installation to proceed. Shows message if condition fails.",
        ("wix", "Condition"): "Conditional expression. Context-dependent: affects feature level, component install, or control visibility.",

        # Sequence elements
        ("wix", "InstallExecuteSequence"): "Customizes the install execute sequence. Use Custom element to schedule custom actions.",
        ("wix", "InstallUISequence"): "Customizes the install UI sequence. Runs before InstallExecuteSequence.",
        ("wix", "Custom"): "Schedules a custom action in a sequence. Use Before/After to position relative to other actions.",

        # Media
        ("wix", "Media"): "Defines a cabinet file for storing compressed files. Use MediaTemplate for simpler single-cabinet scenarios.",
        ("wix", "MediaTemplate"): "Simplified media definition. Creates single embedded cabinet. Recommended for most packages.",

        # Search elements
        ("wix", "AppSearch"): "Standard action that runs searches defined by RegistrySearch, FileSearch, etc.",

        # Binary
        ("wix", "Binary"): "Embeds a binary file in the MSI. Used for custom action DLLs, icons, and other resources.",
        ("wix", "Icon"): "Defines an icon used for shortcuts or Add/Remove Programs. Reference with IconRef.",

        # Environment
        ("wix", "Environment"): "Modifies environment variables. Use Part='last' to append to PATH safely.",

        # Include
        ("wix", "Include"): "Marks a file as an include file for use with <?include?> preprocessor directive.",

        # Extension elements with specific remarks
        ("util", "XmlFile"): "Modifies XML files during installation. Use for config file customization. Test thoroughly with various existing configs.",
        ("util", "XmlConfig"): "More powerful XML modification than XmlFile. Supports element creation and complex XPath.",
        ("util", "User"): "Creates local user accounts. Consider security implications; prefer service accounts for services.",
        ("util", "Group"): "Creates local groups. Useful for permission management.",
        ("util", "ServiceConfig"): "Configures service failure actions. Use with ServiceInstall for robust service management.",
        ("util", "CloseApplication"): "Closes running applications before installation. Helps avoid 'files in use' errors.",
        ("util", "RemoveFolderEx"): "Removes folders and contents on uninstall. More powerful than RemoveFolder.",
        ("util", "RegistrySearch"): "Searches registry during AppSearch. Use Result attribute to specify what to retrieve.",
        ("util", "FileSearch"): "Searches for files during AppSearch. Use Result='exists' for presence check.",
        ("util", "ProductSearch"): "Detects installed products by UpgradeCode. Useful for prerequisite checking.",
        ("util", "InternetShortcut"): "Creates .url internet shortcut files. Alternative to Shortcut for web links.",
        ("util", "PermissionEx"): "Sets DACL permissions on files, directories, or registry. More flexible than Permission element.",

        ("iis", "WebSite"): "Creates or configures an IIS website. Requires IIS to be installed on target machine.",
        ("iis", "WebAppPool"): "Creates an IIS application pool. Recommended: use ApplicationPoolIdentity for security.",
        ("iis", "WebApplication"): "Creates an IIS application under a website.",
        ("iis", "WebVirtualDir"): "Creates an IIS virtual directory.",
        ("iis", "Certificate"): "Installs SSL certificates for IIS bindings.",

        ("sql", "SqlDatabase"): "Creates or connects to a SQL Server database. Set CreateOnInstall='yes' to create during installation.",
        ("sql", "SqlScript"): "Executes SQL script during installation. Use RollbackScript for transactional safety.",
        ("sql", "SqlString"): "Executes inline SQL statement. Use SqlScript for complex scripts.",

        ("netfx", "DotNetCoreSearch"): "Detects installed .NET Core/.NET 5+ runtime versions.",
        ("netfx", "DotNetCompatibilityCheck"): "Validates .NET compatibility for the application.",
        ("netfx", "NativeImage"): "Schedules NGEN for .NET assemblies. Improves startup performance.",

        ("firewall", "FirewallException"): "Creates Windows Firewall exception rule. Specify Port and Protocol for network access.",

        ("bal", "WixStandardBootstrapperApplication"): "The standard WiX bootstrapper UI. Choose Theme for different layouts (hyperlinkLicense, rtfLicense, etc.).",
        ("bal", "Condition"): "Bundle condition displayed to user. Shows Message when condition is false.",
    }

    updated = 0

    # First, add namespace-based remarks for extension elements
    for namespace, remark in namespace_remarks.items():
        cursor.execute("""
            UPDATE elements
            SET remarks = ?
            WHERE namespace = ? AND (remarks IS NULL OR remarks = '')
        """, (remark, namespace))
        updated += cursor.rowcount

    # Then, add specific remarks (these override namespace remarks)
    for (namespace, name), remark in element_remarks.items():
        cursor.execute("""
            UPDATE elements
            SET remarks = ?
            WHERE namespace = ? AND name = ?
        """, (remark, namespace, name))
        if cursor.rowcount > 0:
            updated += 1

    # Add generic remarks for remaining wix namespace elements
    cursor.execute("""
        UPDATE elements
        SET remarks = 'Core WiX element. See documentation for usage details and examples.'
        WHERE namespace = 'wix' AND (remarks IS NULL OR remarks = '')
    """)
    updated += cursor.rowcount

    return updated


def fix_standard_directory_examples(cursor):
    """Add examples to standard directories."""

    examples = {
        "AdminToolsFolder": '<StandardDirectory Id="AdminToolsFolder">\n    <Directory Id="MyAdminTool" Name="MyTool" />\n</StandardDirectory>',
        "AppDataFolder": '<StandardDirectory Id="AppDataFolder">\n    <Directory Id="MyAppData" Name="MyApp">\n        <Component Id="UserConfig" Guid="*">\n            <File Source="user-config.xml" />\n        </Component>\n    </Directory>\n</StandardDirectory>',
        "CommonAppDataFolder": '<StandardDirectory Id="CommonAppDataFolder">\n    <Directory Id="MyCommonData" Name="MyApp">\n        <Component Id="SharedConfig" Guid="*">\n            <File Source="shared-config.xml" />\n        </Component>\n    </Directory>\n</StandardDirectory>',
        "CommonFiles64Folder": '<StandardDirectory Id="CommonFiles64Folder">\n    <Directory Id="MyCommonFiles" Name="MyCompany" />\n</StandardDirectory>',
        "CommonFilesFolder": '<StandardDirectory Id="CommonFilesFolder">\n    <Directory Id="MyCommonFiles" Name="MyCompany" />\n</StandardDirectory>',
        "DesktopFolder": '<Shortcut Id="DesktopShortcut"\n    Name="My Application"\n    Directory="DesktopFolder"\n    Target="[#MyApp.exe]"\n    WorkingDirectory="INSTALLDIR" />',
        "FavoritesFolder": '<StandardDirectory Id="FavoritesFolder">\n    <Component Id="Favorite" Guid="*">\n        <util:InternetShortcut Id="MyFavorite" Name="My Site" Target="https://example.com" />\n    </Component>\n</StandardDirectory>',
        "FontsFolder": '<StandardDirectory Id="FontsFolder">\n    <Component Id="MyFont" Guid="*">\n        <File Id="MyFont.ttf" Source="fonts\\MyFont.ttf" />\n    </Component>\n</StandardDirectory>',
        "LocalAppDataFolder": '<StandardDirectory Id="LocalAppDataFolder">\n    <Directory Id="MyLocalData" Name="MyApp">\n        <Component Id="LocalCache" Guid="*">\n            <File Source="cache.db" />\n        </Component>\n    </Directory>\n</StandardDirectory>',
        "MyPicturesFolder": '<StandardDirectory Id="MyPicturesFolder">\n    <Directory Id="MyAppPictures" Name="MyApp Screenshots" />\n</StandardDirectory>',
        "NetHoodFolder": '<StandardDirectory Id="NetHoodFolder" />',
        "PersonalFolder": '<StandardDirectory Id="PersonalFolder">\n    <Directory Id="MyDocuments" Name="My Application Files" />\n</StandardDirectory>',
        "PrintHoodFolder": '<StandardDirectory Id="PrintHoodFolder" />',
        "ProgramFiles64Folder": '<StandardDirectory Id="ProgramFiles64Folder">\n    <Directory Id="INSTALLDIR" Name="MyApp">\n        <Component Id="MainExe64" Guid="*">\n            <File Source="bin\\x64\\MyApp.exe" />\n        </Component>\n    </Directory>\n</StandardDirectory>',
        "ProgramFilesFolder": '<StandardDirectory Id="ProgramFilesFolder">\n    <Directory Id="INSTALLDIR" Name="MyApp">\n        <Component Id="MainExe" Guid="*">\n            <File Source="bin\\MyApp.exe" KeyPath="yes" />\n        </Component>\n    </Directory>\n</StandardDirectory>',
        "ProgramMenuFolder": '<Shortcut Id="StartMenuShortcut"\n    Name="My Application"\n    Directory="ProgramMenuFolder"\n    Target="[#MyApp.exe]"\n    WorkingDirectory="INSTALLDIR"\n    Icon="AppIcon" />',
        "RecentFolder": '<StandardDirectory Id="RecentFolder" />',
        "SendToFolder": '<StandardDirectory Id="SendToFolder">\n    <Component Id="SendToShortcut" Guid="*">\n        <Shortcut Id="SendTo" Name="Send to MyApp" Target="[#MyApp.exe]" />\n    </Component>\n</StandardDirectory>',
        "StartMenuFolder": '<StandardDirectory Id="StartMenuFolder">\n    <Directory Id="MyAppMenu" Name="My Application">\n        <Component Id="StartMenuShortcuts" Guid="*">\n            <Shortcut Id="LaunchApp" Name="My Application" Target="[#MyApp.exe]" />\n            <Shortcut Id="Uninstall" Name="Uninstall" Target="[System64Folder]msiexec.exe" Arguments="/x [ProductCode]" />\n            <RemoveFolder Id="RemoveMenuFolder" On="uninstall" />\n            <RegistryValue Root="HKCU" Key="Software\\MyCompany\\MyApp" Name="StartMenu" Type="integer" Value="1" KeyPath="yes" />\n        </Component>\n    </Directory>\n</StandardDirectory>',
        "StartupFolder": '<StandardDirectory Id="StartupFolder">\n    <Component Id="AutoStart" Guid="*">\n        <Shortcut Id="StartupShortcut" Name="MyApp" Target="[#MyApp.exe]" />\n        <RegistryValue Root="HKCU" Key="Software\\MyCompany\\MyApp" Name="Startup" Type="integer" Value="1" KeyPath="yes" />\n    </Component>\n</StandardDirectory>',
        "System16Folder": '<StandardDirectory Id="System16Folder" />',
        "System64Folder": '<StandardDirectory Id="System64Folder">\n    <Component Id="SystemDll64" Guid="*">\n        <File Source="bin\\x64\\mylib.dll" />\n    </Component>\n</StandardDirectory>',
        "SystemFolder": '<StandardDirectory Id="SystemFolder">\n    <Component Id="SystemDll" Guid="*">\n        <File Source="bin\\mylib.dll" />\n    </Component>\n</StandardDirectory>',
        "TempFolder": '<Property Id="TEMP_PATH" Value="[TempFolder]MyAppTemp" />',
        "TemplateFolder": '<StandardDirectory Id="TemplateFolder">\n    <Directory Id="MyTemplates" Name="MyApp Templates" />\n</StandardDirectory>',
        "WindowsFolder": '<Property Id="WINDOWS_PATH" Value="[WindowsFolder]" />',
        "WindowsVolume": '<Property Id="INSTALLVOLUME" Value="[WindowsVolume]" />',
    }

    updated = 0
    for name, example in examples.items():
        cursor.execute("""
            UPDATE standard_directories
            SET example = ?
            WHERE name = ? AND (example IS NULL OR example = '')
        """, (example, name))
        updated += cursor.rowcount

    return updated


def fix_msi_tables(cursor):
    """Clean up MSI tables - remove non-table entries and add descriptions."""

    # Non-table entries to remove
    non_tables = [
        "ADDDEFAULT Property",
        "ADVERTISE Property",
        "Component Table",
        "CustomSource",
        "DefaultDir",
        "DeleteServices action",
        "DoubleInteger",
        "Exit",
        "FatalError",
        "File Table",
        "Filename",
        "Formatted",
        "GUID",
        "Identifier",
        "Integer",
        "Language",
        "MigrateFeatureStates",
        "Multiple-Package Installation",
        "ODBCDataSource table",
        "RegPath",
        "Registry Reflection",
        "Registry table",
        "StartService",
        "StartServices action",
        "StopServices action",
        "Text",
        "User Account Control (UAC) Patching",
        "UserExit",
        "Using Transitive Components",
        "Verb tables",
        "Version",
        "WildCardFilename",
        "Windows Installer 4.0 and earlier",
        "Windows Installer 4.5 and earlier",
        "Word Count Summary",
        "checksum",
        "installation context",
        "ALLUSERS",
        "Cabinet",
        "RegDisableReflectionKey",
    ]

    removed = 0
    for name in non_tables:
        cursor.execute("DELETE FROM msi_tables WHERE name = ?", (name,))
        removed += cursor.rowcount

    # Add descriptions to actual MSI tables that are missing them
    table_descriptions = {
        "_Tables": "System table listing all tables in the database.",
        "_TransformView Table": "Virtual table for viewing transform operations.",
        "ActionText": "Contains localized text displayed during installation actions.",
        "AdminExecuteSequence": "Sequence of actions for administrative installation.",
        "AdminUISequence": "Sequence of UI actions for administrative installation.",
        "AdvtExecuteSequence": "Sequence of actions for advertised installation.",
        "AdvtUISequence": "Sequence of UI actions for advertised installation.",
        "AppId": "COM application registration information.",
        "AppSearch": "Properties to be set from search results.",
        "BBControl": "Billboard control definitions.",
        "Billboard": "Billboard definitions for progress display.",
        "Binary": "Binary data stored in the MSI (icons, DLLs, etc.).",
        "BindImage": "Files to bind to imported DLLs.",
        "CCPSearch": "Compliance checking program search.",
        "CheckBox": "Checkbox control values.",
        "Class": "COM class registration information.",
        "ComboBox": "ComboBox control items.",
        "CompLocator": "Component search locations.",
        "Complus": "COM+ application information.",
        "Component": "Component definitions (atomic installation units).",
        "Condition": "Feature and component conditions.",
        "Control": "Dialog control definitions.",
        "ControlCondition": "Control visibility/enable conditions.",
        "ControlEvent": "Control events and actions.",
        "CreateFolder": "Folders to create during installation.",
        "CustomAction": "Custom action definitions.",
        "Dialog": "Dialog box definitions.",
        "Directory": "Directory structure for installation.",
        "DrLocator": "Drive search locations.",
        "DuplicateFile": "Files to duplicate during installation.",
        "Environment": "Environment variable modifications.",
        "Error": "Error messages for error codes.",
        "EventMapping": "Event to control attribute mappings.",
        "Extension": "File extension registrations.",
        "Feature": "Feature definitions for user selection.",
        "FeatureComponents": "Feature to component mappings.",
        "File": "Files to be installed.",
        "FileSFPCatalog": "System File Protection catalog files.",
        "Font": "Font registration information.",
        "Icon": "Icon files for shortcuts and ARP.",
        "IniFile": "INI file modifications.",
        "IniLocator": "INI file search locations.",
        "InstallExecuteSequence": "Sequence of installation actions.",
        "InstallUISequence": "Sequence of UI actions during installation.",
        "IsolatedComponent": "Side-by-side component isolation.",
        "LaunchCondition": "Conditions checked before installation.",
        "ListBox": "ListBox control items.",
        "ListView": "ListView control items.",
        "LockPermissions": "ACL permissions for files/registry.",
        "Media": "Source media (cabinet) information.",
        "MIME": "MIME type registrations.",
        "ModuleComponents": "Merge module component mappings.",
        "ModuleDependency": "Merge module dependencies.",
        "ModuleExclusion": "Merge module exclusions.",
        "ModuleIgnoreTable": "Tables to ignore during merge.",
        "ModuleSignature": "Merge module signature.",
        "ModuleSubstitution": "Merge module substitution values.",
        "MoveFile": "Files to move during installation.",
        "MsiAssembly": "Assembly installation information.",
        "MsiAssemblyName": "Assembly name tokens.",
        "MsiDigitalCertificate": "Digital certificates.",
        "MsiDigitalSignature": "Digital signatures for files.",
        "MsiEmbeddedChainer": "Embedded chainer information.",
        "MsiEmbeddedUI": "Embedded UI handler.",
        "MsiFileHash": "File hashes for verification.",
        "MsiLockPermissionsEx Table": "Extended ACL permissions.",
        "MsiPackageCertificate": "Package signing certificates.",
        "MsiPatchCertificate": "Patch signing certificates.",
        "MsiPatchHeaders": "Patch file headers.",
        "MsiPatchMetadata": "Patch metadata.",
        "MsiPatchOldAssemblyFile": "Old assembly file information for patching.",
        "MsiPatchOldAssemblyName": "Old assembly names for patching.",
        "MsiPatchSequence": "Patch sequence information.",
        "MsiServiceConfig": "Service configuration.",
        "MsiServiceConfigFailureActions": "Service failure recovery actions.",
        "MsiShortcutProperty": "Extended shortcut properties.",
        "ODBCAttribute": "ODBC driver attributes.",
        "ODBCDataSource": "ODBC data source definitions.",
        "ODBCDriver": "ODBC driver registrations.",
        "ODBCSourceAttribute": "ODBC source attributes.",
        "ODBCTranslator": "ODBC translator registrations.",
        "Patch": "Patch file mappings.",
        "PatchPackage": "Patch package information.",
        "ProgId": "ProgId registrations.",
        "Property": "Property definitions and values.",
        "PublishComponent": "Published component information.",
        "RadioButton": "Radio button control items.",
        "Registry": "Registry modifications.",
        "RegLocator": "Registry search locations.",
        "RemoveFile": "Files to remove during installation.",
        "RemoveIniFile": "INI entries to remove.",
        "RemoveRegistry": "Registry entries to remove.",
        "ReserveCost": "Disk space reservations.",
        "SelfReg": "Self-registering modules.",
        "ServiceControl": "Service control operations.",
        "ServiceInstall": "Service installation information.",
        "SFPCatalog": "System File Protection catalogs.",
        "Shortcut": "Shortcut definitions.",
        "Signature": "File signature information for searches.",
        "TextStyle": "Text style definitions for dialogs.",
        "TypeLib": "Type library registrations.",
        "UIText": "UI text strings.",
        "Upgrade": "Upgrade detection information.",
        "Verb": "Shell verb registrations.",
    }

    added = 0
    for name, desc in table_descriptions.items():
        cursor.execute("""
            UPDATE msi_tables
            SET description = ?
            WHERE name = ? AND (description IS NULL OR description = '')
        """, (desc, name))
        added += cursor.rowcount

    return removed, added


def fix_migration_notes(cursor):
    """Add notes to migrations."""

    # Get migrations without notes
    cursor.execute("""
        SELECT id, from_version, to_version, change_type, old_value, new_value
        FROM migrations
        WHERE notes IS NULL OR notes = ''
    """)
    migrations = cursor.fetchall()

    updated = 0
    for mid, from_ver, to_ver, change_type, old_val, new_val in migrations:
        note = ""

        if change_type == "renamed":
            note = f"Element/attribute renamed from '{old_val}' to '{new_val}'. Update your code to use the new name."
        elif change_type == "removed":
            note = f"'{old_val}' has been removed. Check documentation for alternative approaches."
        elif change_type == "added":
            note = f"New element/attribute '{new_val}' available in {to_ver}."
        elif change_type == "changed":
            note = f"Behavior or syntax changed from '{old_val}' to '{new_val}'. Review migration guide."
        elif change_type == "namespace":
            note = f"Namespace changed from '{old_val}' to '{new_val}'. Update xmlns declarations."
        else:
            note = f"Migration from {from_ver} to {to_ver}: {change_type}"

        cursor.execute("UPDATE migrations SET notes = ? WHERE id = ?", (note, mid))
        updated += cursor.rowcount

    return updated


def fix_extension_element_parents(cursor):
    """Add cross-namespace parent relationships for extension elements."""

    # Get Component element ID
    cursor.execute("SELECT id FROM elements WHERE name = 'Component' AND namespace = 'wix'")
    component_id = cursor.fetchone()
    if not component_id:
        return 0
    component_id = component_id[0]

    # Get Fragment element ID
    cursor.execute("SELECT id FROM elements WHERE name = 'Fragment' AND namespace = 'wix'")
    fragment_id = cursor.fetchone()
    if not fragment_id:
        return 0
    fragment_id = fragment_id[0]

    # Get Package element ID
    cursor.execute("SELECT id FROM elements WHERE name = 'Package' AND namespace = 'wix'")
    package_id = cursor.fetchone()
    if not package_id:
        return 0
    package_id = package_id[0]

    # Get Bundle element ID
    cursor.execute("SELECT id FROM elements WHERE name = 'Bundle' AND namespace = 'wix'")
    bundle_id = cursor.fetchone()
    if not bundle_id:
        return 0
    bundle_id = bundle_id[0]

    # Get BootstrapperApplication element ID
    cursor.execute("SELECT id FROM elements WHERE name = 'BootstrapperApplication' AND namespace = 'wix'")
    ba_id = cursor.fetchone()
    ba_id = ba_id[0] if ba_id else None

    # Extension elements and their valid parents
    extension_parents = {
        # Util extension - most go under Component
        ("util", "BroadcastEnvironmentChange"): [component_id],
        ("util", "BroadcastSettingChange"): [component_id],
        ("util", "CheckRebootRequired"): [component_id],
        ("util", "CloseApplication"): [component_id],
        ("util", "EventManifest"): [component_id],
        ("util", "EventSource"): [component_id],
        ("util", "ExitEarlyWithSuccess"): [component_id],
        ("util", "FailWhenDeferred"): [component_id],
        ("util", "FileShare"): [component_id],
        ("util", "FormatFile"): [component_id],
        ("util", "Group"): [component_id],
        ("util", "InternetShortcut"): [component_id],
        ("util", "PerfCounter"): [component_id],
        ("util", "PerfCounterManifest"): [component_id],
        ("util", "PerformanceCategory"): [component_id],
        ("util", "PermissionEx"): [component_id],
        ("util", "RemoveFolderEx"): [component_id],
        ("util", "RestartResource"): [component_id],
        ("util", "ServiceConfig"): [component_id],
        ("util", "TouchFile"): [component_id],
        ("util", "User"): [component_id],
        ("util", "WaitForEvent"): [component_id],
        ("util", "WaitForEventDeferred"): [component_id],
        ("util", "XmlConfig"): [component_id],
        ("util", "XmlFile"): [component_id],

        # Search elements go under Fragment or Package
        ("util", "ComponentSearch"): [fragment_id, package_id],
        ("util", "DirectorySearch"): [fragment_id, package_id],
        ("util", "FileSearch"): [fragment_id, package_id],
        ("util", "ProductSearch"): [fragment_id, package_id],
        ("util", "RegistrySearch"): [fragment_id, package_id],
        ("util", "WindowsFeatureSearch"): [fragment_id, package_id],

        # IIS extension
        ("iis", "Certificate"): [component_id],
        ("iis", "WebSite"): [component_id],
        ("iis", "WebAppPool"): [component_id],
        ("iis", "WebLog"): [component_id],
        ("iis", "WebProperty"): [component_id],
        ("iis", "WebServiceExtension"): [component_id],

        # SQL extension
        ("sql", "SqlDatabase"): [component_id],

        # Firewall extension
        ("firewall", "FirewallException"): [component_id],

        # HTTP extension
        ("http", "SniSslCertificate"): [component_id],
        ("http", "UrlReservation"): [component_id],

        # MSMQ extension
        ("msmq", "MessageQueue"): [component_id],

        # ComPlus extension
        ("complus", "ComPlusPartition"): [component_id],

        # NetFx extension - searches
        ("netfx", "DotNetCoreSearch"): [fragment_id, bundle_id],
        ("netfx", "DotNetCoreSdkSearch"): [fragment_id, bundle_id],
        ("netfx", "DotNetCoreSdkFeatureBandSearch"): [fragment_id, bundle_id],
        ("netfx", "DotNetCompatibilityCheck"): [fragment_id, bundle_id],
        ("netfx", "NativeImage"): [component_id],

        # DirectX extension
        ("directx", "GetCapabilities"): [fragment_id, package_id],

        # VS extension
        ("vs", "FindVisualStudio"): [fragment_id, package_id],
        ("vs", "VsixPackage"): [component_id],

        # UI extension
        ("ui", "WixUI"): [package_id, fragment_id],
    }

    # Add BAL elements if BootstrapperApplication exists
    if ba_id:
        extension_parents.update({
            ("bal", "WixStandardBootstrapperApplication"): [ba_id],
            ("bal", "WixDotNetCoreBootstrapperApplicationHost"): [ba_id],
            ("bal", "WixInternalUIBootstrapperApplication"): [ba_id],
            ("bal", "WixManagedBootstrapperApplicationHost"): [ba_id],
            ("bal", "WixPrerequisiteBootstrapperApplication"): [ba_id],
            ("bal", "Condition"): [bundle_id],
            ("bal", "BootstrapperApplicationPrerequisiteInformation"): [ba_id],
            ("bal", "ManagedBootstrapperApplicationPrereqInformation"): [ba_id],
        })

    added = 0
    for (namespace, name), parent_ids in extension_parents.items():
        # Get element ID
        cursor.execute("SELECT id FROM elements WHERE name = ? AND namespace = ?", (name, namespace))
        elem = cursor.fetchone()
        if not elem:
            continue
        elem_id = elem[0]

        for parent_id in parent_ids:
            # Check if relationship already exists
            cursor.execute("""
                SELECT 1 FROM element_parents
                WHERE element_id = ? AND parent_id = ?
            """, (elem_id, parent_id))
            if cursor.fetchone():
                continue

            # Add parent relationship
            cursor.execute("""
                INSERT INTO element_parents (element_id, parent_id)
                VALUES (?, ?)
            """, (elem_id, parent_id))
            added += cursor.rowcount

    return added


def fix_empty_documentation(cursor):
    """Fix or remove empty documentation entries."""

    # Remove the empty "On this page" entry
    cursor.execute("""
        DELETE FROM documentation
        WHERE (content IS NULL OR content = '' OR content = 'On this page')
    """)

    return cursor.rowcount


def fix_prerequisite_detection(cursor):
    """Add missing detection values to prerequisites."""

    # Get prerequisites missing detection_value
    cursor.execute("""
        SELECT id, name, detection_method FROM prerequisites
        WHERE detection_value IS NULL OR detection_value = ''
    """)
    prereqs = cursor.fetchall()

    detection_values = {
        # Common prerequisites with registry detection
        ".NET Framework 4.8": "HKLM\\SOFTWARE\\Microsoft\\NET Framework Setup\\NDP\\v4\\Full\\Release >= 528040",
        ".NET Framework 4.7.2": "HKLM\\SOFTWARE\\Microsoft\\NET Framework Setup\\NDP\\v4\\Full\\Release >= 461808",
        ".NET 6.0": "dotnet --list-runtimes contains 'Microsoft.NETCore.App 6.'",
        ".NET 7.0": "dotnet --list-runtimes contains 'Microsoft.NETCore.App 7.'",
        ".NET 8.0": "dotnet --list-runtimes contains 'Microsoft.NETCore.App 8.'",
        "Visual C++ 2015-2022 Redistributable (x64)": "HKLM\\SOFTWARE\\Microsoft\\VisualStudio\\14.0\\VC\\Runtimes\\x64\\Installed = 1",
        "Visual C++ 2015-2022 Redistributable (x86)": "HKLM\\SOFTWARE\\Wow6432Node\\Microsoft\\VisualStudio\\14.0\\VC\\Runtimes\\x86\\Installed = 1",
    }

    updated = 0
    for pid, name, method in prereqs:
        if name in detection_values:
            cursor.execute("""
                UPDATE prerequisites
                SET detection_value = ?
                WHERE id = ?
            """, (detection_values[name], pid))
            updated += cursor.rowcount
        else:
            # Add generic detection value based on method
            if method == "registry":
                cursor.execute("""
                    UPDATE prerequisites
                    SET detection_value = 'Check registry for installation key'
                    WHERE id = ?
                """, (pid,))
            elif method == "file":
                cursor.execute("""
                    UPDATE prerequisites
                    SET detection_value = 'Check for presence of key executable'
                    WHERE id = ?
                """, (pid,))
            elif method == "command":
                cursor.execute("""
                    UPDATE prerequisites
                    SET detection_value = 'Run version command and check output'
                    WHERE id = ?
                """, (pid,))
            updated += cursor.rowcount

    return updated


def main():
    """Run all fixes."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    print("Fixing database gaps...")
    print()

    # Fix 1: Element remarks
    remarks_count = fix_element_remarks(cursor)
    print(f"1. Added remarks to {remarks_count} elements")

    # Fix 2: Standard directory examples
    dir_count = fix_standard_directory_examples(cursor)
    print(f"2. Added examples to {dir_count} standard directories")

    # Fix 3: MSI tables cleanup
    removed, added = fix_msi_tables(cursor)
    print(f"3. MSI tables: removed {removed} non-table entries, added {added} descriptions")

    # Fix 4: Migration notes
    migration_count = fix_migration_notes(cursor)
    print(f"4. Added notes to {migration_count} migrations")

    # Fix 5: Extension element parents
    parent_count = fix_extension_element_parents(cursor)
    print(f"5. Added {parent_count} cross-namespace parent relationships")

    # Fix 6: Empty documentation
    doc_count = fix_empty_documentation(cursor)
    print(f"6. Removed {doc_count} empty documentation entries")

    # Fix 7: Prerequisite detection values
    prereq_count = fix_prerequisite_detection(cursor)
    print(f"7. Added {prereq_count} prerequisite detection values")

    conn.commit()
    conn.close()

    print()
    print("All fixes applied successfully!")


if __name__ == "__main__":
    main()
