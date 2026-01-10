# WixCraft Data Sources

Reference document for building the WiX data layer.

---

## 1. WiX Schema (XSD Files)

### Primary Sources

| Version | URL | Format |
|---------|-----|--------|
| WiX v4/v5 | `https://raw.githubusercontent.com/wixtoolset/web/master/src/xsd4/wix.xsd` | XSD |
| WiX v3 | `https://wixtoolset.org/docs/v3/xsd/wix/` | HTML docs |
| All schemas | `https://wixtoolset.org/docs/schema/` | HTML docs |

### Extension Schemas

| Extension | Purpose | Schema URL |
|-----------|---------|------------|
| Util | Utility functions, XML config, user management | `/docs/schema/util/` |
| NetFx | .NET Framework detection and installation | `/docs/schema/netfx/` |
| IIS | IIS website and app pool configuration | `/docs/schema/iis/` |
| SQL | SQL Server database creation | `/docs/schema/sql/` |
| Firewall | Windows Firewall exceptions | `/docs/schema/firewall/` |
| Bal | Burn bootstrapper application | `/docs/schema/bal/` |
| Http | HTTP URL reservations | `/docs/schema/http/` |
| Msmq | Message Queue configuration | `/docs/schema/msmq/` |
| Powershell | PowerShell snap-in registration | `/docs/schema/powershell/` |
| ComPlus | COM+ application configuration | `/docs/schema/complus/` |
| Dependency | Package dependency tracking | `/docs/schema/dependency/` |
| DirectX | DirectX detection | `/docs/schema/directx/` |

### How to Extract

```bash
# Download XSD directly
curl -O https://raw.githubusercontent.com/wixtoolset/web/master/src/xsd4/wix.xsd

# Parse XSD to extract elements, attributes, types
# Build JSON database from XSD definitions
```

---

## 2. Windows Installer Properties

### Source
- **Microsoft Learn**: https://learn.microsoft.com/en-us/windows/win32/msi/property-reference

### Categories

| Category | Examples |
|----------|----------|
| Required | ProductCode, ProductName, ProductVersion, Manufacturer, ProductLanguage |
| System Folders | ProgramFilesFolder, SystemFolder, WindowsFolder, TempFolder |
| Feature Selection | ADDLOCAL, REMOVE, REINSTALL, ADVERTISE |
| UI Properties | ARPNOREMOVE, ARPNOMODIFY, ARPSYSTEMCOMPONENT |
| Installation | ALLUSERS, TARGETDIR, INSTALLLEVEL |
| Logging | MsiLogFileLocation, MsiLogging |

### Standard Directories (StandardDirectoryType)

```
TARGETDIR, AdminToolsFolder, AppDataFolder, CommonAppDataFolder,
CommonFilesFolder, CommonFiles64Folder, CommonFiles6432Folder,
DesktopFolder, FavoritesFolder, FontsFolder, LocalAppDataFolder,
MyPicturesFolder, NetHoodFolder, PersonalFolder, PrintHoodFolder,
ProgramFilesFolder, ProgramFiles64Folder, ProgramFiles6432Folder,
ProgramMenuFolder, PerUserProgramFilesFolder, RecentFolder,
SendToFolder, StartMenuFolder, StartupFolder, SystemFolder,
System16Folder, System64Folder, System6432Folder, TempFolder,
TemplateFolder, WindowsFolder, WindowsVolume
```

---

## 3. ICE Validation Rules

### Source
- **Microsoft Learn**: https://learn.microsoft.com/en-us/windows/win32/msi/ice-reference

### Key ICE Rules to Document

| ICE | Purpose | Severity |
|-----|---------|----------|
| ICE01 | Simple ICE used as an example | Info |
| ICE02 | Circular reference check | Error |
| ICE03 | Data type validation | Error |
| ICE04 | File sequence number validation | Error |
| ICE05 | Required properties present | Error |
| ICE06 | Database column definitions | Error |
| ICE08 | Duplicate component GUIDs | Error |
| ICE09 | Permanent components in SystemFolder | Warning |
| ICE13 | Dialogs in correct sequence tables | Error |
| ICE18 | Empty directory as KeyPath | Error |
| ICE24 | Property name conflicts | Error |
| ICE27 | Action sequence validation | Error |
| ICE30 | File sequence/media validation | Error |
| ICE33 | Registry key validation | Error |
| ICE38 | Component with same name as feature | Warning |
| ICE43 | Non-advertised shortcuts | Warning |
| ICE45 | Reserved bit usage | Warning |
| ICE57 | Per-user/per-machine consistency | Error |
| ICE61 | Upgrade code validation | Error |
| ICE64 | New directory per-user profile | Warning |
| ICE67 | Target of shortcut validation | Error |
| ICE69 | Shortcut in Advertise table | Error |
| ICE80 | 64-bit component validation | Error |
| ICE82 | InstallExecuteSequence actions | Error |
| ICE83 | MsiAssembly table validation | Error |
| ICE91 | File hash validation | Error |
| ICE99 | Reserved directory names | Error |

