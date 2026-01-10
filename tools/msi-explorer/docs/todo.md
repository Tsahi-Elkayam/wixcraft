# MSI Explorer - Future Features

A list of features and improvements planned for MSI Explorer.

## High Priority

### Core Functionality
- [ ] **MSI Writing Support** - Actually save changes back to MSI file (currently read-only)
- [ ] **Transform Creation** - Create MST transform files from changes
- [ ] **Full MSP Patch Support** - Open and edit MSP patch files natively
- [ ] **MSM Merge Module Editor** - Full support for creating/editing merge modules
- [ ] **CAB Extraction** - Extract embedded CAB files to disk
- [ ] **CAB Rebuilding** - Rebuild CAB files with modified content

### MSI Internals
- [ ] **Summary Information Stream Editor** - Full editing of summary stream
- [ ] **_Validation Table Editor** - Define and edit column constraints
- [ ] **Storage/Stream Browser** - Raw OLE compound document structure
- [ ] **String Pool Analyzer** - View and optimize string pool
- [ ] **Codepage Handler** - Handle different codepages correctly

### Validation
- [ ] **Full ICE Suite** - Implement all 100+ ICE rules natively
- [ ] **Custom ICE Rules** - Allow user-defined validation rules
- [ ] **Real-time Validation** - Validate as you type/edit
- [ ] **Validation Profiles** - Save sets of rules for different scenarios
- [ ] **MSI Schema Version Detection** - Detect v1.0, v2.0, v3.0 schema versions
- [ ] **Database Integrity Checker** - Verify internal consistency
- [ ] **Orphaned Stream Cleanup** - Find and remove unused streams

### Editing
- [ ] **Multi-cell Selection** - Select and edit multiple cells at once
- [ ] **Copy/Paste Rows** - Copy rows within and between tables
- [ ] **Drag & Drop Rows** - Reorder rows by dragging
- [ ] **Column Reordering** - Drag columns to reorder
- [ ] **Find & Replace** - Global find and replace across all tables

### Batch Operations
- [ ] **Batch GUID Regeneration** - Regenerate GUIDs in bulk
- [ ] **Batch Path Updates** - Find/replace in paths across tables
- [ ] **Batch Property Changes** - Update multiple properties at once
- [ ] **Regex Find and Replace** - Regex-powered bulk editing

### Table-Specific Tools
- [ ] **Registry Path Resolver** - Convert HKEY roots to full paths
- [ ] **File Attribute Decoder** - Explain file attribute bit flags
- [ ] **Component Keypath Analyzer** - Validate and suggest keypaths
- [ ] **Feature Level Advisor** - Recommend feature install levels
- [ ] **Directory Standard Folder Mapper** - Map to standard directories
- [ ] **Shortcut Target Validator** - Validate shortcut targets exist
- [ ] **Service Account Validator** - Validate service account settings

## Medium Priority

### UI/UX Improvements
- [ ] **Dark/Light Theme Toggle** - User-selectable theme
- [ ] **Custom Color Schemes** - Configurable color themes
- [ ] **Resizable Panels** - Remember panel sizes
- [ ] **Multiple Windows** - Open multiple MSI files in separate windows
- [ ] **Tabbed Interface** - Open multiple files in tabs
- [ ] **Docking Panels** - Rearrangeable panel layout
- [ ] **Zoom Support** - Scale UI for different display densities
- [ ] **Font Size Adjustment** - Configurable font sizes
- [ ] **Breadcrumb Navigation** - Show current location path
- [ ] **Quick Actions Toolbar** - Customizable toolbar

### Accessibility
- [ ] **Screen Reader Support** - Full ARIA/accessibility support
- [ ] **High Contrast Mode** - For visually impaired users
- [ ] **Keyboard-only Navigation** - Complete keyboard support
- [ ] **Color Blind Modes** - Alternative color schemes
- [ ] **Large Cursor Mode** - Enhanced cursor visibility

### Binary/Stream Operations
- [ ] **Hex Editor** - Edit binary streams in hex view
- [ ] **Script Decompiler** - Decompile VBScript/JScript custom actions
- [ ] **Icon Editor** - Edit embedded icons
- [ ] **Bitmap Viewer** - View embedded bitmaps
- [ ] **Certificate Viewer** - View embedded certificates

### File Analysis
- [ ] **PE Header Viewer** - View DLL/EXE headers from Binary table
- [ ] **DLL Dependency Walker** - Show DLL dependencies
- [ ] **File Version Extractor** - Extract version info from binaries
- [ ] **Digital Signature Chain Viewer** - Full certificate chain display
- [ ] **Assembly Info Viewer** - .NET assembly metadata

