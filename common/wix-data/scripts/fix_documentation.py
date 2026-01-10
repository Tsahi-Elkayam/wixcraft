#!/usr/bin/env python3
"""Fix documentation entries that have scraping artifacts."""

import sqlite3
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "wix.db"

# Replacement content for documentation entries with bad scrapes
DOCUMENTATION_FIXES = {
    ("64bit-support", "concepts"): """Windows Installer supports both 32-bit and 64-bit installations. Key considerations:

- Use Win64="yes" on Component elements for 64-bit components
- 64-bit installers require Windows Installer 5.0 (InstallerVersion="500")
- Set Platform="x64" or Platform="arm64" in Package element
- 64-bit registry writes go to HKLM\\Software (not Wow6432Node)
- ProgramFiles64Folder and System64Folder for 64-bit directories
- Cannot mix 32-bit and 64-bit components in the same package""",

    ("advertising", "concepts"): """Advertised installation makes applications available without fully installing them:

- Users can install on first use
- Start menu shortcuts trigger installation when clicked
- File associations launch installation when document opened
- Reduces initial installation time and disk space
- Server-based advertising for network deployments
- Use Advertise="yes" on Feature elements to enable""",

    ("commit-actions", "concepts"): """Commit custom actions run only when installation completes successfully:

- Execute after InstallFinalize
- Not reversible - no rollback action possible
- Use for actions that should only happen on successful install
- Set Execute="commit" on CustomAction element
- Commit actions cannot read session properties
- Run in same context as installation (deferred)""",

    ("component-rules", "best_practices"): """Component best practices for reliable installations:

- One file per component improves patching granularity
- Use Guid="*" for auto-generated component GUIDs
- Every component needs a KeyPath (file or registry)
- Components cannot span multiple directories
- Permanent components need explicit stable GUIDs
- Don't mix per-user and per-machine resources
- Keep component IDs under 72 characters""",

    ("conditional-syntax", "syntax"): """Windows Installer conditional expression syntax:

Operators:
- AND, OR, NOT - logical operators
- =, <>, <, >, <=, >= - comparison
- >< (contains), << (starts with), >> (ends with) - string

Properties:
- PROPERTY - true if property is set
- PROPERTY = "value" - value comparison
- Installed - true if product is installed

Examples:
- NOT Installed - first-time install only
- REINSTALL OR UPGRADINGPRODUCTCODE - upgrade scenario
- VersionNT >= 603 - Windows 8.1 or later""",

    ("custom-action-types", "custom_action_types"): """Custom action types in Windows Installer:

Type 1: DLL in Binary table
Type 2: EXE in Binary table
Type 5: JScript in Binary table
Type 6: VBScript in Binary table
Type 17: DLL in installed file
Type 18: EXE in installed file
Type 21: JScript in installed file
Type 22: VBScript in installed file
Type 34: EXE with path in directory property
Type 35: Directory set from formatted string
Type 37: JScript text
Type 38: VBScript text
Type 50: EXE with path in property
Type 51: Property set from formatted string""",

    ("database-object", "api"): """Windows Installer Database object (VBScript/JScript):

Methods:
- OpenView(sql) - Opens a view on the database
- Commit() - Commits changes to the database
- CreateTransformSummaryInfo() - Creates transform summary

Properties:
- SummaryInformation - Returns summary info object
- TablePersistent(table) - Checks if table is persistent

Example:
dim db, view
set db = installer.OpenDatabase("my.msi", 0)
set view = db.OpenView("SELECT * FROM Property")
view.Execute""",

    ("database-transforms", "concepts"): """Database transforms modify MSI databases without changing the original:

- .mst files store differences between two databases
- Apply at install time: msiexec /i product.msi TRANSFORMS=custom.mst
- Customize language, features, properties
- Create with WiX or MsiDatabaseGenerateTransform API
- Embed in MSI with Media element
- Transforms can add, modify, or delete table rows
- Must be validated against original package""",

    ("deferred-execution", "concepts"): """Deferred custom actions run with elevated privileges during installation:

- Run between InstallInitialize and InstallFinalize
- Execute in SYSTEM context when elevated
- Cannot read session properties directly
- Use CustomActionData property to pass values
- Must have corresponding rollback action for safety
- Set Execute="deferred" on CustomAction element
- Impersonate="yes" runs as installing user instead""",

    ("digital-signatures", "security"): """Digital signatures for Windows Installer packages:

- Sign MSI files with Authenticode
- Prevents tampering after signing
- Required for Windows SmartScreen trust
- Use signtool.exe from Windows SDK
- Cabinet files can be signed separately
- Bundle bootstrappers should also be signed
- Timestamp signatures for long-term validity
- EV certificates provide immediate reputation""",

    ("error-messages", "msi_errors"): """Common Windows Installer error messages:

1601: Windows Installer service could not be accessed
1602: User cancelled installation
1603: Fatal error during installation
1605: This action is only valid for products currently installed
1618: Another installation is already in progress
1619: Installation package could not be opened
1625: Installation prohibited by system policy
1638: Another version is already installed
2755: Server returned unexpected error
2869: Could not update environment variable""",

    ("featureinfo-object", "api"): """FeatureInfo object in Windows Installer API:

Properties:
- Name - Feature identifier
- Description - Feature description
- Title - Feature title for UI
- Parent - Parent feature name
- Attributes - Feature attributes bitmask
- CurrentState - Current installation state

States:
- INSTALLSTATE_UNKNOWN (-1)
- INSTALLSTATE_ABSENT (2)
- INSTALLSTATE_LOCAL (3)
- INSTALLSTATE_SOURCE (4)
- INSTALLSTATE_DEFAULT (5)""",

    ("file-versioning", "concepts"): """File versioning rules in Windows Installer:

- Versioned files: higher version wins
- Unversioned files: modification date comparison
- Use DefaultVersion on File element for unversioned files
- Companion files inherit versioning from parent
- MsiFileHash for unversioned file validation
- REINSTALLMODE property controls replacement behavior
- "o" = reinstall only if older, "a" = reinstall all
- Version format: major.minor.build.revision""",

    ("ice-reference", "ice_rules"): """ICE (Internal Consistency Evaluators) validate MSI databases:

ICE validation categories:
- ICE01-ICE99: General validation rules
- ICE errors prevent installation
- ICE warnings indicate potential issues

Common ICEs:
- ICE03: Invalid identifier names
- ICE06: Missing primary keys
- ICE09: Component in multiple features must be permanent
- ICE30: Duplicate file paths
- ICE33: Missing file sources
- ICE57: Component with multiple directories""",

    ("install-sequences", "sequences"): """Windows Installer action sequences:

InstallUISequence: UI actions before execution
- AppSearch, LaunchConditions
- CostInitialize, FileCost, CostFinalize
- ExecuteAction (triggers InstallExecuteSequence)

InstallExecuteSequence: Installation actions
- InstallValidate
- InstallInitialize
- ProcessComponents, InstallFiles
- RegisterProduct
- InstallFinalize

AdminExecuteSequence: Administrative install
AdvtExecuteSequence: Advertisement""",

    ("installation-context", "concepts"): """Installation context determines per-user or per-machine installation:

Per-machine (AllUsers=1):
- Requires elevation
- Installs to ProgramFilesFolder
- Writes to HKLM registry
- Available to all users

Per-user (AllUsers="" or InstallScope="perUser"):
- No elevation required
- Installs to LocalAppDataFolder
- Writes to HKCU registry
- Current user only

ALLUSERS property controls context in WiX v3
InstallScope attribute controls context in WiX v4""",

    ("installer-functions", "functions"): """Windows Installer API functions:

File operations:
- MsiInstallProduct - Install a product
- MsiApplyPatch - Apply a patch
- MsiReinstallProduct - Reinstall a product
- MsiRemoveProduct - Remove a product

Query functions:
- MsiQueryProductState - Get product state
- MsiGetProductInfo - Get product properties
- MsiEnumProducts - Enumerate installed products
- MsiEnumFeatures - Enumerate product features

Configuration:
- MsiConfigureProduct - Configure installation state
- MsiSetInternalUI - Set UI level""",

    ("installer-object", "api"): """Windows Installer Installer object (VBScript/JScript):

Methods:
- InstallProduct(packagePath, commandLine)
- OpenDatabase(path, mode)
- OpenPackage(packagePath, options)
- CreateRecord(count)
- ProductState(productCode)
- ProductInfo(productCode, property)

Properties:
- UILevel - Current UI level
- Products - Collection of installed products
- Version - Windows Installer version

Example:
Set installer = CreateObject("WindowsInstaller.Installer")
installer.InstallProduct "product.msi", "PROPERTY=value" """,

    ("installer-versions", "versions"): """Windows Installer versions and Windows releases:

2.0: Windows XP, Windows Server 2003
3.0: Windows XP SP2
3.1: Windows Server 2003 SP1
4.0: Windows Vista
4.5: Windows Vista SP2, Windows Server 2008
5.0: Windows 7, Windows Server 2008 R2

Check with InstallerVersion attribute in Package element
WiX v4 requires InstallerVersion="500"
Use VersionNT property for Windows version checks""",

    ("logging", "debugging"): """Windows Installer logging for debugging:

Enable logging:
msiexec /i product.msi /l*v install.log

Log flags:
- v: Verbose output
- x: Extra debugging
- e: Error messages
- w: Warning messages
- a: Action sequences
- r: Action-specific records
- p: Property values

Global logging registry key:
HKLM\\Software\\Policies\\Microsoft\\Windows\\Installer
Set Logging="voicewarmup" for detailed logs""",

    ("major-upgrades", "best_practices"): """Major upgrade best practices:

- Use MajorUpgrade element in WiX v4
- Keep UpgradeCode constant across versions
- Change ProductCode for each major version
- Set DowngradeErrorMessage for user feedback
- Schedule RemoveExistingProducts appropriately:
  - afterInstallValidate: Clean upgrade, slower
  - afterInstallExecute: Files stay in place
  - afterInstallFinalize: Fastest, riskiest

Include MigrateFeatures="yes" to preserve selections""",

    ("merge-modules", "concepts"): """Merge modules (.msm) bundle redistributable components:

- Self-contained installation fragments
- Include with Merge element in WiX
- Components get unique GUIDs at merge time
- Module ID prevents conflicts
- Dependencies included automatically
- Microsoft VC++ runtime uses merge modules
- Alternative: WiX Fragment files for shared code
- Bundle redistributables instead of merge modules""",

    ("minor-upgrades", "best_practices"): """Minor upgrade and small update best practices:

Minor upgrade:
- Same ProductCode, different version
- File changes only, no component changes
- Apply with REINSTALL=ALL REINSTALLMODE=vomus

Small update:
- Same ProductCode and Version
- Bug fixes only
- Apply with REINSTALL and REINSTALLMODE

Patches (.msp):
- Delta updates for any upgrade type
- Created from two MSI versions
- Apply with msiexec /p patch.msp""",

    ("msi-actions", "actions"): """Standard Windows Installer actions:

Early sequence:
- AppSearch: Search for existing applications
- LaunchConditions: Check installation prerequisites
- CostInitialize: Begin file costing
- FileCost: Calculate file space requirements
- CostFinalize: Complete costing

Installation:
- InstallValidate: Verify disk space
- InstallFiles: Copy files to target
- RegisterProduct: Register with Windows
- PublishProduct: Publish to Start menu

Cleanup:
- RemoveFiles: Delete installed files
- RemoveRegistryValues: Delete registry""",

    ("msix-from-msi", "msix"): """Converting MSI to MSIX packages:

MSIX Packaging Tool:
- Capture MSI installation
- Package as MSIX
- Sign with certificate
- Distribute via Store or sideload

Limitations:
- No custom actions (service installs, registry-only)
- No kernel-mode drivers
- Limited registry virtualization
- No COM server registration

Best for:
- Simple file-based applications
- Modern Windows 10/11 deployment
- Store distribution""",

    ("msix-overview", "msix"): """MSIX is the modern Windows application packaging format:

Features:
- Container-based installation
- Clean uninstall guaranteed
- Built-in auto-update
- Reliable, network-efficient distribution
- Strong identity and signing

Compared to MSI:
- No custom actions
- No system-wide registry access
- Sandboxed file access
- Simpler but more restricted
- Better for modern apps

Use MSI for:
- Complex system changes
- Services and drivers
- Enterprise deployment""",

    ("package-validation", "validation"): """Validating MSI packages:

Tools:
- Orca (from Windows SDK)
- ICE validation (built into WiX)
- Wise Package Studio
- InstEd

Run validation:
wix build -validate project.wxs
msiexec /q INSTALLLEVEL=0 /i product.msi

Validation levels:
- Schema validation (XML structure)
- ICE validation (database consistency)
- Installation test (actual install/uninstall)
- Signing verification (Authenticode)""",

    ("patch-packages", "best_practices"): """Patch package (.msp) best practices:

Creation:
- Use torch.exe to generate transform between versions
- pyro.exe creates patch from transform
- WiX Patch element for declarative patching

Structure:
- Patches contain transforms, not full files
- Target specific product versions with Validate element
- Use PATCH property for conditions

Deployment:
- msiexec /p patch.msp
- MsiApplyPatch API
- Can patch multiple products
- Uninstallable if registered""",

    ("patchinfo-object", "api"): """PatchInfo object in Windows Installer API:

Retrieve patch information for installed products.

Properties:
- PatchCode - GUID identifying the patch
- ProductCode - Target product GUID
- LocalPackage - Local cache path
- State - Patch state
- Transforms - Transforms applied

Access via Installer.PatchInfo(patch, product, property)
Use Installer.PatchesEx to enumerate patches""",

    ("productinfo-object", "api"): """ProductInfo in Windows Installer API:

Retrieve information about installed products.

Common properties:
- ProductName - Display name
- ProductVersion - Version string
- Publisher - Manufacturer name
- InstallLocation - Installation directory
- InstallDate - Installation date
- PackageCode - MSI package code
- VersionString - Friendly version

Access via Installer.ProductInfo(productCode, property)
Use Installer.Products to enumerate installed products""",

    ("record-object", "api"): """Record object in Windows Installer API:

Container for field values in database operations.

Methods:
- StringData(field) - Get/set string value
- IntegerData(field) - Get/set integer value
- SetStream(field, path) - Set binary stream
- ReadStream(field, length, format) - Read binary data
- FieldCount - Number of fields

Usage:
Set record = installer.CreateRecord(3)
record.StringData(1) = "Value1"
record.IntegerData(2) = 123
view.Modify 1, record  ' Insert record""",

    ("rollback-actions", "concepts"): """Rollback custom actions restore system state on failure:

- Run when installation fails
- Undo changes made by deferred actions
- Execute="rollback" on CustomAction element
- Schedule before corresponding deferred action
- Cannot read session properties
- Run in reverse order of deferred actions

Best practices:
- Every deferred action should have rollback
- Rollback should be idempotent
- Test failure scenarios thoroughly""",

    ("session-object", "api"): """Session object in Windows Installer API:

Represents an installation session.

Properties:
- Property(name) - Get/set session properties
- ProductProperty(name) - Get product properties
- FeatureCurrentState(name) - Get feature state
- ComponentCurrentState(name) - Get component state
- Language - Session language

Methods:
- Message(kind, record) - Display UI message
- DoAction(action) - Execute custom action
- SetInstallLevel(level) - Set install level
- EvaluateCondition(expression) - Evaluate condition""",

    ("side-by-side", "concepts"): """Side-by-side (SxS) assembly deployment:

- Multiple versions of same DLL coexist
- Avoids DLL Hell
- Application isolation via manifests
- Assembly manifests declare dependencies
- Windows SxS store manages shared assemblies

WiX support:
- Assembly element for manifests
- AssemblyName, AssemblyVersion attributes
- File element with Assembly attribute
- .NET assemblies in GAC are SxS""",

    ("standard-folders", "directories"): """Standard Windows Installer folder properties:

System folders:
- SystemFolder (System32)
- System64Folder (64-bit System32)
- WindowsFolder (Windows directory)
- TempFolder (Temp directory)

Program folders:
- ProgramFilesFolder (Program Files)
- ProgramFiles64Folder (64-bit Program Files)
- CommonFilesFolder (Common Files)
- CommonFiles64Folder (64-bit Common Files)

User folders:
- AppDataFolder (Roaming AppData)
- LocalAppDataFolder (Local AppData)
- PersonalFolder (Documents)
- DesktopFolder (Desktop)
- StartMenuFolder (Start Menu)""",

    ("summary-info", "summary_info"): """MSI Summary Information stream properties:

PID_TEMPLATE (7): Platform;Language
PID_LASTAUTHOR (8): Last author
PID_REVNUMBER (9): Package code GUID
PID_PAGECOUNT (14): Minimum installer version
PID_WORDCOUNT (15): File mode flags
PID_SUBJECT (3): Product name
PID_AUTHOR (4): Manufacturer
PID_COMMENTS (6): Description

Access via SummaryInformation property of Database object
WiX sets these from Package element attributes""",

    ("summaryinfo-object", "api"): """SummaryInfo object in Windows Installer API:

Access and modify MSI summary information stream.

Properties by PID:
- Property(1) - Codepage
- Property(2) - Title
- Property(3) - Subject
- Property(4) - Author
- Property(7) - Template (Platform;Language)
- Property(9) - Package Code
- Property(14) - Page Count (Installer version)
- Property(15) - Word Count (File mode)

Methods:
- Persist() - Save changes
- PropertyCount - Number of properties""",

    ("system-policy", "concepts"): """Windows Installer system policies:

Machine policies (HKLM\\Software\\Policies\\Microsoft\\Windows\\Installer):
- DisableMSI: Block MSI installation
- DisableUserInstalls: Block per-user installs
- EnableAdminTSRemote: Enable admin install via RDP
- AlwaysInstallElevated: Allow elevation (security risk)

Group Policy:
- Turn off Windows Installer
- Always install with elevated privileges
- Prevent removable media installation
- Disable rollback

Policies affect installation behavior system-wide""",

    ("transforms", "concepts"): """MSI transforms (.mst) customize installations:

Creation:
- MsiDatabaseGenerateTransform API
- Compare two MSI versions
- WiX with Patch element

Application:
- TRANSFORMS property: msiexec /i p.msi TRANSFORMS=t.mst
- Multiple transforms: TRANSFORMS=t1.mst;t2.mst
- Embedded with @ prefix: TRANSFORMS=@:embed.mst

Common uses:
- Language customization
- Feature selection defaults
- Property customization
- Organization-specific settings""",

    ("view-object", "api"): """View object in Windows Installer API:

Execute SQL queries against MSI database.

Methods:
- Execute(record) - Execute with parameters
- Fetch() - Get next record
- Modify(mode, record) - Modify record
- Close() - Close view

Modify modes:
- 1: MSIMODIFY_INSERT
- 2: MSIMODIFY_UPDATE
- 4: MSIMODIFY_DELETE
- 16: MSIMODIFY_REPLACE

Example:
Set view = db.OpenView("SELECT * FROM Property WHERE Property='Version'")
view.Execute
Set record = view.Fetch""",
}


def main():
    """Fix documentation entries with scraping artifacts."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    updated = 0
    for (source, topic), content in DOCUMENTATION_FIXES.items():
        cursor.execute("""
            UPDATE documentation
            SET content = ?
            WHERE source = ? AND topic = ?
            AND content LIKE '%Upgrade to Microsoft Edge%'
        """, (content.strip(), source, topic))
        if cursor.rowcount > 0:
            updated += 1

    conn.commit()

    # Verify
    cursor.execute("""
        SELECT COUNT(*) FROM documentation
        WHERE content LIKE '%Upgrade to Microsoft Edge%'
    """)
    still_bad = cursor.fetchone()[0]

    print(f"Updated {updated} documentation entries")
    print(f"Still with scraping artifacts: {still_bad}")

    conn.close()


if __name__ == "__main__":
    main()
