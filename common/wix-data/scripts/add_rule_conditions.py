#!/usr/bin/env python3
"""Add rule_conditions for rules that are missing them."""

import sqlite3
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"

# Rule conditions organized by rule_id
# Format: rule_id -> [(condition_type, target, operator, value), ...]
RULE_CONDITIONS = {
    # Bundle rules (BNDL004-BNDL012)
    "BNDL004": [
        ("element", "Variable", "exists", None),
        ("attribute", "Variable/@Name", "matches", "^(WixBundleAction|WixBundleInstalled|WixBundleProviderKey|WixBundleTag|WixBundleVersion|WixBundleName|WixBundleManufacturer|WixBundleOriginalSource|WixBundleOriginalSourceFolder|WixBundleLastUsedSource|WixBundleElevated)$"),
    ],
    "BNDL005": [
        ("element", "MsiPackage|ExePackage|MspPackage|MsuPackage", "exists", None),
        ("attribute", "*Package/@Vital", "equals", "no"),
    ],
    "BNDL006": [
        ("element", "Payload", "exists", None),
        ("attribute", "Payload/@SourceFile", "not_exists", None),
    ],
    "BNDL007": [
        ("element", "Payload|*Package", "exists", None),
        ("attribute", "*/@DownloadUrl", "exists", None),
        ("attribute", "*/@Hash", "not_exists", None),
    ],
    "BNDL008": [
        ("element", "RollbackBoundary", "exists", None),
        ("pattern", "RollbackBoundary position in Chain", "matches", "not-first"),
    ],
    "BNDL009": [
        ("element", "ExePackage", "exists", None),
        ("attribute", "ExePackage/@RepairCommand", "not_exists", None),
    ],
    "BNDL010": [
        ("element", "MsiPackage", "exists", None),
        ("attribute", "MsiPackage/@DisplayInternalUI", "equals", "no"),
    ],
    "BNDL011": [
        ("element", "MsuPackage", "exists", None),
        ("attribute", "MsuPackage/@KB", "not_exists", None),
    ],
    "BNDL012": [
        ("element", "PackageGroupRef", "exists", None),
        ("pattern", "PackageGroup references", "matches", "circular"),
    ],

    # Component rules (COMP010-COMP015)
    "COMP010": [
        ("element", "Component", "exists", None),
        ("pattern", "Component Id referenced by Feature", "not_exists", None),
    ],
    "COMP011": [
        ("element", "Component", "exists", None),
        ("attribute", "Component/@Directory", "exists", None),
        ("pattern", "Component/@Directory value", "matches", "invalid-ref"),
    ],
    "COMP012": [
        ("element", "Component", "exists", None),
        ("xpath", "Component[not(*)]", "exists", None),
    ],
    "COMP013": [
        ("element", "Component", "exists", None),
        ("attribute", "Component/@Win64", "equals", "yes"),
        ("attribute", "Package/@Platform", "equals", "x86"),
    ],
    "COMP014": [
        ("element", "Component", "exists", None),
        ("attribute", "Component/@Id", "matches", "^.{73,}$"),
    ],
    "COMP015": [
        ("element", "Component", "exists", None),
        ("attribute", "Component/@Transitive", "equals", "yes"),
        ("attribute", "Component/@Condition", "not_exists", None),
    ],

    # Custom action rules (CA005-CA015)
    "CA005": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@DllEntry", "exists", None),
        ("pattern", "DllEntry value", "matches", "undefined"),
    ],
    "CA006": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@Execute", "equals", "deferred"),
        ("attribute", "CustomAction/@Impersonate", "equals", "yes"),
    ],
    "CA007": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@BinaryRef", "exists", None),
        ("pattern", "BinaryRef target", "not_exists", None),
    ],
    "CA008": [
        ("element", "CustomAction", "exists", None),
        ("pattern", "Custom action scheduled in sequence", "not_exists", None),
    ],
    "CA009": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@Execute", "equals", "immediate"),
        ("pattern", "CustomAction scheduled before InstallFiles", "equals", "true"),
    ],
    "CA010": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@Return", "exists", None),
        ("pattern", "Return type mismatch", "matches", "invalid"),
    ],
    "CA011": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@Return", "equals", "asyncWait"),
    ],
    "CA012": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@Execute", "exists", None),
        ("pattern", "Execute type vs sequence", "matches", "mismatch"),
    ],
    "CA013": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@Property", "exists", None),
        ("attribute", "CustomAction/@Value", "not_exists", None),
    ],
    "CA014": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@Execute", "equals", "deferred"),
        ("pattern", "Deferred CA reads session property", "matches", "true"),
    ],
    "CA015": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@Script", "equals", "powershell"),
    ],

    # Deprecated rules (DEPR003-DEPR006)
    "DEPR003": [
        ("element", "Product|Module", "exists", None),
        ("attribute", "Wix/@xmlns", "matches", "schemas.microsoft.com/wix/2006"),
    ],
    "DEPR004": [
        ("attribute", "*/@xmlns", "matches", "schemas.microsoft.com/wix/(2006|200[0-5])"),
    ],
    "DEPR005": [
        ("element", "Product", "exists", None),
    ],
    "DEPR006": [
        ("element", "DirectoryRef", "exists", None),
        ("attribute", "DirectoryRef/@Id", "matches", "^(TARGETDIR|ProgramFilesFolder|ProgramFiles64Folder|CommonFilesFolder|CommonFiles64Folder|SystemFolder|System64Folder|WindowsFolder|TempFolder)$"),
    ],

    # Directory rules (DIR006-DIR010)
    "DIR006": [
        ("element", "Directory", "exists", None),
        ("attribute", "Directory/@Name", "matches", "\\s"),
    ],
    "DIR007": [
        ("element", "StandardDirectory|Directory", "exists", None),
        ("attribute", "StandardDirectory/@Id", "not_equals", "ProgramFilesFolder"),
        ("attribute", "StandardDirectory/@Id", "not_equals", "ProgramFiles64Folder"),
    ],
    "DIR008": [
        ("element", "Directory", "exists", None),
        ("xpath", "Directory[not(.//File)]", "exists", None),
        ("element", "CreateFolder", "not_exists", None),
    ],
    "DIR009": [
        ("element", "StandardDirectory", "exists", None),
        ("attribute", "StandardDirectory/@Id", "matches", "^(?!(TARGETDIR|ProgramFilesFolder|ProgramFiles64Folder|CommonFilesFolder|CommonFiles64Folder|SystemFolder|System64Folder|WindowsFolder|TempFolder|DesktopFolder|StartMenuFolder|ProgramMenuFolder|AppDataFolder|LocalAppDataFolder|PersonalFolder|FontsFolder|SendToFolder|StartupFolder|FavoritesFolder|TemplateFolder)).*$"),
    ],
    "DIR010": [
        ("element", "Directory|RemoveFolder", "exists", None),
        ("attribute", "RemoveFolder/@On", "equals", "uninstall"),
        ("pattern", "Directory only removed on uninstall", "matches", "true"),
    ],

    # Extension rules (EXT001-EXT008)
    "EXT001": [
        ("element", "iis:WebSite", "exists", None),
        ("xpath", "iis:WebSite[not(iis:WebAddress)]", "exists", None),
    ],
    "EXT002": [
        ("element", "iis:WebAppPool", "exists", None),
        ("attribute", "iis:WebAppPool/@Identity", "equals", "localSystem"),
    ],
    "EXT003": [
        ("element", "sql:SqlDatabase", "exists", None),
        ("attribute", "sql:SqlDatabase/@ContinueOnError", "equals", "yes"),
    ],
    "EXT004": [
        ("element", "util:User", "exists", None),
        ("attribute", "util:User/@Domain", "exists", None),
        ("pattern", "Domain account permissions", "matches", "elevated"),
    ],
    "EXT005": [
        ("element", "netfx:NativeImage", "exists", None),
        ("attribute", "netfx:NativeImage/@Platform", "exists", None),
        ("pattern", "NetFx version availability", "matches", "unavailable"),
    ],
    "EXT006": [
        ("element", "firewall:FirewallException", "exists", None),
        ("attribute", "firewall:FirewallException/@Protocol", "not_exists", None),
    ],
    "EXT007": [
        ("element", "util:XmlFile|util:XmlConfig", "exists", None),
        ("attribute", "*/@Action", "equals", "setValue"),
        ("pattern", "XML transformation complexity", "matches", "complex"),
    ],
    "EXT008": [
        ("element", "Environment", "exists", None),
        ("attribute", "Environment/@Name", "equals", "PATH"),
        ("attribute", "Environment/@Action", "equals", "set"),
    ],

    # Feature rules (FEAT004-FEAT010)
    "FEAT004": [
        ("element", "Feature", "exists", None),
        ("attribute", "Feature/@Id", "matches", "^.{39,}$"),
    ],
    "FEAT005": [
        ("element", "FeatureRef", "exists", None),
        ("pattern", "FeatureRef references", "matches", "circular"),
    ],
    "FEAT006": [
        ("element", "Feature", "exists", None),
        ("xpath", "Feature//Feature//Feature//Feature", "exists", None),
    ],
    "FEAT007": [
        ("element", "Feature", "exists", None),
        ("pattern", "Feature count", "matches", ">32"),
    ],
    "FEAT008": [
        ("element", "Feature", "exists", None),
        ("attribute", "Feature/@Description", "not_exists", None),
    ],
    "FEAT009": [
        ("element", "Feature", "exists", None),
        ("attribute", "Feature/@Display", "equals", "hidden"),
        ("xpath", "Feature[@Display='hidden'][ComponentRef]", "exists", None),
    ],
    "FEAT010": [
        ("element", "Package|Product", "exists", None),
        ("xpath", "//Feature[not(ancestor::Feature)]", "not_exists", None),
    ],

    # File rules (FILE007-FILE015)
    "FILE007": [
        ("element", "File", "exists", None),
        ("attribute", "File/@Source", "matches", "\\.(dll|exe)$"),
        ("attribute", "File/@Assembly", "exists", None),
        ("pattern", "Assembly manifest", "not_exists", None),
    ],
    "FILE008": [
        ("element", "File", "exists", None),
        ("attribute", "File/@Source", "matches", "\\.dll$"),
        ("pattern", "DLL version info", "not_exists", None),
    ],
    "FILE009": [
        ("element", "File", "exists", None),
        ("attribute", "File/@Name", "matches", "[<>:\"|?*]"),
    ],
    "FILE010": [
        ("element", "File", "exists", None),
        ("attribute", "File/@Source", "matches", "^.{260,}$"),
    ],
    "FILE011": [
        ("element", "File", "exists", None),
        ("attribute", "File/@Source", "matches", "\\.exe$"),
        ("attribute", "File/@Checksum", "not_exists", None),
    ],
    "FILE012": [
        ("element", "File", "exists", None),
        ("attribute", "File/@Source", "matches", "\\.(ttf|otf|fon|ttc)$"),
        ("pattern", "Font not in FontsFolder", "matches", "true"),
    ],
    "FILE013": [
        ("element", "File", "exists", None),
        ("pattern", "File size", "matches", ">50MB"),
    ],
    "FILE014": [
        ("element", "File", "exists", None),
        ("attribute", "File/@Source", "matches", "\\.pdb$"),
    ],
    "FILE015": [
        ("element", "File", "exists", None),
        ("attribute", "File/@Source", "matches", "\\.(tmp|temp|bak|log)$"),
    ],

    # Localization rules (LOC001-LOC004)
    "LOC001": [
        ("element", "Control|Dialog", "exists", None),
        ("attribute", "*/@Text", "matches", "^[^!\\(\\[]"),
        ("pattern", "Text not localized", "matches", "true"),
    ],
    "LOC002": [
        ("element", "String", "exists", None),
        ("attribute", "String/@Id", "exists", None),
        ("pattern", "Missing translation for culture", "matches", "true"),
    ],
    "LOC003": [
        ("element", "String", "exists", None),
        ("attribute", "String/@Id", "matches", "^[^A-Z]|[^a-zA-Z0-9_]"),
    ],
    "LOC004": [
        ("element", "String", "exists", None),
        ("attribute", "String/@Id", "exists", None),
        ("pattern", "String Id not referenced", "matches", "true"),
    ],

    # Naming rules (NAME001-NAME005)
    "NAME001": [
        ("attribute", "*/@Id", "matches", ".*"),
        ("pattern", "Naming convention inconsistent", "matches", "true"),
    ],
    "NAME002": [
        ("attribute", "*/@Id", "matches", "^(ALLUSERS|ARPAUTHORIZEDCDFPREFIX|ARPCOMMENTS|ARPCONTACT|ARPINSTALLLOCATION|ARPNOMODIFY|ARPNOREMOVE|ARPNOREPAIR|ARPPRODUCTICON|ARPREADME|ARPSIZE|ARPSYSTEMCOMPONENT|ARPURLINFOABOUT|ARPURLUPDATEINFO|PRIMARYFOLDER|Manufacturer|ProductCode|ProductLanguage|ProductName|ProductVersion|UpgradeCode)$"),
    ],
    "NAME003": [
        ("attribute", "*/@Id", "matches", "^(Id[0-9]+|Temp[0-9]*|Test[0-9]*|Foo|Bar|Baz|Component[0-9]+|Feature[0-9]+|File[0-9]+)$"),
    ],
    "NAME004": [
        ("attribute", "*/@Id", "matches", "^[0-9]"),
    ],
    "NAME005": [
        ("attribute", "*/@Id", "matches", "^[a-z].*[A-Z]|^[A-Z].*[a-z].*[A-Z]"),
    ],

    # Package rules (PKG005-PKG015)
    "PKG005": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@Manufacturer", "not_exists", None),
    ],
    "PKG006": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@Name", "not_exists", None),
    ],
    "PKG007": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@InstallerVersion", "matches", "^[0-2][0-9]{0,2}$"),
    ],
    "PKG008": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@Compressed", "equals", "no"),
    ],
    "PKG009": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@Language", "not_exists", None),
    ],
    "PKG010": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@ProductCode", "matches", "^(?!\\{?[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\\}?$)"),
    ],
    "PKG011": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@UpgradeCode", "exists", None),
        ("attribute", "Package/@ProductCode", "exists", None),
        ("pattern", "UpgradeCode equals ProductCode", "matches", "true"),
    ],
    "PKG012": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@Version", "matches", "^[0-9]+\\.[0-9]+\\.[0-9]+\\.[1-9]"),
    ],
    "PKG013": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@Scope", "equals", "perUser"),
    ],
    "PKG014": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@Codepage", "not_exists", None),
    ],
    "PKG015": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@AdminImage", "equals", "yes"),
    ],

    # Performance rules (PERF004-PERF008)
    "PERF004": [
        ("element", "Media|MediaTemplate", "exists", None),
        ("attribute", "Media/@Cabinet", "not_exists", None),
    ],
    "PERF005": [
        ("element", "CustomAction", "exists", None),
        ("pattern", "CustomAction count", "matches", ">20"),
    ],
    "PERF006": [
        ("element", "Custom", "exists", None),
        ("xpath", "InstallUISequence//Custom", "exists", None),
        ("attribute", "CustomAction/@Return", "not_equals", "asyncNoWait"),
    ],
    "PERF007": [
        ("element", "Feature", "exists", None),
        ("xpath", "Feature//Feature//Feature//Feature//Feature", "exists", None),
    ],
    "PERF008": [
        ("element", "Binary", "exists", None),
        ("pattern", "Binary table size", "matches", ">10MB"),
    ],

    # Property rules (PROP004-PROP010)
    "PROP004": [
        ("element", "Property", "exists", None),
        ("attribute", "Property/@Value", "matches", "^.{256,}$"),
    ],
    "PROP005": [
        ("element", "Property", "exists", None),
        ("attribute", "Property/@Id", "exists", None),
        ("pattern", "Property not referenced", "matches", "true"),
    ],
    "PROP006": [
        ("element", "Property", "exists", None),
        ("attribute", "Property/@Id", "matches", "[^a-zA-Z0-9_.]"),
    ],
    "PROP007": [
        ("element", "Property", "exists", None),
        ("attribute", "Property/@Id", "matches", "^(Installed|REINSTALL|REINSTALLMODE|ADDLOCAL|ADDSOURCE|ADDDEFAULT|REMOVE|ADVERTISE|FILEADDLOCAL|FILEADDSOURCE|FILEADDDEFAULT|FILEREMOVE|COMPADDLOCAL|COMPADDSOURCE|COMPADDDEFAULT)$"),
    ],
    "PROP008": [
        ("element", "Property", "exists", None),
        ("attribute", "Property/@Admin", "exists", None),
        ("pattern", "Admin value differs from default", "matches", "true"),
    ],
    "PROP009": [
        ("element", "Property", "exists", None),
        ("attribute", "Property/@Id", "matches", "(PASSWORD|SECRET|KEY|TOKEN|CREDENTIAL|API_KEY)"),
    ],
    "PROP010": [
        ("element", "Condition", "exists", None),
        ("pattern", "Property used in condition not defined", "matches", "true"),
    ],

    # Registry rules (REG004-REG010)
    "REG004": [
        ("element", "RegistryKey|RegistryValue", "exists", None),
        ("attribute", "*/@Key", "matches", "^.{256,}$"),
    ],
    "REG005": [
        ("element", "RegistryValue", "exists", None),
        ("attribute", "RegistryValue/@Name", "matches", "\\\\"),
    ],
    "REG006": [
        ("element", "RegistryKey|RegistryValue", "exists", None),
        ("attribute", "*/@Root", "equals", "HKCR"),
    ],
    "REG007": [
        ("element", "RegistryValue", "exists", None),
        ("attribute", "RegistryValue/@Type", "equals", "expandable"),
        ("attribute", "RegistryValue/@Value", "not_matches", "%[^%]+%"),
    ],
    "REG008": [
        ("element", "RegistryValue", "exists", None),
        ("attribute", "RegistryValue/@Action", "equals", "write"),
        ("pattern", "Registry value exists", "matches", "true"),
    ],
    "REG009": [
        ("element", "RegistryValue", "exists", None),
        ("attribute", "RegistryValue/@Type", "matches", "^(?!(string|integer|binary|expandable|multiString)$)"),
    ],
    "REG010": [
        ("element", "RegistryKey|RegistryValue", "exists", None),
        ("attribute", "*/@Key", "matches", "^(SOFTWARE\\\\Microsoft\\\\Windows\\\\CurrentVersion\\\\Run|SOFTWARE\\\\Microsoft\\\\Windows NT\\\\CurrentVersion|SYSTEM\\\\CurrentControlSet)"),
    ],

    # Security rules (SEC005-SEC012)
    "SEC005": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@DllEntry", "exists", None),
        ("pattern", "DLL search order vulnerability", "matches", "true"),
    ],
    "SEC006": [
        ("attribute", "*/@DownloadUrl|*/@SourceFile", "matches", "^http://"),
    ],
    "SEC007": [
        ("element", "RegistryKey", "exists", None),
        ("xpath", "RegistryKey[Permission]", "exists", None),
        ("attribute", "Permission/@GenericAll", "equals", "yes"),
    ],
    "SEC008": [
        ("element", "ServiceInstall", "exists", None),
        ("attribute", "ServiceInstall/@Account", "matches", "(LocalSystem|SYSTEM|Administrator)"),
    ],
    "SEC009": [
        ("element", "firewall:FirewallException", "exists", None),
        ("attribute", "firewall:FirewallException/@Scope", "equals", "any"),
    ],
    "SEC010": [
        ("element", "File|Binary", "exists", None),
        ("attribute", "*/@Source", "matches", "\\.(exe|dll|msi|msp)$"),
        ("pattern", "File not signed", "matches", "true"),
    ],
    "SEC011": [
        ("element", "CustomAction", "exists", None),
        ("attribute", "CustomAction/@Script", "matches", "(powershell|vbscript|jscript)"),
    ],
    "SEC012": [
        ("element", "Directory", "exists", None),
        ("attribute", "Directory/@Id", "matches", "CommonAppDataFolder"),
        ("xpath", "Directory[Permission[@GenericWrite='yes']]", "exists", None),
    ],

    # Service rules (SVC004-SVC010)
    "SVC004": [
        ("element", "ServiceInstall", "exists", None),
        ("attribute", "ServiceInstall/@Account", "matches", "(LocalSystem|SYSTEM)"),
    ],
    "SVC005": [
        ("element", "ServiceInstall", "exists", None),
        ("attribute", "ServiceInstall/@Name", "exists", None),
        ("pattern", "Service executable not found", "matches", "true"),
    ],
    "SVC006": [
        ("element", "ServiceInstall", "exists", None),
        ("attribute", "ServiceInstall/@Description", "not_exists", None),
    ],
    "SVC007": [
        ("element", "ServiceInstall", "exists", None),
        ("attribute", "ServiceInstall/@Dependencies", "exists", None),
        ("pattern", "Service dependencies circular", "matches", "true"),
    ],
    "SVC008": [
        ("element", "ServiceInstall", "exists", None),
        ("attribute", "ServiceInstall/@Start", "equals", "auto"),
        ("element", "ServiceConfig", "not_exists", None),
    ],
    "SVC009": [
        ("element", "ServiceInstall", "exists", None),
        ("attribute", "ServiceInstall/@Account", "matches", "^[^\\[\\$]"),
    ],
    "SVC010": [
        ("element", "ServiceInstall", "exists", None),
        ("attribute", "ServiceInstall/@Interactive", "equals", "yes"),
    ],

    # Shortcut rules (SHRT001-SHRT010)
    "SHRT001": [
        ("element", "Shortcut", "exists", None),
        ("attribute", "Shortcut/@Target", "not_exists", None),
        ("attribute", "Shortcut/@Advertise", "not_equals", "yes"),
    ],
    "SHRT002": [
        ("element", "Shortcut", "exists", None),
        ("attribute", "Shortcut/@Directory", "matches", "^(?!(DesktopFolder|ProgramMenuFolder|StartMenuFolder|StartupFolder|SendToFolder))"),
    ],
    "SHRT003": [
        ("element", "Shortcut", "exists", None),
        ("attribute", "Shortcut/@Icon", "not_exists", None),
        ("attribute", "Shortcut/@IconIndex", "not_exists", None),
    ],
    "SHRT004": [
        ("element", "Shortcut", "exists", None),
        ("attribute", "Shortcut/@WorkingDirectory", "exists", None),
        ("pattern", "WorkingDirectory reference invalid", "matches", "true"),
    ],
    "SHRT005": [
        ("element", "Shortcut", "exists", None),
        ("attribute", "Shortcut/@Target", "matches", "\\s"),
    ],
    "SHRT006": [
        ("element", "Shortcut", "exists", None),
        ("attribute", "Shortcut/@Advertise", "equals", "yes"),
        ("attribute", "Package/@Scope", "equals", "perUser"),
    ],
    "SHRT007": [
        ("element", "Shortcut", "exists", None),
        ("pattern", "Multiple shortcuts to same target", "matches", "true"),
    ],
    "SHRT008": [
        ("element", "Shortcut", "exists", None),
        ("attribute", "Shortcut/@Name", "matches", "[<>:\"|?*\\\\]"),
    ],
    "SHRT009": [
        ("element", "Shortcut", "exists", None),
        ("attribute", "Shortcut/@Directory", "equals", "DesktopFolder"),
        ("xpath", "Shortcut[not(ancestor::Component/Condition)]", "exists", None),
    ],
    "SHRT010": [
        ("element", "Shortcut", "exists", None),
        ("attribute", "Shortcut/@Target", "matches", "^https?://"),
        ("attribute", "Shortcut/@Icon", "not_exists", None),
    ],

    # UI rules (UI003-UI010)
    "UI003": [
        ("element", "Control", "exists", None),
        ("pattern", "Control rectangles overlap", "matches", "true"),
    ],
    "UI004": [
        ("element", "Dialog", "exists", None),
        ("xpath", "Dialog[not(Control[@Type='PushButton'][@Cancel='yes'])]", "exists", None),
    ],
    "UI005": [
        ("element", "Control", "exists", None),
        ("attribute", "Control/@Type", "equals", "Text"),
        ("pattern", "Text exceeds control width", "matches", "true"),
    ],
    "UI006": [
        ("element", "Dialog", "exists", None),
        ("attribute", "Dialog/@DefaultAction", "exists", None),
        ("pattern", "DefaultAction control not found", "matches", "true"),
    ],
    "UI007": [
        ("element", "Control", "exists", None),
        ("attribute", "Control/@Type", "equals", "Bitmap"),
        ("attribute", "Control/@Bitmap", "exists", None),
        ("pattern", "Bitmap file not found", "matches", "true"),
    ],
    "UI008": [
        ("element", "Control", "exists", None),
        ("attribute", "Control/@Type", "equals", "Bitmap"),
        ("pattern", "Bitmap file size", "matches", ">500KB"),
    ],
    "UI009": [
        ("element", "UIRef", "exists", None),
        ("xpath", "Package[UIRef][not(.//Dialog)]", "exists", None),
    ],
    "UI010": [
        ("element", "ProgressText", "exists", None),
        ("attribute", "ProgressText/@Action", "exists", None),
        ("pattern", "ProgressText missing for action", "matches", "true"),
    ],

    # Upgrade rules (UPG001-UPG006)
    "UPG001": [
        ("element", "Upgrade", "exists", None),
        ("attribute", "UpgradeVersion/@Maximum", "exists", None),
        ("attribute", "UpgradeVersion/@IncludeMaximum", "equals", "yes"),
    ],
    "UPG002": [
        ("element", "Package", "exists", None),
        ("element", "MajorUpgrade", "not_exists", None),
        ("xpath", "Upgrade/UpgradeVersion[@OnlyDetect='yes']", "not_exists", None),
    ],
    "UPG003": [
        ("element", "MajorUpgrade|Upgrade", "exists", None),
        ("pattern", "Upgrade version gap exists", "matches", "true"),
    ],
    "UPG004": [
        ("element", "MajorUpgrade", "exists", None),
        ("attribute", "MajorUpgrade/@Schedule", "matches", "(afterInstallExecute|afterInstallFinalize)"),
    ],
    "UPG005": [
        ("element", "Package", "exists", None),
        ("pattern", "Small update without version change", "matches", "true"),
    ],
    "UPG006": [
        ("element", "Package", "exists", None),
        ("attribute", "Package/@UpgradeCode", "exists", None),
        ("pattern", "UpgradeCode differs from previous version", "matches", "true"),
    ],
}