### Custom Action Tools
- [ ] **CA Type Decoder** - Explain what Type 1, 2, 17, 50, etc. mean
- [ ] **Deferred/Immediate Visualizer** - Show execution context for each CA
- [ ] **Impersonation Analyzer** - Analyze CA security context
- [ ] **Script Syntax Checker** - Validate VBScript/JScript syntax
- [ ] **CA Dependency Graph** - Show CA dependencies and order

### UI/Dialog Tools
- [ ] **Dialog Flow Visualizer** - Show ControlEvent navigation chains
- [ ] **Control Tab Order Editor** - Edit tab order visually
- [ ] **Billboard Sequence Previewer** - Preview billboard timing
- [ ] **Text Style Previewer** - Preview TextStyle definitions
- [ ] **Control Positioning Editor** - Visual layout editor

### Upgrades & Patches
- [ ] **Upgrade Code Relationship Graph** - Visualize upgrade relationships
- [ ] **FindRelatedProducts Analyzer** - Analyze product detection
- [ ] **Patch Applicability Checker** - Check if patch applies
- [ ] **Minor/Major Upgrade Advisor** - Recommend upgrade type
- [ ] **Patch Sequencing Analyzer** - Analyze patch order

### Analysis Tools
- [ ] **Disk Space Calculator** - Accurate installation size calculation
- [ ] **Registry Impact Report** - Show all registry changes
- [ ] **File System Impact Report** - Show all file system changes
- [ ] **Upgrade Path Analyzer** - Analyze upgrade scenarios
- [ ] **Conflict Detector** - Detect conflicts with other MSIs
- [ ] **Security Analyzer** - Check for security issues (permissions, paths)
- [ ] **Compliance Report** - Check against corporate policies
- [ ] **Security Audit Report** - Full security assessment
- [ ] **Change Log Generator** - Generate changelog between MSI versions

### Data Quality
- [ ] **Duplicate GUID Finder** - Find duplicate GUIDs across all tables
- [ ] **Invalid Path Syntax Detector** - Find malformed paths
- [ ] **Broken Condition Syntax Highlighter** - Validate condition expressions
- [ ] **Component GUID Collision Checker** - Detect GUID conflicts
- [ ] **Missing File Reference Detector** - Find references to missing files
- [ ] **Foreign Key Integrity Checker** - Validate all FK relationships

### Education & Help
- [ ] **Interactive MSI Tutorials** - Step-by-step learning
- [ ] **Best Practices Advisor** - Suggest improvements
- [ ] **Common Mistakes Detector** - Flag known anti-patterns
- [ ] **Guided Fix Wizards** - Walk through fixing issues
- [ ] **Contextual Help System** - Help based on current context
- [ ] **MSI Glossary** - Built-in terminology reference

### Debugging Tools
- [ ] **MSI Log File Parser** - Parse and visualize verbose MSI logs
- [ ] **Windows Event Log Integration** - View installation events
- [ ] **Installation Troubleshooter** - Wizard to diagnose install failures
- [ ] **Error Code Database** - Lookup MSI error codes with solutions
- [ ] **Property Tracker** - Track property values during simulation

### Developer Tools
- [ ] **Generate Empty MSI** - Create MSI from template
- [ ] **MSI Test File Generator** - Generate test MSIs for validation
- [ ] **Snippet Library** - Common patterns and code snippets
- [ ] **Schema Reference** - Built-in MSI schema documentation
- [ ] **Sample MSI Browser** - Browse example MSI structures

### Import/Export
- [ ] **Import from Orca** - Import Orca table exports
- [ ] **Export to InstallShield** - Export in InstallShield format
- [ ] **Export to Advanced Installer** - Compatible export format
- [ ] **PowerShell Script Export** - Generate PS scripts for automation
- [ ] **C# Code Generation** - Generate WiX# code
- [ ] **Documentation Generator** - Auto-generate MSI documentation

### Templates & Presets
- [ ] **MSI Templates** - Basic, service, driver installer templates
- [ ] **Property Presets** - Common property value sets
- [ ] **CustomAction Templates** - Pre-built CA patterns
- [ ] **Feature Structure Templates** - Common feature hierarchies
- [ ] **Table Templates** - Pre-populated table structures

### Session Management
- [ ] **Project Files** - Save analysis state to project file
- [ ] **Workspace Management** - Multiple project workspaces
- [ ] **Bookmarks Across Sessions** - Persistent bookmarks
- [ ] **Recent Files with Preview** - Thumbnail previews of recent MSIs
- [ ] **Auto-save Recovery** - Recover from crashes

### Advanced Search
- [ ] **Regex Search Across Tables** - Full regex support
- [ ] **Visual Query Builder** - SQL-like drag-drop query builder
- [ ] **Saved Searches** - Save and reuse search queries
- [ ] **Search in Binary Streams** - Hex pattern search
- [ ] **Search History with Results** - Browse past search results

