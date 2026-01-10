//! Generated WiX lint rules
//!
//! Generated from wixkb database on 2026-01-06 10:11:08
//! Total rules: 234
//!
//! DO NOT EDIT MANUALLY - regenerate with gen-rules.py

use crate::diagnostic::Severity;
use crate::rule::Rule;

/// Get all built-in WiX rules
pub fn builtin_rules() -> Vec<Rule> {
    vec![
        Rule::new(
            "billboard-requires-feature",
            "!attributes.Feature",
            "Billboard '{attributes.Id}' is missing Feature attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Billboard"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "billboard-requires-id",
            "!attributes.Id",
            "Billboard is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Billboard"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "billboardaction-requires-id",
            "!attributes.Id",
            "BillboardAction is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("BillboardAction"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "binary-requires-id",
            "!attributes.Id",
            "Binary is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Binary"))
        .with_tag("required")
        ,

        Rule::new(
            "binary-requires-sourcefile",
            "!attributes.SourceFile",
            "Binary is missing SourceFile attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Binary"))
        .with_tag("required")
        ,

        Rule::new(
            "bootstrapperapplication-requires-id",
            "!attributes.Id",
            "BootstrapperApplication needs Id or use BootstrapperApplicationRef",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("BootstrapperApplication"))
        .with_tag("recommended")
        ,

        Rule::new(
            "bundle-requires-manufacturer",
            "!attributes.Manufacturer",
            "Bundle is missing Manufacturer attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Bundle"))
        .with_tag("required")
        ,

        Rule::new(
            "bundle-requires-name",
            "!attributes.Name",
            "Bundle is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Bundle"))
        .with_tag("required")
        ,

        Rule::new(
            "bundle-requires-upgradecode",
            "!attributes.UpgradeCode",
            "Bundle is missing UpgradeCode attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Bundle"))
        .with_tag("required")
        ,

        Rule::new(
            "bundle-requires-version",
            "!attributes.Version",
            "Bundle is missing Version attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Bundle"))
        .with_tag("required")
        ,

        Rule::new(
            "chain-disable-rollback-warning",
            "attributes.DisableRollback == \"yes\"",
            "Chain has rollback disabled - failed installs cannot be rolled back",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Chain"))
        .with_tag("awareness")
        ,

        Rule::new(
            "combobox-requires-property",
            "!attributes.Property",
            "ComboBox is missing Property attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ComboBox"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "component-directory-mismatch",
            "name == \"Component\" && !attributes.Directory",
            "Component '{attributes.Id}' should specify Directory or be inside a Directory element",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Component"))
        .with_tag("best-practice")
        ,

        Rule::new(
            "component-empty-guid",
            "attributes.Guid == \"\"",
            "Component '{attributes.Id}' has empty Guid - use '*' for auto-generation or specify a GUID",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Component"))
        .with_tag("validation")
        ,

        Rule::new(
            "component-id-prefix",
            "attributes.Id && !attributes.Id =~ /^(cmp|Cmp|CMP)/",
            "Component Id '{attributes.Id}' - consider using 'cmp' or 'Cmp' prefix for clarity",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Component"))
        .with_tag("naming")
        ,

        Rule::new(
            "component-keypath-file",
            "name == \"Component\" && hasChild('File') && !hasChild('RegistryValue')",
            "Component with File but no explicit KeyPath - first File is keypath by default",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Component"))
        .with_tag("awareness")
        ,

        Rule::new(
            "component-permanent-warning",
            "attributes.Permanent == \"yes\"",
            "Component '{attributes.Id}' is marked Permanent - files will not be removed on uninstall",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Component"))
        .with_tag("awareness")
        ,

        Rule::new(
            "component-requires-guid",
            "!attributes.Guid",
            "Component '{attributes.Id}' is missing Guid attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Component"))
        .with_tag("best-practice")
        ,

        Rule::new(
            "component-requires-id",
            "!attributes.Id",
            "Component is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Component"))
        .with_tag("required")
        ,

        Rule::new(
            "component-shared-warning",
            "attributes.Shared == \"yes\"",
            "Component '{attributes.Id}' is marked Shared - ensure proper reference counting",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Component"))
        .with_tag("awareness")
        ,

        Rule::new(
            "component-win64-mismatch",
            "attributes.Win64 == \"yes\"",
            "Component '{attributes.Id}' is marked Win64 - ensure this matches your target platform",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Component"))
        .with_tag("platform")
        ,

        Rule::new(
            "componentgroup-requires-id",
            "!attributes.Id",
            "ComponentGroup is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ComponentGroup"))
        .with_tag("required")
        ,

        Rule::new(
            "componentref-requires-id",
            "!attributes.Id",
            "ComponentRef is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ComponentRef"))
        .with_tag("required")
        ,

        Rule::new(
            "condition-empty",
            "isEmpty(attributes.Message)",
            "Condition has empty Message attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Condition"))
        .with_tag("validation")
        ,

        Rule::new(
            "condition-empty-string-compare",
            "attributes.Condition =~ /= \"\"/",
            "Condition compares to empty string - consider using NOT PropertyName instead",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("condition")
        ,

        Rule::new(
            "condition-installed",
            "attributes.Condition =~ /Installed/",
            "Condition uses Installed property - true during maintenance mode",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("condition")
        ,

        Rule::new(
            "condition-msi-version",
            "attributes.Condition =~ /MsiNTProductType/",
            "Condition uses MsiNTProductType - valid values: 1=workstation, 2=domain controller, 3=server",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("condition")
        ,

        Rule::new(
            "condition-privileged",
            "attributes.Condition =~ /Privileged/",
            "Condition checks Privileged - ensures install runs with elevated privileges",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("condition")
        .with_tag("security")
        ,

        Rule::new(
            "condition-property-reference",
            "attributes.Condition =~ /\\[[A-Z]/",
            "Condition references property with brackets - use plain property name",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), None)
        .with_tag("condition")
        ,

        Rule::new(
            "condition-remove",
            "attributes.Condition =~ /REMOVE/",
            "Condition uses REMOVE - verify uninstall scenarios work correctly",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("condition")
        ,

        Rule::new(
            "condition-syntax-and",
            "attributes.Condition =~ / AND /",
            "Condition uses ' AND ' - MSI conditions are case-insensitive but WiX convention is lowercase",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("condition")
        ,

        Rule::new(
            "condition-syntax-or",
            "attributes.Condition =~ / OR /",
            "Condition uses ' OR ' - MSI conditions are case-insensitive but WiX convention is lowercase",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("condition")
        ,

        Rule::new(
            "condition-unbalanced-parens",
            "attributes.Condition =~ /\\([^)]*$|^[^(]*\\)/",
            "Condition may have unbalanced parentheses",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), None)
        .with_tag("condition")
        .with_tag("validation")
        ,

        Rule::new(
            "condition-version-compare",
            "attributes.Condition =~ /VersionNT\\s*[<>=]/",
            "Condition uses VersionNT comparison - ensure correct operator for version checks",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("condition")
        ,

        Rule::new(
            "control-property-checkbox",
            "attributes.Type == \"CheckBox\" && !attributes.Property",
            "CheckBox Control '{attributes.Id}' is missing Property attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Control"))
        .with_tag("ui")
        ,

        Rule::new(
            "control-property-edit",
            "attributes.Type == \"Edit\" && !attributes.Property",
            "Edit Control '{attributes.Id}' is missing Property attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Control"))
        .with_tag("ui")
        ,

        Rule::new(
            "control-requires-height",
            "!attributes.Height",
            "Control '{attributes.Id}' is missing Height",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Control"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "control-requires-id",
            "!attributes.Id",
            "Control is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Control"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "control-requires-type",
            "!attributes.Type",
            "Control '{attributes.Id}' is missing Type attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Control"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "control-requires-width",
            "!attributes.Width",
            "Control '{attributes.Id}' is missing Width",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Control"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "control-requires-x",
            "!attributes.X",
            "Control '{attributes.Id}' is missing X position",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Control"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "control-requires-y",
            "!attributes.Y",
            "Control '{attributes.Id}' is missing Y position",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Control"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "control-text-pushbutton",
            "attributes.Type == \"PushButton\" && !attributes.Text",
            "PushButton Control '{attributes.Id}' is missing Text attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Control"))
        .with_tag("ui")
        ,

        Rule::new(
            "controlevent-requires-dialog",
            "!attributes.Dialog && !attributes.NewDialog",
            "ControlEvent should have Dialog or NewDialog attribute for navigation",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("ControlEvent"))
        .with_tag("ui")
        ,

        Rule::new(
            "createfolder-requires-directory",
            "!attributes.Directory",
            "CreateFolder should specify Directory attribute for clarity",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("CreateFolder"))
        .with_tag("recommended")
        ,

        Rule::new(
            "customaction-dll-entry",
            "attributes.DllEntry && !attributes.BinaryRef",
            "CustomAction '{attributes.Id}' has DllEntry but no BinaryRef",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("CustomAction"))
        .with_tag("validation")
        ,

        Rule::new(
            "customaction-elevated-script",
            "attributes.Execute == \"deferred\" && attributes.Impersonate == \"no\" && attributes.Script",
            "Script CustomAction '{attributes.Id}' runs elevated - review for security implications",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("CustomAction"))
        .with_tag("security")
        ,

        Rule::new(
            "customaction-execute-deferred",
            "attributes.Execute == \"deferred\" && !attributes.Impersonate",
            "Deferred CustomAction '{attributes.Id}' should specify Impersonate attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("CustomAction"))
        .with_tag("best-practice")
        ,

        Rule::new(
            "customaction-requires-id",
            "!attributes.Id",
            "CustomAction is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("CustomAction"))
        .with_tag("required")
        ,

        Rule::new(
            "customaction-return-check",
            "attributes.Return == \"ignore\"",
            "CustomAction '{attributes.Id}' ignores return code - failures won't stop installation",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("CustomAction"))
        .with_tag("awareness")
        ,

        Rule::new(
            "customaction-script-impersonate",
            "attributes.Script && attributes.Impersonate != \"no\"",
            "CustomAction '{attributes.Id}' script runs impersonated - consider Impersonate=no for elevated actions",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("CustomAction"))
        .with_tag("security")
        ,

        Rule::new(
            "customactionref-requires-id",
            "!attributes.Id",
            "CustomActionRef is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("CustomActionRef"))
        .with_tag("required")
        ,

        Rule::new(
            "deeply-nested-directory",
            "name == \"Directory\" && countChildren('Directory') > 10",
            "Deeply nested Directory structure - consider flattening",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Directory"))
        .with_tag("organization")
        ,

        Rule::new(
            "deprecated-module-element",
            "name == \"Module\"",
            "Module element is deprecated in WiX v4 - merge modules are discouraged",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Module"))
        .with_tag("deprecated")
        .with_tag("migration")
        ,

        Rule::new(
            "deprecated-patchcreation",
            "name == \"PatchCreation\"",
            "PatchCreation is deprecated - use Patch element instead",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("PatchCreation"))
        .with_tag("deprecated")
        ,

        Rule::new(
            "deprecated-product-element",
            "name == \"Product\"",
            "Product element is deprecated in WiX v4 - use Package instead",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Product"))
        .with_tag("deprecated")
        .with_tag("migration")
        ,

        Rule::new(
            "dialog-requires-height",
            "!attributes.Height",
            "Dialog '{attributes.Id}' is missing Height attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Dialog"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "dialog-requires-id",
            "!attributes.Id",
            "Dialog is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Dialog"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "dialog-requires-title",
            "!attributes.Title",
            "Dialog '{attributes.Id}' is missing Title attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Dialog"))
        .with_tag("recommended")
        .with_tag("ui")
        ,

        Rule::new(
            "dialog-requires-width",
            "!attributes.Width",
            "Dialog '{attributes.Id}' is missing Width attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Dialog"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "directory-requires-id",
            "!attributes.Id",
            "Directory is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Directory"))
        .with_tag("required")
        ,

        Rule::new(
            "directory-uppercase-id",
            "attributes.Id =~ /^[a-z]/",
            "Directory Id '{attributes.Id}' should start with uppercase letter (convention)",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Directory"))
        .with_tag("naming")
        ,

        Rule::new(
            "directoryref-requires-id",
            "!attributes.Id",
            "DirectoryRef is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("DirectoryRef"))
        .with_tag("required")
        ,

        Rule::new(
            "duplicate-id-pattern",
            "attributes.Id =~ /^(Id|ID|id)[0-9]+$/",
            "Generic Id pattern '{attributes.Id}' - use descriptive identifiers",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("naming")
        ,

        Rule::new(
            "empty-feature",
            "name == \"Feature\" && !hasChild('ComponentRef') && !hasChild('ComponentGroupRef') && !hasChild('Feature')",
            "Feature '{attributes.Id}' has no components - will install nothing",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Feature"))
        .with_tag("validation")
        ,

        Rule::new(
            "environment-requires-id",
            "!attributes.Id",
            "Environment is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Environment"))
        .with_tag("required")
        ,

        Rule::new(
            "environment-requires-name",
            "!attributes.Name",
            "Environment is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Environment"))
        .with_tag("required")
        ,

        Rule::new(
            "environment-system-path",
            "attributes.Name == \"PATH\" && attributes.System == \"yes\"",
            "Modifying system PATH environment variable - ensure this is necessary",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Environment"))
        .with_tag("awareness")
        ,

        Rule::new(
            "exepackage-detect-condition",
            "!attributes.DetectCondition",
            "ExePackage '{attributes.Id}' is missing DetectCondition - cannot detect if already installed",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("ExePackage"))
        .with_tag("best-practice")
        ,

        Rule::new(
            "exepackage-requires-id",
            "!attributes.Id",
            "ExePackage is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ExePackage"))
        .with_tag("required")
        ,

        Rule::new(
            "exepackage-requires-sourcefile",
            "!attributes.SourceFile && !attributes.Name",
            "ExePackage needs SourceFile or Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ExePackage"))
        .with_tag("required")
        ,

        Rule::new(
            "feature-absent-allow",
            "attributes.Absent == \"disallow\"",
            "Feature '{attributes.Id}' cannot be deselected by user (Absent=disallow)",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Feature"))
        .with_tag("awareness")
        ,

        Rule::new(
            "feature-id-prefix",
            "attributes.Id && !attributes.Id =~ /^(feat|Feat|FEAT|Feature)/",
            "Feature Id '{attributes.Id}' - consider using 'feat' or 'Feature' prefix for clarity",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Feature"))
        .with_tag("naming")
        ,

        Rule::new(
            "feature-level-zero",
            "attributes.Level == \"0\"",
            "Feature '{attributes.Id}' has Level=0 - will be disabled by default",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Feature"))
        .with_tag("awareness")
        ,

        Rule::new(
            "feature-requires-description",
            "!attributes.Description",
            "Feature '{attributes.Id}' is missing Description attribute",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Feature"))
        .with_tag("ui")
        ,

        Rule::new(
            "feature-requires-id",
            "!attributes.Id",
            "Feature is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Feature"))
        .with_tag("required")
        ,

        Rule::new(
            "feature-requires-title",
            "!attributes.Title",
            "Feature '{attributes.Id}' is missing Title attribute - will show blank in UI",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Feature"))
        .with_tag("ui")
        ,

        Rule::new(
            "featureref-requires-id",
            "!attributes.Id",
            "FeatureRef is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("FeatureRef"))
        .with_tag("required")
        ,

        Rule::new(
            "file-hardcoded-path",
            "attributes.Source =~ /^[A-Z]:\\\\/",
            "File '{attributes.Id}' has hardcoded path in Source - use variables like $(var.SourceDir)",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("File"))
        .with_tag("portability")
        ,

        Rule::new(
            "file-hidden-warning",
            "attributes.Hidden == \"yes\"",
            "File '{attributes.Id}' will be installed as hidden",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("File"))
        .with_tag("awareness")
        ,

        Rule::new(
            "file-readonly-warning",
            "attributes.ReadOnly == \"yes\"",
            "File '{attributes.Id}' will be installed read-only",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("File"))
        .with_tag("awareness")
        ,

        Rule::new(
            "file-requires-source",
            "!attributes.Source",
            "File is missing Source attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("File"))
        .with_tag("required")
        ,

        Rule::new(
            "file-system32",
            "attributes.Source =~ /System32/i || attributes.Directory == \"System32Folder\"",
            "Installing to System32 - ensure proper 32/64-bit handling",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("File"))
        .with_tag("security")
        .with_tag("platform")
        ,

        Rule::new(
            "file-vital-warning",
            "attributes.Vital == \"no\"",
            "File '{attributes.Id}' marked Vital=no - installation will continue if this file fails",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("File"))
        .with_tag("awareness")
        ,

        Rule::new(
            "files-requires-include",
            "!attributes.Include",
            "Files element is missing Include attribute (glob pattern)",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Files"))
        .with_tag("required")
        ,

        Rule::new(
            "firewall-exception-requires-id",
            "!attributes.Id",
            "fire:FirewallException is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("FirewallException"))
        .with_tag("required")
        .with_tag("firewall")
        ,

        Rule::new(
            "firewall-exception-requires-name",
            "!attributes.Name",
            "fire:FirewallException is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("FirewallException"))
        .with_tag("required")
        .with_tag("firewall")
        ,

        Rule::new(
            "firewall-profile-all",
            "attributes.Profile == \"all\"",
            "fire:FirewallException applies to all profiles - consider restricting to specific profiles",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("FirewallException"))
        .with_tag("security")
        .with_tag("firewall")
        ,

        Rule::new(
            "firewall-scope-any",
            "attributes.Scope == \"any\"",
            "fire:FirewallException allows any scope - consider restricting to subnet or specific addresses",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("FirewallException"))
        .with_tag("security")
        .with_tag("firewall")
        ,

        Rule::new(
            "fragment-requires-id",
            "!attributes.Id",
            "Fragment should have Id attribute for better organization",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Fragment"))
        .with_tag("best-practice")
        ,

        Rule::new(
            "guid-invalid-format",
            "attributes.Guid && attributes.Guid != \"*\" && !isGuid(attributes.Guid)",
            "Invalid GUID format in '{attributes.Id}' - use format {12345678-1234-1234-1234-123456789012}",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Component"))
        .with_tag("validation")
        ,

        Rule::new(
            "hardcoded-description",
            "attributes.Description && !attributes.Description =~ /^\\!/",
            "Hardcoded Description - consider using localization",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Feature"))
        .with_tag("localization")
        ,

        Rule::new(
            "hardcoded-english-text",
            "attributes.Message =~ /^[A-Za-z]/ && !attributes.Message =~ /^\\!/",
            "Hardcoded text in Message - consider using localization !(loc.StringId)",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Condition"))
        .with_tag("localization")
        ,

        Rule::new(
            "hardcoded-installdir",
            "attributes.Value =~ /C:\\\\Program Files/i",
            "Hardcoded Program Files path - use [ProgramFilesFolder] or [ProgramFiles64Folder]",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Property"))
        .with_tag("portability")
        ,

        Rule::new(
            "hardcoded-title",
            "attributes.Title && !attributes.Title =~ /^\\!/",
            "Hardcoded Title '{attributes.Title}' - consider using localization",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Feature"))
        .with_tag("localization")
        ,

        Rule::new(
            "ice03-invalid-identifier",
            "attributes.Id =~ /[^A-Za-z0-9_\\.]/",
            "ICE03: Id '{attributes.Id}' contains invalid characters - use only A-Z, a-z, 0-9, _, .",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), None)
        .with_tag("ice")
        .with_tag("validation")
        ,

        Rule::new(
            "ice06-feature-missing-display",
            "name == \"Feature\" && !attributes.Display",
            "ICE06: Feature '{attributes.Id}' should have Display attribute for feature tree order",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Feature"))
        .with_tag("ice")
        ,

        Rule::new(
            "ice09-component-guid",
            "attributes.Guid && attributes.Guid != \"*\" && attributes.Guid =~ /[G-Zg-z]/",
            "ICE09: Component '{attributes.Id}' has invalid GUID characters",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Component"))
        .with_tag("ice")
        .with_tag("validation")
        ,

        Rule::new(
            "ice18-empty-keypath",
            "name == \"Component\" && !hasChild('File') && !hasChild('RegistryValue') && !hasChild('ODBCDataSource')",
            "ICE18: Component '{attributes.Id}' has no valid KeyPath resource",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Component"))
        .with_tag("ice")
        .with_tag("validation")
        ,

        Rule::new(
            "ice21-ca-needs-source",
            "name == \"CustomAction\" && !attributes.BinaryRef && !attributes.FileRef && !attributes.Property && !attributes.Directory",
            "ICE21: CustomAction '{attributes.Id}' needs BinaryRef, FileRef, Property, or Directory",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("CustomAction"))
        .with_tag("ice")
        .with_tag("validation")
        ,

        Rule::new(
            "ice24-productcode-format",
            "attributes.ProductCode && !isGuid(attributes.ProductCode)",
            "ICE24: ProductCode must be a valid GUID format",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Package"))
        .with_tag("ice")
        .with_tag("validation")
        ,

        Rule::new(
            "ice30-shortcut-target",
            "name == \"Shortcut\" && !attributes.Target && !attributes.Advertise",
            "ICE30: Shortcut '{attributes.Id}' needs Target attribute or Advertise=yes",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Shortcut"))
        .with_tag("ice")
        .with_tag("validation")
        ,

        Rule::new(
            "ice33-registry-keypath",
            "name == \"RegistryValue\" && attributes.KeyPath == \"yes\"",
            "ICE33: RegistryValue is KeyPath - ensure component has no File children",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("RegistryValue"))
        .with_tag("ice")
        .with_tag("awareness")
        ,

        Rule::new(
            "ice38-component-ref-check",
            "name == \"ComponentRef\"",
            "ICE38: Ensure Component is not referenced by multiple features without Shared=yes",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("ComponentRef"))
        .with_tag("ice")
        .with_tag("awareness")
        ,

        Rule::new(
            "ice43-shortcut-nonadvertised",
            "attributes.Advertise == \"no\"",
            "ICE43: Non-advertised Shortcut '{attributes.Id}' - must have valid Target",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Shortcut"))
        .with_tag("ice")
        ,

        Rule::new(
            "ice46-ca-property-defined",
            "attributes.Property && !attributes.Value && !attributes.ExeCommand",
            "ICE46: CustomAction '{attributes.Id}' sets Property but no Value specified",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("CustomAction"))
        .with_tag("ice")
        ,

        Rule::new(
            "ice48-directory-parent",
            "name == \"Directory\" && !attributes.Id =~ /^(TARGETDIR|ProgramFilesFolder|CommonFilesFolder|SystemFolder)/",
            "ICE48: Verify Directory '{attributes.Id}' has valid parent in directory tree",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Directory"))
        .with_tag("ice")
        .with_tag("awareness")
        ,

        Rule::new(
            "ice57-allusers-scope",
            "attributes.InstallScope && attributes.InstallPrivileges",
            "ICE57: Verify InstallScope and InstallPrivileges are consistent",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Package"))
        .with_tag("ice")
        ,

        Rule::new(
            "ice60-file-version",
            "name == \"File\" && attributes.DefaultVersion",
            "ICE60: File with DefaultVersion - ensure file actually has version resource",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("File"))
        .with_tag("ice")
        ,

        Rule::new(
            "ice61-upgradecode",
            "name == \"Package\" && attributes.UpgradeCode",
            "ICE61: Package has UpgradeCode - verify upgrade scenarios are tested",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Package"))
        .with_tag("ice")
        .with_tag("awareness")
        ,

        Rule::new(
            "ice64-removefile-dir",
            "!attributes.Directory && !attributes.Property",
            "ICE64: RemoveFile '{attributes.Id}' needs Directory or Property attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RemoveFile"))
        .with_tag("ice")
        .with_tag("validation")
        ,

        Rule::new(
            "ice67-targetdir",
            "attributes.Id == \"TARGETDIR\" && attributes.Name && attributes.Name != \"SourceDir\"",
            "ICE67: TARGETDIR directory Name should be 'SourceDir'",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Directory"))
        .with_tag("ice")
        ,

        Rule::new(
            "ice69-merge-requires-id",
            "!attributes.Id",
            "Merge is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Merge"))
        .with_tag("required")
        .with_tag("ice")
        ,

        Rule::new(
            "ice69-merge-requires-sourcefile",
            "!attributes.SourceFile",
            "Merge is missing SourceFile attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Merge"))
        .with_tag("required")
        .with_tag("ice")
        ,

        Rule::new(
            "ice71-media-sequence",
            "name == \"Media\" && attributes.Id == \"1\"",
            "ICE71: Media Id=1 is required - ensure cabinet sequence is correct",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Media"))
        .with_tag("ice")
        ,

        Rule::new(
            "ice80-64bit-component",
            "attributes.Win64 == \"yes\" && !attributes.Directory =~ /64/",
            "ICE80: 64-bit Component '{attributes.Id}' should use 64-bit directory",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Component"))
        .with_tag("ice")
        .with_tag("platform")
        ,

        Rule::new(
            "ice82-action-condition",
            "name == \"InstallExecuteSequence\" || name == \"InstallUISequence\"",
            "ICE82: Verify action sequence conditions are valid",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("ice")
        ,

        Rule::new(
            "ice83-assembly-requires-file",
            "!attributes.File",
            "MsiAssembly is missing File attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("MsiAssembly"))
        .with_tag("required")
        .with_tag("ice")
        ,

        Rule::new(
            "ice87-summary-codepage",
            "name == \"Package\" && !attributes.Codepage",
            "ICE87: Package should specify Codepage for proper localization",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Package"))
        .with_tag("ice")
        .with_tag("localization")
        ,

        Rule::new(
            "ice90-shortcut-workdir",
            "name == \"Shortcut\" && !attributes.WorkingDirectory",
            "ICE90: Shortcut '{attributes.Id}' - consider setting WorkingDirectory",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Shortcut"))
        .with_tag("ice")
        ,

        Rule::new(
            "ice91-file-hash",
            "name == \"File\" && !attributes.Checksum",
            "ICE91: Consider adding Checksum=yes to File for integrity verification",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("File"))
        .with_tag("ice")
        .with_tag("security")
        ,

        Rule::new(
            "icon-requires-id",
            "!attributes.Id",
            "Icon is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Icon"))
        .with_tag("required")
        ,

        Rule::new(
            "icon-requires-sourcefile",
            "!attributes.SourceFile",
            "Icon is missing SourceFile attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Icon"))
        .with_tag("required")
        ,

        Rule::new(
            "id-contains-spaces",
            "attributes.Id =~ / /",
            "Id '{attributes.Id}' contains spaces - use underscores or camelCase",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), None)
        .with_tag("naming")
        ,

        Rule::new(
            "id-starts-with-number",
            "attributes.Id =~ /^[0-9]/",
            "Id '{attributes.Id}' starts with a number - identifiers should start with a letter",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), None)
        .with_tag("naming")
        ,

        Rule::new(
            "id-too-long",
            "attributes.Id =~ /^.{73,}/",
            "Id '{attributes.Id}' exceeds 72 characters - may cause issues in MSI tables",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), None)
        .with_tag("validation")
        ,

        Rule::new(
            "iis-certificate-requires-id",
            "!attributes.Id",
            "iis:Certificate is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Certificate"))
        .with_tag("required")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-webaddress-requires-id",
            "!attributes.Id",
            "iis:WebAddress is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("WebAddress"))
        .with_tag("required")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-webapplication-requires-id",
            "!attributes.Id",
            "iis:WebApplication is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("WebApplication"))
        .with_tag("required")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-webapplication-requires-name",
            "!attributes.Name",
            "iis:WebApplication is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("WebApplication"))
        .with_tag("required")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-webapppool-identity",
            "attributes.Identity == \"LocalSystem\"",
            "iis:WebAppPool '{attributes.Id}' runs as LocalSystem - use ApplicationPoolIdentity or specific account",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("WebAppPool"))
        .with_tag("security")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-webapppool-requires-id",
            "!attributes.Id",
            "iis:WebAppPool is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("WebAppPool"))
        .with_tag("required")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-webapppool-requires-name",
            "!attributes.Name",
            "iis:WebAppPool is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("WebAppPool"))
        .with_tag("required")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-website-requires-description",
            "!attributes.Description",
            "iis:WebSite '{attributes.Id}' is missing Description attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("WebSite"))
        .with_tag("recommended")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-website-requires-directory",
            "!attributes.Directory",
            "iis:WebSite '{attributes.Id}' is missing Directory attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("WebSite"))
        .with_tag("required")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-website-requires-id",
            "!attributes.Id",
            "iis:WebSite is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("WebSite"))
        .with_tag("required")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-webvirtualdir-requires-alias",
            "!attributes.Alias",
            "iis:WebVirtualDir is missing Alias attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("WebVirtualDir"))
        .with_tag("required")
        .with_tag("iis")
        ,

        Rule::new(
            "iis-webvirtualdir-requires-id",
            "!attributes.Id",
            "iis:WebVirtualDir is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("WebVirtualDir"))
        .with_tag("required")
        .with_tag("iis")
        ,

        Rule::new(
            "large-file-count-warning",
            "name == \"ComponentGroup\" && countChildren('Component') > 100",
            "ComponentGroup has many components - consider splitting for better organization",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("ComponentGroup"))
        .with_tag("performance")
        ,

        Rule::new(
            "launch-requires-condition",
            "!attributes.Condition",
            "Launch is missing Condition attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Launch"))
        .with_tag("recommended")
        ,

        Rule::new(
            "listbox-requires-property",
            "!attributes.Property",
            "ListBox is missing Property attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ListBox"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "listitem-requires-value",
            "!attributes.Value",
            "ListItem is missing Value attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ListItem"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "majorupgrade-downgrade-error",
            "!attributes.DowngradeErrorMessage",
            "MajorUpgrade should have DowngradeErrorMessage to prevent downgrade installations",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("MajorUpgrade"))
        .with_tag("best-practice")
        ,

        Rule::new(
            "media-requires-id",
            "!attributes.Id",
            "Media is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Media"))
        .with_tag("required")
        ,

        Rule::new(
            "mediatemplate-recommended",
            "name == \"Media\"",
            "Consider using MediaTemplate instead of Media for simpler cabinet management",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Media"))
        .with_tag("best-practice")
        ,

        Rule::new(
            "missing-install-condition",
            "name == \"Package\" && !hasChild('Condition')",
            "Consider adding installation conditions (OS version, prerequisites)",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Package"))
        .with_tag("best-practice")
        ,

        Rule::new(
            "missing-majorupgrade",
            "name == \"Package\" && !hasChild('MajorUpgrade')",
            "Package should have a MajorUpgrade element for proper upgrade handling",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Package"))
        .with_tag("best-practice")
        .with_tag("upgrade")
        ,

        Rule::new(
            "msipackage-requires-id",
            "!attributes.Id",
            "MsiPackage is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("MsiPackage"))
        .with_tag("required")
        ,

        Rule::new(
            "msipackage-requires-sourcefile",
            "!attributes.SourceFile && !attributes.Name",
            "MsiPackage needs SourceFile or Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("MsiPackage"))
        .with_tag("required")
        ,

        Rule::new(
            "netfx-dotnetcorecheck",
            "name == \"Package\" && !hasChild('Condition')",
            "Consider adding .NET runtime prerequisite check using netfx extension",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Package"))
        .with_tag("best-practice")
        .with_tag("netfx")
        ,

        Rule::new(
            "netfx-nativeimagereq-id",
            "!attributes.Id",
            "netfx:NativeImage is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("NativeImage"))
        .with_tag("required")
        .with_tag("netfx")
        ,

        Rule::new(
            "package-compressed-recommended",
            "!attributes.Compressed",
            "Package should specify Compressed attribute for explicit cab embedding control",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Package"))
        .with_tag("best-practice")
        ,

        Rule::new(
            "package-invalid-version",
            "attributes.Version =~ /^[0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+/ && attributes.Version =~ /\\.[0-9]{6,}/",
            "Package Version field exceeds 65535 limit - use format X.X.X.X where each part <= 65535",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Package"))
        .with_tag("validation")
        ,

        Rule::new(
            "package-requires-manufacturer",
            "!attributes.Manufacturer",
            "Package is missing Manufacturer attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Package"))
        .with_tag("required")
        ,

        Rule::new(
            "package-requires-name",
            "!attributes.Name",
            "Package is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Package"))
        .with_tag("required")
        ,

        Rule::new(
            "package-requires-upgradecode",
            "!attributes.UpgradeCode",
            "Package is missing UpgradeCode attribute - required for upgrades",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Package"))
        .with_tag("required")
        .with_tag("upgrade")
        ,

        Rule::new(
            "package-requires-version",
            "!attributes.Version",
            "Package is missing Version attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Package"))
        .with_tag("recommended")
        ,

        Rule::new(
            "payload-requires-sourcefile",
            "!attributes.SourceFile && !attributes.Name",
            "Payload needs SourceFile or Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Payload"))
        .with_tag("required")
        ,

        Rule::new(
            "permission-everyone",
            "attributes.User == \"Everyone\" || attributes.User == \"*S-1-1-0\"",
            "Permission grants access to Everyone - review security implications",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Permission"))
        .with_tag("security")
        ,

        Rule::new(
            "productcode-invalid-format",
            "attributes.ProductCode && !isGuid(attributes.ProductCode)",
            "Invalid ProductCode GUID format - use format {12345678-1234-1234-1234-123456789012}",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Package"))
        .with_tag("validation")
        ,

        Rule::new(
            "property-hidden",
            "attributes.Hidden == \"yes\"",
            "Property '{attributes.Id}' is hidden from logs - ensure this is intentional for sensitive data",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Property"))
        .with_tag("awareness")
        ,

        Rule::new(
            "property-lowercase-id",
            "attributes.Id =~ /^[a-z]/ && attributes.Secure != \"yes\"",
            "Property '{attributes.Id}' starts with lowercase - will not be passed to server side (use UPPERCASE for public properties)",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Property"))
        .with_tag("naming")
        ,

        Rule::new(
            "property-requires-id",
            "!attributes.Id",
            "Property is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Property"))
        .with_tag("required")
        ,

        Rule::new(
            "property-secure-recommendation",
            "attributes.Id =~ /^[A-Z]/ && !attributes.Secure",
            "Public Property '{attributes.Id}' should consider Secure=yes to restrict command-line setting",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Property"))
        .with_tag("security")
        ,

        Rule::new(
            "propertyref-requires-id",
            "!attributes.Id",
            "PropertyRef is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("PropertyRef"))
        .with_tag("required")
        ,

        Rule::new(
            "publish-requires-control",
            "!attributes.Control",
            "Publish is missing Control attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Publish"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "publish-requires-dialog",
            "!attributes.Dialog",
            "Publish is missing Dialog attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Publish"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "radiobutton-requires-value",
            "!attributes.Value",
            "RadioButton is missing Value attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RadioButton"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "radiobuttongroup-requires-property",
            "!attributes.Property",
            "RadioButtonGroup is missing Property attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RadioButtonGroup"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "registry-component-no-keypath",
            "name == \"Component\" && hasChild('RegistryValue') && !hasChild('File')",
            "Component with only registry values - ensure a registry value is marked as KeyPath",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Component"))
        .with_tag("best-practice")
        ,

        Rule::new(
            "registry-hklm-per-user",
            "attributes.Root == \"HKLM\"",
            "Registry key uses HKLM - ensure this is appropriate (consider HKCU for per-user settings)",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("RegistryValue"))
        .with_tag("awareness")
        ,

        Rule::new(
            "registry-run-key",
            "attributes.Key =~ /SOFTWARE\\\\Microsoft\\\\Windows\\\\CurrentVersion\\\\Run/i",
            "Auto-run registry key detected - ensure this is intentional",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("RegistryValue"))
        .with_tag("security")
        .with_tag("awareness")
        ,

        Rule::new(
            "registrykey-requires-key",
            "!attributes.Key",
            "RegistryKey is missing Key attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RegistryKey"))
        .with_tag("required")
        ,

        Rule::new(
            "registrykey-requires-root",
            "!attributes.Root",
            "RegistryKey is missing Root attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RegistryKey"))
        .with_tag("required")
        ,

        Rule::new(
            "registrysearch-requires-id",
            "!attributes.Id",
            "RegistrySearch is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RegistrySearch"))
        .with_tag("required")
        ,

        Rule::new(
            "registrysearch-requires-key",
            "!attributes.Key",
            "RegistrySearch is missing Key attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RegistrySearch"))
        .with_tag("required")
        ,

        Rule::new(
            "registrysearch-requires-root",
            "!attributes.Root",
            "RegistrySearch is missing Root attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RegistrySearch"))
        .with_tag("required")
        ,

        Rule::new(
            "registryvalue-requires-key",
            "!attributes.Key",
            "RegistryValue is missing Key attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RegistryValue"))
        .with_tag("required")
        ,

        Rule::new(
            "registryvalue-requires-root",
            "!attributes.Root",
            "RegistryValue is missing Root attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RegistryValue"))
        .with_tag("required")
        ,

        Rule::new(
            "registryvalue-requires-type",
            "!attributes.Type",
            "RegistryValue is missing Type attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("RegistryValue"))
        .with_tag("recommended")
        ,

        Rule::new(
            "removefile-requires-id",
            "!attributes.Id",
            "RemoveFile is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RemoveFile"))
        .with_tag("required")
        ,

        Rule::new(
            "removefile-requires-on",
            "!attributes.On",
            "RemoveFile is missing On attribute (install/uninstall/both)",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RemoveFile"))
        .with_tag("required")
        ,

        Rule::new(
            "removefolder-requires-id",
            "!attributes.Id",
            "RemoveFolder is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RemoveFolder"))
        .with_tag("required")
        ,

        Rule::new(
            "removefolder-requires-on",
            "!attributes.On",
            "RemoveFolder is missing On attribute (install/uninstall/both)",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RemoveFolder"))
        .with_tag("required")
        ,

        Rule::new(
            "sequence-custom-after",
            "name == \"Custom\" && !attributes.After && !attributes.Before && !attributes.Sequence",
            "Custom action in sequence needs After, Before, or Sequence attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Custom"))
        .with_tag("required")
        ,

        Rule::new(
            "servicecontrol-requires-id",
            "!attributes.Id",
            "ServiceControl is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ServiceControl"))
        .with_tag("required")
        ,

        Rule::new(
            "servicecontrol-requires-name",
            "!attributes.Name",
            "ServiceControl is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ServiceControl"))
        .with_tag("required")
        ,

        Rule::new(
            "serviceinstall-auto-start",
            "attributes.Start == \"auto\"",
            "ServiceInstall '{attributes.Name}' starts automatically - ensure this is necessary",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("ServiceInstall"))
        .with_tag("awareness")
        ,

        Rule::new(
            "serviceinstall-localsystem",
            "attributes.Account == \"LocalSystem\"",
            "ServiceInstall '{attributes.Name}' runs as LocalSystem - consider using a less privileged account",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("ServiceInstall"))
        .with_tag("security")
        ,

        Rule::new(
            "serviceinstall-requires-id",
            "!attributes.Id",
            "ServiceInstall is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ServiceInstall"))
        .with_tag("required")
        ,

        Rule::new(
            "serviceinstall-requires-name",
            "!attributes.Name",
            "ServiceInstall is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ServiceInstall"))
        .with_tag("required")
        ,

        Rule::new(
            "serviceinstall-requires-start",
            "!attributes.Start",
            "ServiceInstall '{attributes.Name}' is missing Start attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("ServiceInstall"))
        .with_tag("recommended")
        ,

        Rule::new(
            "serviceinstall-requires-type",
            "!attributes.Type",
            "ServiceInstall '{attributes.Name}' is missing Type attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("ServiceInstall"))
        .with_tag("recommended")
        ,

        Rule::new(
            "setproperty-requires-id",
            "!attributes.Id",
            "SetProperty is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("SetProperty"))
        .with_tag("required")
        ,

        Rule::new(
            "setproperty-requires-value",
            "!attributes.Value",
            "SetProperty is missing Value attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("SetProperty"))
        .with_tag("required")
        ,

        Rule::new(
            "shortcut-no-component",
            "name == \"Shortcut\"",
            "Shortcuts must be in a Component to be installed properly",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Shortcut"))
        .with_tag("awareness")
        ,

        Rule::new(
            "shortcut-requires-directory",
            "!attributes.Directory",
            "Shortcut '{attributes.Id}' is missing Directory attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Shortcut"))
        .with_tag("recommended")
        ,

        Rule::new(
            "shortcut-requires-id",
            "!attributes.Id",
            "Shortcut is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Shortcut"))
        .with_tag("required")
        ,

        Rule::new(
            "shortcut-requires-name",
            "!attributes.Name",
            "Shortcut is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Shortcut"))
        .with_tag("required")
        ,

        Rule::new(
            "sql-database-requires-database",
            "!attributes.Database",
            "sql:SqlDatabase is missing Database attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("SqlDatabase"))
        .with_tag("required")
        .with_tag("sql")
        ,

        Rule::new(
            "sql-database-requires-id",
            "!attributes.Id",
            "sql:SqlDatabase is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("SqlDatabase"))
        .with_tag("required")
        .with_tag("sql")
        ,

        Rule::new(
            "sql-database-requires-server",
            "!attributes.Server",
            "sql:SqlDatabase is missing Server attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("SqlDatabase"))
        .with_tag("required")
        .with_tag("sql")
        ,

        Rule::new(
            "sql-hardcoded-password",
            "attributes.Password && !attributes.Password =~ /^\\[/",
            "sql:SqlDatabase has hardcoded password - use property reference [PROPERTY]",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("SqlDatabase"))
        .with_tag("security")
        .with_tag("sql")
        ,

        Rule::new(
            "sql-script-requires-binaryref",
            "!attributes.BinaryRef && !attributes.SqlDb",
            "sql:SqlScript needs BinaryRef or SqlDb attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("SqlScript"))
        .with_tag("required")
        .with_tag("sql")
        ,

        Rule::new(
            "sql-script-requires-id",
            "!attributes.Id",
            "sql:SqlScript is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("SqlScript"))
        .with_tag("required")
        .with_tag("sql")
        ,

        Rule::new(
            "sql-string-requires-id",
            "!attributes.Id",
            "sql:SqlString is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("SqlString"))
        .with_tag("required")
        .with_tag("sql")
        ,

        Rule::new(
            "sql-string-requires-sql",
            "!attributes.SQL",
            "sql:SqlString is missing SQL attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("SqlString"))
        .with_tag("required")
        .with_tag("sql")
        ,

        Rule::new(
            "standarddirectory-requires-id",
            "!attributes.Id",
            "StandardDirectory is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("StandardDirectory"))
        .with_tag("required")
        ,

        Rule::new(
            "textstyle-requires-facename",
            "!attributes.FaceName",
            "TextStyle '{attributes.Id}' is missing FaceName attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("TextStyle"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "textstyle-requires-id",
            "!attributes.Id",
            "TextStyle is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("TextStyle"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "textstyle-requires-size",
            "!attributes.Size",
            "TextStyle '{attributes.Id}' is missing Size attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("TextStyle"))
        .with_tag("required")
        .with_tag("ui")
        ,

        Rule::new(
            "ui-requires-id",
            "!attributes.Id",
            "UI element is missing Id attribute",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("UI"))
        .with_tag("recommended")
        ,

        Rule::new(
            "uiref-requires-id",
            "!attributes.Id",
            "UIRef is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("UIRef"))
        .with_tag("required")
        ,

        Rule::new(
            "upgrade-requires-id",
            "!attributes.Id",
            "Upgrade is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Upgrade"))
        .with_tag("required")
        ,

        Rule::new(
            "upgradecode-invalid-format",
            "attributes.UpgradeCode && !isGuid(attributes.UpgradeCode)",
            "Invalid UpgradeCode GUID format - use format {12345678-1234-1234-1234-123456789012}",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Package"))
        .with_tag("validation")
        ,

        Rule::new(
            "util-closeapplication-requires-id",
            "!attributes.Id",
            "util:CloseApplication is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("CloseApplication"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-closeapplication-requires-target",
            "!attributes.Target",
            "util:CloseApplication is missing Target attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("CloseApplication"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-filesearch-requires-id",
            "!attributes.Id",
            "util:FileSearch is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("FileSearch"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-group-requires-id",
            "!attributes.Id",
            "util:Group is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Group"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-group-requires-name",
            "!attributes.Name",
            "util:Group is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Group"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-permissionex-requires-id",
            "!attributes.Id",
            "util:PermissionEx is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("PermissionEx"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-productsearch-requires-id",
            "!attributes.Id",
            "util:ProductSearch is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ProductSearch"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-registrysearch-requires-id",
            "!attributes.Id",
            "util:RegistrySearch is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("RegistrySearch"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-serviceconfig-requires-servicename",
            "!attributes.ServiceName",
            "util:ServiceConfig is missing ServiceName attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("ServiceConfig"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-user-hardcoded-password",
            "attributes.Password && !attributes.Password =~ /^\\[/",
            "util:User has hardcoded password - use property reference [PROPERTY]",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("User"))
        .with_tag("security")
        .with_tag("util")
        ,

        Rule::new(
            "util-user-requires-id",
            "!attributes.Id",
            "util:User is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("User"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-user-requires-name",
            "!attributes.Name",
            "util:User is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("User"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-xmlconfig-requires-id",
            "!attributes.Id",
            "util:XmlConfig is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("XmlConfig"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-xmlfile-requires-file",
            "!attributes.File",
            "util:XmlFile is missing File attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("XmlFile"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "util-xmlfile-requires-id",
            "!attributes.Id",
            "util:XmlFile is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("XmlFile"))
        .with_tag("required")
        .with_tag("util")
        ,

        Rule::new(
            "v3-namespace",
            "attributes.xmlns =~ /schemas\\.microsoft\\.com\\/wix/",
            "WiX v3 namespace detected - consider migrating to WiX v4",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), Some("Wix"))
        .with_tag("migration")
        ,

        Rule::new(
            "v4-namespace-missing",
            "name == \"Wix\" && !attributes.xmlns =~ /wixtoolset\\.org/",
            "Missing WiX v4 namespace - add xmlns=\"http://wixtoolset.org/schemas/v4/wxs\"",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), Some("Wix"))
        .with_tag("migration")
        ,

        Rule::new(
            "variable-requires-name",
            "!attributes.Name",
            "Variable is missing Name attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("Variable"))
        .with_tag("required")
        ,

        Rule::new(
            "wixvariable-requires-id",
            "!attributes.Id",
            "WixVariable is missing Id attribute",
        )
        .with_severity(Severity::Error)
        .with_target(Some("element"), Some("WixVariable"))
        .with_tag("required")
        ,

    ]
}