def main():
    """Add rule conditions for rules missing them."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    # Get existing rule IDs that have conditions
    cursor.execute("SELECT DISTINCT rule_id FROM rule_conditions")
    existing = set(row[0] for row in cursor.fetchall())

    # Get all rules and map rule_id text to database id
    cursor.execute("SELECT id, rule_id FROM rules")
    rule_id_map = {row[1]: row[0] for row in cursor.fetchall()}

    added = 0
    for rule_id, conditions in RULE_CONDITIONS.items():
        if rule_id not in rule_id_map:
            print(f"Warning: Rule {rule_id} not found in database")
            continue

        db_id = rule_id_map[rule_id]
        if db_id in existing:
            continue

        for condition_type, target, operator, value in conditions:
            cursor.execute("""
                INSERT INTO rule_conditions (rule_id, condition_type, target, operator, value)
                VALUES (?, ?, ?, ?, ?)
            """, (db_id, condition_type, target, operator, value))
            added += 1

    conn.commit()

    # Verify
    cursor.execute("SELECT COUNT(DISTINCT rule_id) FROM rule_conditions")
    covered = cursor.fetchone()[0]

    cursor.execute("SELECT COUNT(*) FROM rules")
    total = cursor.fetchone()[0]

    print(f"Added {added} conditions")
    print(f"Rules with conditions: {covered}/{total}")

    # Check remaining uncovered
    cursor.execute("""
        SELECT rule_id FROM rules
        WHERE id NOT IN (SELECT DISTINCT rule_id FROM rule_conditions)
        ORDER BY rule_id
    """)
    uncovered = [row[0] for row in cursor.fetchall()]
    if uncovered:
        print(f"Still uncovered: {len(uncovered)}")
        for r in uncovered[:10]:
            print(f"  - {r}")

    conn.close()


if __name__ == "__main__":
    main()