---

## 4. WiX Error Codes

### Error Code Prefixes

| Prefix | Tool | Version |
|--------|------|---------|
| CNDL | Candle (compiler) | v3 |
| LGHT | Light (linker) | v3 |
| LIT | Lit (library) | v3 |
| SMOK | Smoke (validation) | v3 |
| PYRO | Pyro (patching) | v3 |
| WIX | wix.exe (unified) | v4+ |

### Source
- Extract from WiX source code: `https://github.com/wixtoolset/wix`
- Error message resources in `/src/` directories

---

## 5. Code Examples

### GitHub Repositories

| Repository | Description | WiX Version |
|------------|-------------|-------------|
| [wixtoolset/wix](https://github.com/wixtoolset/wix) | Official WiX Toolset | v4/v5/v6 |
| [wixtoolset/wix3](https://github.com/wixtoolset/wix3) | WiX v3.x source | v3 |
| [kurtanr/WiXInstallerExamples](https://github.com/kurtanr/WiXInstallerExamples) | Multiple sample installers | v3 |
| [n13org/WixToolset-Tutorials](https://github.com/n13org/WixToolset-Tutorials) | Tutorials with samples | v3/v4/v5 |
| [stephenlepisto/wixtoolsetexamples](https://github.com/stephenlepisto/wixtoolsetexamples) | Dialog templates | v5 |
| [michelou/wix-examples](https://github.com/michelou/wix-examples) | Various examples | v3/v4 |
| [deepak-rathi/Wix-Setup-Samples](https://github.com/deepak-rathi/Wix-Setup-Samples) | Beginner samples | v3 |

### Real-World Open Source Projects Using WiX

Search GitHub for `.wxs` files in active projects to gather real-world patterns.

---

## 6. Preprocessor Reference

### Variable Types

| Syntax | Type | Example |
|--------|------|---------|
| `$(var.Name)` | Custom variable | `$(var.ProductVersion)` |
| `$(env.Name)` | Environment variable | `$(env.windir)` |
| `$(sys.NAME)` | System variable | `$(sys.CURRENTDIR)` |
| `$(loc.Name)` | Localization string | `$(loc.WelcomeMessage)` |
| `$(wix.Name)` | WiX variable | `$(wix.WixMajorVersion)` |

### Preprocessor Directives

```xml
<?define VarName = "value" ?>
<?if $(var.Condition) ?>
<?ifdef VarName ?>
<?ifndef VarName ?>
<?else?>
<?elseif $(var.Other) ?>
<?endif?>
<?include "file.wxi" ?>
<?foreach VarName in val1;val2;val3 ?>
<?endforeach?>
<?error "message" ?>
<?warning "message" ?>
```

### Built-in Functions

```
$(sys.CURRENTDIR)
$(sys.SOURCEFILEPATH)
$(sys.SOURCEFILEDIR)
$(sys.PLATFORM)
$(sys.BUILDARCH)
```

---

## 7. Documentation Sources

### Official

| Source | URL | Notes |
|--------|-----|-------|
| WiX Toolset Docs | https://wixtoolset.org/docs/ | Current version |
| WiX v3 Manual | https://wixtoolset.org/docs/v3/ | Legacy but comprehensive |
| FireGiant Docs | https://docs.firegiant.com/wix/ | Maintained by WiX team |
| Schema Reference | https://wixtoolset.org/docs/schema/ | Element/attribute details |

### Community

| Source | URL | Notes |
|--------|-----|-------|
| WiX v3 Tutorial | https://docs.firegiant.com/wix3/tutorial/ | Step-by-step guide |
| GitHub Discussions | https://github.com/orgs/wixtoolset/discussions | Q&A |
| Stack Overflow | Tag: `wix` | Community answers |

### Books

- "WiX 3.6: A Developer's Guide to Windows Installer XML" (Packt)
- "The Definitive Guide to Windows Installer" (Apress)

---

## 8. MSI Internals (Windows Installer Database)

WiX generates MSI files, so understanding the underlying MSI database is essential.

### Primary Documentation Sources

| Source | URL | Content |
|--------|-----|---------|
| **Database Tables** | https://learn.microsoft.com/en-us/windows/win32/msi/database-tables | Complete table reference |
| **Property Reference** | https://learn.microsoft.com/en-us/windows/win32/msi/property-reference | All MSI properties |
| **Standard Actions** | https://learn.microsoft.com/en-us/windows/win32/msi/standard-actions-reference | Built-in actions |
| **Custom Actions** | https://learn.microsoft.com/en-us/windows/win32/msi/custom-actions | CA types 1-54 |
| **Error Codes** | https://learn.microsoft.com/en-us/windows/win32/msi/error-codes | MsiExec error codes |
| **ICE Reference** | https://learn.microsoft.com/en-us/windows/win32/msi/ice-reference | All ICE rules |

### GitHub Source (Scrapeable)

Microsoft Learn docs are mirrored on GitHub for easier extraction:
- `https://github.com/MicrosoftDocs/win32/tree/docs/desktop-src/Msi`

### Key MSI Tables

| Table | Purpose | WiX Element |
|-------|---------|-------------|
| Property | Key-value configuration | `<Property>` |
| Directory | Folder structure | `<Directory>`, `<StandardDirectory>` |
| Component | Installation units | `<Component>` |
| File | Files to install | `<File>` |
| Registry | Registry entries | `<RegistryKey>`, `<RegistryValue>` |
| Feature | User-selectable features | `<Feature>` |
| FeatureComponents | Feature to component mapping | (implicit) |
| CustomAction | Custom code execution | `<CustomAction>` |
| InstallExecuteSequence | Installation order | `<InstallExecuteSequence>` |
| InstallUISequence | UI display order | `<InstallUISequence>` |
| Shortcut | Desktop/Start menu shortcuts | `<Shortcut>` |
| ServiceInstall | Windows services | `<ServiceInstall>` |
| ServiceControl | Service start/stop | `<ServiceControl>` |
| Environment | Environment variables | `<Environment>` |
| LaunchCondition | Install prerequisites | `<Condition>` |
| Upgrade | Upgrade detection | `<Upgrade>`, `<MajorUpgrade>` |
| Media | Source media definition | `<Media>`, `<MediaTemplate>` |
| Binary | Embedded binary data | `<Binary>` |
| Icon | Application icons | `<Icon>` |

### MSI Error Codes (msiexec.exe)

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1601 | Windows Installer service not accessible |
| 1602 | User cancelled |
| 1603 | Fatal error during installation |
| 1604 | Installation suspended, incomplete |
| 1605 | This action only valid for currently installed products |
| 1618 | Another installation in progress |
| 1619 | Installation package could not be opened |
| 1620 | Installation package could not be opened (path/permissions) |
| 1624 | Error applying transforms |
| 1625 | Installation prohibited by policy |
| 1638 | Another version already installed |
| 1639 | Invalid command line argument |
| 1641 | Reboot initiated |
| 3010 | Reboot required |

### Standard Actions (Sequence)

Key actions that WiX schedules:

| Action | Purpose | Typical Sequence |
|--------|---------|------------------|
| AppSearch | Search for existing files/registry | Early |
| LaunchConditions | Check prerequisites | Early |
| CostInitialize | Begin disk costing | Before UI |
| FileCost | Calculate file space | Before UI |
| CostFinalize | Complete disk costing | Before UI |
| InstallValidate | Validate installation | Before execute |
| InstallInitialize | Begin install transaction | Execute start |
| ProcessComponents | Register components | Execute |
| InstallFiles | Copy files | Execute |
| WriteRegistryValues | Write registry | Execute |
| RegisterProduct | Register with Windows | Execute |
| InstallFinalize | Complete transaction | Execute end |
| RemoveExistingProducts | Uninstall previous version | Varies |

### Custom Action Types

| Type | Source | Execution | Notes |
|------|--------|-----------|-------|
| 1 | DLL in Binary table | Immediate | Most common |
| 2 | EXE in Binary table | Immediate | External process |
| 5 | JScript in Binary table | Immediate | Scripted |
| 6 | VBScript in Binary table | Immediate | Scripted |
| 17 | DLL from installed file | Deferred | After InstallFiles |
| 18 | EXE from installed file | Deferred | After InstallFiles |
| 21 | JScript from installed file | Deferred | After InstallFiles |
| 22 | VBScript from installed file | Deferred | After InstallFiles |
| 34 | Directory path in property | Immediate | Set property |
| 35 | Directory path in property | Immediate | Set directory |
| 50 | EXE with command line | Immediate | Run any EXE |
| 51 | Property assignment | Immediate | Copy property |

### Condition Syntax

MSI conditions use specific operators and properties:

```
# Operators
=, <>, <, >, <=, >=, ><  (substring)
AND, OR, NOT
~=  (case-insensitive equals)

# Common Properties for Conditions
VersionNT           # OS version (e.g., 603 = Win 8.1, 1000 = Win 10+)
VersionNT64         # 64-bit OS version
MsiNTProductType    # 1=Workstation, 2=DC, 3=Server
Privileged          # Running elevated
ALLUSERS            # Per-machine install
Installed           # Product already installed
REINSTALL           # Repair/reinstall mode
REMOVE              # Uninstall mode

# Examples
VersionNT >= 600                    # Vista or later
VersionNT64                         # 64-bit OS
MsiNTProductType = 1                # Workstation only
NOT Installed                       # First install only
Installed AND REINSTALL             # Repair only
Installed AND REMOVE="ALL"          # Uninstall only
```

### OS Version Values

| OS | VersionNT | VersionNT64 |
|----|-----------|-------------|
| Windows XP | 501 | - |
| Windows Vista | 600 | 600 |
| Windows 7 | 601 | 601 |
| Windows 8 | 602 | 602 |
| Windows 8.1 | 603 | 603 |
| Windows 10/11 | 603* | 603* |

*Note: Windows 10/11 reports as 603 by default. Some tools reset to 1000.

---

## 9. MSIX Support (Limited)

WiX does NOT natively create MSIX packages. Options:

| Approach | Tool | Notes |
|----------|------|-------|
| FireGiant Extension | HeatWave.Msix | Commercial, WiX v4+ |
| WixSharp + MSIX Packaging Tool | Microsoft tool | Convert MSI to MSIX |
| Separate MSIX project | makeappx.exe | Manual, not WiX |

**Recommendation:** Focus on MSI for WixCraft. MSIX is out of scope for now.

---

## 10. Data Layer Architecture

### Two-Layer Model

```
┌─────────────────────────────────────────────┐
│           WiX Layer (XML Schema)            │
│  Elements, Attributes, Preprocessor, etc.   │
├─────────────────────────────────────────────┤
│         MSI Layer (Database Schema)         │
│  Tables, Properties, Actions, Conditions    │
└─────────────────────────────────────────────┘
```

### Why Both Layers Matter

1. **WiX Layer** - What developers write (XML)
   - Element names, valid children, required attributes
   - Preprocessor variables, includes
   - WiX-specific abstractions (MajorUpgrade, StandardDirectory)

2. **MSI Layer** - What WiX generates (database)
   - Table structures, column types
   - Standard actions, custom action types
   - Conditions, properties
   - Error codes, ICE validation

### Mapping WiX to MSI

| WiX Element | MSI Table(s) | Notes |
|-------------|--------------|-------|
| `<Package>` | Property, _SummaryInformation | Product metadata |
| `<Directory>` | Directory | Folder structure |
| `<Component>` | Component | Installation unit |
| `<File>` | File | File to install |
| `<RegistryKey>` | Registry | Registry entries |
| `<Feature>` | Feature, FeatureComponents | Selection UI |
| `<CustomAction>` | CustomAction, Binary | Custom code |
| `<MajorUpgrade>` | Upgrade, Property | WiX abstraction |
| `<StandardDirectory>` | Directory | WiX v4+ abstraction |

---

## 11. Data Extraction Strategy

### Phase 1: WiX Schema Extraction
1. Download WiX XSD files (v3, v4, v5)
2. Parse XSD to extract:
   - Element names and descriptions
   - Attribute names, types, required status
   - Parent/child relationships
   - Enum values
3. Output as JSON database

### Phase 2: Property/Directory Database
1. Scrape Microsoft Learn for MSI properties
2. Extract StandardDirectory enum from WiX schema
3. Document common values and descriptions

### Phase 3: Error Database
1. Extract error codes from WiX source
2. Scrape ICE rules from Microsoft Learn
3. Add fix suggestions and examples

### Phase 4: Code Snippets
1. Analyze GitHub examples for common patterns
2. Create snippet templates for:
   - Basic MSI package
   - Service installation
   - Registry configuration
   - Custom actions
   - Bootstrapper bundle

---

## 12. Data Format

### Recommended Structure

```
wix-data/
├── elements/           # One JSON per element
│   ├── component.json
│   ├── file.json
│   └── ...
├── attributes/         # Shared attribute types
│   └── types.json
├── properties/         # MSI properties
│   └── standard.json
├── directories/        # Standard directories
│   └── standard.json
├── errors/             # Error codes
│   ├── wix-errors.json
│   └── ice-errors.json
├── snippets/           # Code templates
│   └── snippets.json
└── schema/             # Version info
    └── versions.json
```

### Example Element JSON

```json
{
  "name": "Component",
  "description": "A component is a piece of the application that is installed.",
  "documentation": "https://wixtoolset.org/docs/schema/wxs/component/",
  "parents": ["ComponentGroup", "Directory", "DirectoryRef", "Fragment"],
  "children": ["File", "Registry", "RegistryKey", "Shortcut", "..."],
  "attributes": [
    {
      "name": "Id",
      "type": "string",
      "required": true,
      "description": "Component identifier"
    },
    {
      "name": "Guid",
      "type": "guid",
      "required": false,
      "description": "Component GUID for identification"
    }
  ],
  "examples": ["..."],
  "versions": {"introduced": "3.0", "deprecated": null}
}
```