### Interoperability
- [ ] **WiX dark.exe Integration** - Direct decompile support
- [ ] **lessmsi Integration** - Extract files via lessmsi
- [ ] **Orca Feature Parity Mode** - Match Orca shortcuts/behavior
- [ ] **InstEd Import/Export** - InstEd compatibility layer
- [ ] **SuperOrca Compatibility** - Import SuperOrca projects

## Low Priority

### Automation
- [ ] **Scripting Support** - Built-in scripting (Lua/Python)
- [ ] **Macro Recording** - Record and playback actions
- [ ] **Batch Processing** - Process multiple MSIs
- [ ] **Command-line Automation** - Full CLI for all operations
- [ ] **Watch Mode** - Auto-reload when file changes

### Comparison
- [ ] **3-Way Diff** - Compare three MSI files
- [ ] **Directory Comparison** - Compare MSI against installed directory
- [ ] **Patch Diff** - Show what a patch will change
- [ ] **Version History** - Track changes across multiple versions

### Integration
- [ ] **GitHub Integration** - Commit MSI changes directly
- [ ] **CI/CD Integration** - Plugins for Jenkins, Azure DevOps
- [ ] **WiX Toolset Integration** - Direct WiX build/decompile
- [ ] **Visual Studio Extension** - VS integration
- [ ] **VS Code Extension** - Lightweight editor integration

### Network & Cloud
- [ ] **Open MSI from URL** - Download and open remote MSI files
- [ ] **Auto-update Checker** - Check for new versions
- [ ] **Crash Reporting** - Optional crash report submission
- [ ] **Cloud Storage Integration** - Open from OneDrive, Google Drive, Dropbox
- [ ] **Share Analysis Results** - Export shareable reports

### Advanced Features
- [ ] **MSI Repair Wizard** - Fix common MSI issues automatically
- [ ] **Performance Profiler** - Identify slow custom actions
- [ ] **Installation Simulator** - Full simulation with virtual filesystem
- [ ] **Rollback Simulator** - Simulate rollback scenarios
- [ ] **Condition Debugger** - Step-through condition evaluation
- [ ] **Custom Action Debugger** - Debug CA execution
- [ ] **AI-Powered Suggestions** - Smart recommendations for improvements

### Localization
- [ ] **String Extraction** - Extract all localizable strings
- [ ] **Translation Memory** - Integrate with TM tools
- [ ] **Multi-language Preview** - Preview UI in different languages
- [ ] **RTL Support** - Right-to-left language support

### Collaboration
- [ ] **Change Tracking** - Track who changed what
- [ ] **Comments/Annotations** - Add notes to tables/rows
- [ ] **Share Sessions** - Real-time collaboration
- [ ] **Export Change Report** - Generate change reports for review

## Technical Debt

### Code Quality
- [ ] **Increase Test Coverage** - Target 90%+ coverage
- [ ] **Integration Tests** - Tests with real MSI files
- [ ] **Performance Benchmarks** - Track performance over time
- [ ] **Memory Profiling** - Optimize memory usage
- [ ] **Reduce Warnings** - Fix all compiler warnings

### Architecture
- [ ] **Plugin Architecture** - Formal plugin API
- [ ] **Async Operations** - Non-blocking file operations
- [ ] **Caching Layer** - Cache table data for large files
- [ ] **Undo System Optimization** - More efficient undo storage
- [ ] **Lazy Loading** - Load tables on demand

### Documentation
- [ ] **API Documentation** - Document public API
- [ ] **Architecture Guide** - Document codebase structure
- [ ] **Contributing Guide** - How to contribute
- [ ] **Video Tutorials** - Screencasts for common tasks

## Platform-Specific

### Windows
- [ ] **Windows Installer API Integration** - Use native MSI APIs
- [ ] **Shell Integration** - Context menu, file associations
- [ ] **Jump List Support** - Recent files in taskbar
- [ ] **Toast Notifications** - Notify on long operations

### macOS
- [ ] **Touch Bar Support** - MacBook Pro touch bar
- [ ] **macOS Menu Bar** - Native menu integration
- [ ] **Handoff Support** - Continue on other devices
- [ ] **Spotlight Integration** - Search MSI content

### Linux
- [ ] **Wayland Support** - Native Wayland rendering
- [ ] **System Tray Integration** - Background monitoring
- [ ] **Desktop Integration** - .desktop file, MIME types

## Community Requests

_This section will be populated based on user feedback and GitHub issues._

---

## Completed Features

See [README.md](../README.md) for the full list of implemented features.

## Contributing

Want to help implement these features? See our [Contributing Guide](CONTRIBUTING.md) (coming soon).

## Priority Legend

- **High Priority**: Core functionality, frequently requested
- **Medium Priority**: Nice to have, improves workflow
- **Low Priority**: Future enhancements, specialized use cases
