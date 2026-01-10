//! WiX project scaffolder with templates for common scenarios
//!
//! Generates new WiX projects with appropriate structure and skeleton files.
//!
//! # Example
//!
//! ```no_run
//! use wix_init::{Project, Template};
//! use std::path::Path;
//!
//! let project = Project::new("MyInstaller", Template::SimpleMsi);
//! project.create(Path::new("./my-installer")).unwrap();
//! ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

/// Error types for project creation
#[derive(Error, Debug)]
pub enum InitError {
    #[error("Directory already exists: {0}")]
    DirectoryExists(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid project name: {0}")]
    InvalidName(String),
}

/// Project template type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Template {
    /// Simple MSI with single component
    SimpleMsi,
    /// MSI with multiple features
    FeatureMsi,
    /// Burn bootstrapper bundle
    Bootstrapper,
    /// MSI with custom actions
    CustomAction,
    /// MSI with service installation
    Service,
    /// MSI with UI dialogs
    CustomUi,
    /// Merge module
    MergeModule,
    /// Minimal skeleton
    Minimal,
}

impl Template {
    pub fn description(&self) -> &'static str {
        match self {
            Self::SimpleMsi => "Simple MSI with a single component and file",
            Self::FeatureMsi => "MSI with multiple selectable features",
            Self::Bootstrapper => "Burn bootstrapper bundle for chained installs",
            Self::CustomAction => "MSI with custom action example",
            Self::Service => "MSI that installs a Windows service",
            Self::CustomUi => "MSI with custom UI dialogs",
            Self::MergeModule => "Reusable merge module",
            Self::Minimal => "Minimal skeleton with just the essentials",
        }
    }

    pub fn all() -> &'static [Template] {
        &[
            Template::SimpleMsi,
            Template::FeatureMsi,
            Template::Bootstrapper,
            Template::CustomAction,
            Template::Service,
            Template::CustomUi,
            Template::MergeModule,
            Template::Minimal,
        ]
    }
}

impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SimpleMsi => write!(f, "simple-msi"),
            Self::FeatureMsi => write!(f, "feature-msi"),
            Self::Bootstrapper => write!(f, "bootstrapper"),
            Self::CustomAction => write!(f, "custom-action"),
            Self::Service => write!(f, "service"),
            Self::CustomUi => write!(f, "custom-ui"),
            Self::MergeModule => write!(f, "merge-module"),
            Self::Minimal => write!(f, "minimal"),
        }
    }
}

impl std::str::FromStr for Template {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "simple-msi" | "simple" | "msi" => Ok(Template::SimpleMsi),
            "feature-msi" | "features" => Ok(Template::FeatureMsi),
            "bootstrapper" | "bundle" | "burn" => Ok(Template::Bootstrapper),
            "custom-action" | "ca" => Ok(Template::CustomAction),
            "service" | "svc" => Ok(Template::Service),
            "custom-ui" | "ui" => Ok(Template::CustomUi),
            "merge-module" | "merge" | "msm" => Ok(Template::MergeModule),
            "minimal" | "min" => Ok(Template::Minimal),
            _ => Err(format!("Unknown template: {}", s)),
        }
    }
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Project name (used for directory and package name)
    pub name: String,
    /// Template type
    pub template: Template,
    /// Manufacturer name
    pub manufacturer: String,
    /// Version string
    pub version: String,
    /// Description
    pub description: String,
    /// WiX version to target (v4, v5)
    pub wix_version: WixVersion,
}

/// WiX toolset version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WixVersion {
    V4,
    #[default]
    V5,
}

impl std::fmt::Display for WixVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V4 => write!(f, "v4"),
            Self::V5 => write!(f, "v5"),
        }
    }
}

impl Project {
    /// Create a new project with defaults
    pub fn new(name: impl Into<String>, template: Template) -> Self {
        let name = name.into();
        Self {
            description: format!("{} installer", name),
            name,
            template,
            manufacturer: "My Company".to_string(),
            version: "1.0.0".to_string(),
            wix_version: WixVersion::default(),
        }
    }

    /// Set manufacturer
    pub fn with_manufacturer(mut self, manufacturer: impl Into<String>) -> Self {
        self.manufacturer = manufacturer.into();
        self
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set WiX version
    pub fn with_wix_version(mut self, version: WixVersion) -> Self {
        self.wix_version = version;
        self
    }

    /// Validate project configuration
    pub fn validate(&self) -> Result<(), InitError> {
        if self.name.is_empty() {
            return Err(InitError::InvalidName("Name cannot be empty".to_string()));
        }
        if self.name.contains(|c: char| !c.is_alphanumeric() && c != '_' && c != '-') {
            return Err(InitError::InvalidName(
                "Name can only contain alphanumeric characters, underscores, and hyphens".to_string(),
            ));
        }
        Ok(())
    }

    /// Create the project directory and files
    pub fn create(&self, base_path: &Path) -> Result<CreatedProject, InitError> {
        self.validate()?;

        let project_path = base_path.join(&self.name);

        if project_path.exists() {
            return Err(InitError::DirectoryExists(
                project_path.to_string_lossy().to_string(),
            ));
        }

        // Create directories
        fs::create_dir_all(&project_path)?;
        fs::create_dir_all(project_path.join("src"))?;

        let mut created_files = Vec::new();

        // Generate files based on template
        match self.template {
            Template::SimpleMsi => {
                created_files.extend(self.create_simple_msi(&project_path)?);
            }
            Template::FeatureMsi => {
                created_files.extend(self.create_feature_msi(&project_path)?);
            }
            Template::Bootstrapper => {
                created_files.extend(self.create_bootstrapper(&project_path)?);
            }
            Template::CustomAction => {
                created_files.extend(self.create_custom_action(&project_path)?);
            }
            Template::Service => {
                created_files.extend(self.create_service(&project_path)?);
            }
            Template::CustomUi => {
                created_files.extend(self.create_custom_ui(&project_path)?);
            }
            Template::MergeModule => {
                created_files.extend(self.create_merge_module(&project_path)?);
            }
            Template::Minimal => {
                created_files.extend(self.create_minimal(&project_path)?);
            }
        }

        // Create common files
        created_files.push(self.create_wixproj(&project_path)?);
        created_files.push(self.create_gitignore(&project_path)?);

        Ok(CreatedProject {
            path: project_path,
            files: created_files,
        })
    }

    fn create_simple_msi(&self, path: &Path) -> Result<Vec<String>, InitError> {
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="{name}"
             Manufacturer="{manufacturer}"
             Version="{version}"
             UpgradeCode="PUT-GUID-HERE">

        <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
        <MediaTemplate EmbedCab="yes" />

        <Feature Id="Main" Title="Main Feature" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
        </Feature>

        <StandardDirectory Id="ProgramFilesFolder">
            <Directory Id="INSTALLFOLDER" Name="{name}">
                <Component Id="MainComponent" Guid="*">
                    <!-- Add your files here -->
                    <File Source="$(var.SourceDir)\YourApp.exe" />
                </Component>
            </Directory>
        </StandardDirectory>

        <ComponentGroup Id="ProductComponents">
            <ComponentRef Id="MainComponent" />
        </ComponentGroup>
    </Package>
</Wix>
"#,
            name = self.name,
            manufacturer = self.manufacturer,
            version = self.version,
        );

        let file_path = path.join("src").join("Product.wxs");
        self.write_file(&file_path, &content)?;
        Ok(vec!["src/Product.wxs".to_string()])
    }

    fn create_feature_msi(&self, path: &Path) -> Result<Vec<String>, InitError> {
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="{name}"
             Manufacturer="{manufacturer}"
             Version="{version}"
             UpgradeCode="PUT-GUID-HERE">

        <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
        <MediaTemplate EmbedCab="yes" />

        <!-- Main application feature (always installed) -->
        <Feature Id="MainFeature" Title="Main Application" Level="1">
            <ComponentGroupRef Id="MainComponents" />
        </Feature>

        <!-- Optional documentation feature -->
        <Feature Id="DocsFeature" Title="Documentation" Level="1000">
            <ComponentGroupRef Id="DocsComponents" />
        </Feature>

        <!-- Optional examples feature -->
        <Feature Id="ExamplesFeature" Title="Examples" Level="1000">
            <ComponentGroupRef Id="ExamplesComponents" />
        </Feature>

        <StandardDirectory Id="ProgramFilesFolder">
            <Directory Id="INSTALLFOLDER" Name="{name}">
                <Directory Id="DocsDir" Name="docs" />
                <Directory Id="ExamplesDir" Name="examples" />
            </Directory>
        </StandardDirectory>

        <ComponentGroup Id="MainComponents" Directory="INSTALLFOLDER">
            <Component Id="MainApp" Guid="*">
                <File Source="$(var.SourceDir)\App.exe" />
            </Component>
        </ComponentGroup>

        <ComponentGroup Id="DocsComponents" Directory="DocsDir">
            <Component Id="Readme" Guid="*">
                <File Source="$(var.SourceDir)\docs\README.txt" />
            </Component>
        </ComponentGroup>

        <ComponentGroup Id="ExamplesComponents" Directory="ExamplesDir">
            <Component Id="Example1" Guid="*">
                <File Source="$(var.SourceDir)\examples\example.txt" />
            </Component>
        </ComponentGroup>
    </Package>
</Wix>
"#,
            name = self.name,
            manufacturer = self.manufacturer,
            version = self.version,
        );

        let file_path = path.join("src").join("Product.wxs");
        self.write_file(&file_path, &content)?;
        Ok(vec!["src/Product.wxs".to_string()])
    }

    fn create_bootstrapper(&self, path: &Path) -> Result<Vec<String>, InitError> {
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs"
     xmlns:bal="http://wixtoolset.org/schemas/v4/wxs/bal">

    <Bundle Name="{name}"
            Version="{version}"
            Manufacturer="{manufacturer}"
            UpgradeCode="PUT-GUID-HERE">

        <BootstrapperApplication>
            <bal:WixStandardBootstrapperApplication
                LicenseUrl=""
                Theme="hyperlinkLicense" />
        </BootstrapperApplication>

        <!-- Prerequisites chain -->
        <Chain>
            <!-- .NET Runtime prerequisite example -->
            <!--
            <ExePackage Id="NetRuntime"
                        SourceFile="path\to\dotnet-runtime.exe"
                        DetectCondition="NETRUNTIME_INSTALLED"
                        InstallCondition="NOT NETRUNTIME_INSTALLED"
                        InstallCommand="/quiet /norestart" />
            -->

            <!-- Main MSI package -->
            <MsiPackage Id="MainPackage"
                        SourceFile="$(var.MainMsi)"
                        Vital="yes" />
        </Chain>
    </Bundle>
</Wix>
"#,
            name = self.name,
            manufacturer = self.manufacturer,
            version = self.version,
        );

        let file_path = path.join("src").join("Bundle.wxs");
        self.write_file(&file_path, &content)?;
        Ok(vec!["src/Bundle.wxs".to_string()])
    }

    fn create_custom_action(&self, path: &Path) -> Result<Vec<String>, InitError> {
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="{name}"
             Manufacturer="{manufacturer}"
             Version="{version}"
             UpgradeCode="PUT-GUID-HERE">

        <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
        <MediaTemplate EmbedCab="yes" />

        <!-- Custom action DLL -->
        <Binary Id="CustomActionDll" SourceFile="$(var.SourceDir)\CustomActions.dll" />

        <!-- Immediate custom action (runs during UI) -->
        <CustomAction Id="CheckPrerequisites"
                      DllEntry="CheckPrerequisites"
                      BinaryRef="CustomActionDll"
                      Execute="immediate"
                      Return="check" />

        <!-- Deferred custom action (runs during install) -->
        <CustomAction Id="ConfigureApp"
                      DllEntry="ConfigureApp"
                      BinaryRef="CustomActionDll"
                      Execute="deferred"
                      Impersonate="no"
                      Return="check" />

        <!-- Rollback custom action -->
        <CustomAction Id="ConfigureAppRollback"
                      DllEntry="ConfigureAppRollback"
                      BinaryRef="CustomActionDll"
                      Execute="rollback"
                      Impersonate="no"
                      Return="ignore" />

        <InstallExecuteSequence>
            <Custom Action="CheckPrerequisites" Before="LaunchConditions" />
            <Custom Action="ConfigureAppRollback" Before="ConfigureApp" Condition="NOT REMOVE" />
            <Custom Action="ConfigureApp" Before="InstallFinalize" Condition="NOT REMOVE" />
        </InstallExecuteSequence>

        <Feature Id="Main" Title="Main Feature" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
        </Feature>

        <StandardDirectory Id="ProgramFilesFolder">
            <Directory Id="INSTALLFOLDER" Name="{name}">
                <Component Id="MainComponent" Guid="*">
                    <File Source="$(var.SourceDir)\App.exe" />
                </Component>
            </Directory>
        </StandardDirectory>

        <ComponentGroup Id="ProductComponents">
            <ComponentRef Id="MainComponent" />
        </ComponentGroup>
    </Package>
</Wix>
"#,
            name = self.name,
            manufacturer = self.manufacturer,
            version = self.version,
        );

        let file_path = path.join("src").join("Product.wxs");
        self.write_file(&file_path, &content)?;
        Ok(vec!["src/Product.wxs".to_string()])
    }

    fn create_service(&self, path: &Path) -> Result<Vec<String>, InitError> {
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="{name}"
             Manufacturer="{manufacturer}"
             Version="{version}"
             UpgradeCode="PUT-GUID-HERE">

        <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
        <MediaTemplate EmbedCab="yes" />

        <Feature Id="Main" Title="Main Feature" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
        </Feature>

        <StandardDirectory Id="ProgramFilesFolder">
            <Directory Id="INSTALLFOLDER" Name="{name}">
                <Component Id="ServiceComponent" Guid="*">
                    <File Id="ServiceExe" Source="$(var.SourceDir)\MyService.exe" KeyPath="yes" />

                    <ServiceInstall Id="ServiceInstall"
                                    Name="{name}Service"
                                    DisplayName="{name} Service"
                                    Description="{description}"
                                    Type="ownProcess"
                                    Start="auto"
                                    ErrorControl="normal"
                                    Account="LocalSystem" />

                    <ServiceControl Id="ServiceControl"
                                    Name="{name}Service"
                                    Start="install"
                                    Stop="both"
                                    Remove="uninstall"
                                    Wait="yes" />
                </Component>
            </Directory>
        </StandardDirectory>

        <ComponentGroup Id="ProductComponents">
            <ComponentRef Id="ServiceComponent" />
        </ComponentGroup>
    </Package>
</Wix>
"#,
            name = self.name,
            manufacturer = self.manufacturer,
            version = self.version,
            description = self.description,
        );

        let file_path = path.join("src").join("Product.wxs");
        self.write_file(&file_path, &content)?;
        Ok(vec!["src/Product.wxs".to_string()])
    }

    fn create_custom_ui(&self, path: &Path) -> Result<Vec<String>, InitError> {
        let product_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs"
     xmlns:ui="http://wixtoolset.org/schemas/v4/wxs/ui">
    <Package Name="{name}"
             Manufacturer="{manufacturer}"
             Version="{version}"
             UpgradeCode="PUT-GUID-HERE">

        <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
        <MediaTemplate EmbedCab="yes" />

        <!-- Reference the WixUI extension -->
        <ui:WixUI Id="WixUI_InstallDir" InstallDirectory="INSTALLFOLDER" />

        <!-- Custom dialog sequence -->
        <UIRef Id="CustomDialogs" />

        <Feature Id="Main" Title="Main Feature" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
        </Feature>

        <StandardDirectory Id="ProgramFilesFolder">
            <Directory Id="INSTALLFOLDER" Name="{name}">
                <Component Id="MainComponent" Guid="*">
                    <File Source="$(var.SourceDir)\App.exe" />
                </Component>
            </Directory>
        </StandardDirectory>

        <ComponentGroup Id="ProductComponents">
            <ComponentRef Id="MainComponent" />
        </ComponentGroup>
    </Package>
</Wix>
"#,
            name = self.name,
            manufacturer = self.manufacturer,
            version = self.version,
        );

        let dialogs_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Fragment>
        <UI Id="CustomDialogs">
            <!-- Custom welcome dialog -->
            <Dialog Id="CustomWelcomeDlg" Width="370" Height="270" Title="Welcome">
                <Control Id="Title" Type="Text" X="20" Y="20" Width="330" Height="20"
                         Text="Welcome to the installation wizard" />

                <Control Id="Description" Type="Text" X="20" Y="50" Width="330" Height="40"
                         Text="This wizard will guide you through the installation." />

                <Control Id="Next" Type="PushButton" X="236" Y="243" Width="56" Height="17"
                         Text="Next" Default="yes">
                    <Publish Event="NewDialog" Value="InstallDirDlg">1</Publish>
                </Control>

                <Control Id="Cancel" Type="PushButton" X="304" Y="243" Width="56" Height="17"
                         Text="Cancel" Cancel="yes">
                    <Publish Event="SpawnDialog" Value="CancelDlg">1</Publish>
                </Control>
            </Dialog>
        </UI>
    </Fragment>
</Wix>
"#;

        let product_path = path.join("src").join("Product.wxs");
        let dialogs_path = path.join("src").join("Dialogs.wxs");

        self.write_file(&product_path, &product_content)?;
        self.write_file(&dialogs_path, dialogs_content)?;

        Ok(vec![
            "src/Product.wxs".to_string(),
            "src/Dialogs.wxs".to_string(),
        ])
    }

    fn create_merge_module(&self, path: &Path) -> Result<Vec<String>, InitError> {
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Module Id="{name}Module"
            Language="1033"
            Version="{version}">

        <SummaryInformation Description="{description}"
                            Manufacturer="{manufacturer}" />

        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="MergeRedirectFolder">
                <Component Id="SharedComponent" Guid="*">
                    <File Source="$(var.SourceDir)\SharedLib.dll" />
                </Component>
            </Directory>
        </Directory>
    </Module>
</Wix>
"#,
            name = self.name,
            manufacturer = self.manufacturer,
            version = self.version,
            description = self.description,
        );

        let file_path = path.join("src").join("Module.wxs");
        self.write_file(&file_path, &content)?;
        Ok(vec!["src/Module.wxs".to_string()])
    }

    fn create_minimal(&self, path: &Path) -> Result<Vec<String>, InitError> {
        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="{name}"
             Manufacturer="{manufacturer}"
             Version="{version}"
             UpgradeCode="PUT-GUID-HERE">

        <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />

        <Feature Id="Main" Level="1">
            <!-- Add components here -->
        </Feature>
    </Package>
</Wix>
"#,
            name = self.name,
            manufacturer = self.manufacturer,
            version = self.version,
        );

        let file_path = path.join("src").join("Product.wxs");
        self.write_file(&file_path, &content)?;
        Ok(vec!["src/Product.wxs".to_string()])
    }

    fn create_wixproj(&self, path: &Path) -> Result<String, InitError> {
        let content = format!(
            r#"<Project Sdk="WixToolset.Sdk/{wix_version}">
    <PropertyGroup>
        <OutputType>Package</OutputType>
        <SourceDir>.</SourceDir>
    </PropertyGroup>

    <ItemGroup>
        <Compile Include="src\*.wxs" />
    </ItemGroup>
</Project>
"#,
            wix_version = match self.wix_version {
                WixVersion::V4 => "4.0.0",
                WixVersion::V5 => "5.0.0",
            }
        );

        let file_name = format!("{}.wixproj", self.name);
        let file_path = path.join(&file_name);
        self.write_file(&file_path, &content)?;
        Ok(file_name)
    }

    fn create_gitignore(&self, path: &Path) -> Result<String, InitError> {
        let content = r#"# Build outputs
bin/
obj/

# WiX outputs
*.msi
*.msm
*.wixobj
*.wixpdb

# IDE files
.vs/
*.user

# OS files
.DS_Store
Thumbs.db
"#;

        let file_path = path.join(".gitignore");
        self.write_file(&file_path, content)?;
        Ok(".gitignore".to_string())
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), InitError> {
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

/// Result of project creation
#[derive(Debug, Clone)]
pub struct CreatedProject {
    /// Path to created project
    pub path: std::path::PathBuf,
    /// List of created files (relative to project path)
    pub files: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_template_display() {
        assert_eq!(Template::SimpleMsi.to_string(), "simple-msi");
        assert_eq!(Template::Bootstrapper.to_string(), "bootstrapper");
    }

    #[test]
    fn test_template_from_str() {
        assert_eq!("simple-msi".parse::<Template>().unwrap(), Template::SimpleMsi);
        assert_eq!("simple".parse::<Template>().unwrap(), Template::SimpleMsi);
        assert_eq!("bundle".parse::<Template>().unwrap(), Template::Bootstrapper);
        assert!("invalid".parse::<Template>().is_err());
    }

    #[test]
    fn test_template_all() {
        let all = Template::all();
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn test_project_new() {
        let project = Project::new("TestApp", Template::SimpleMsi);
        assert_eq!(project.name, "TestApp");
        assert_eq!(project.template, Template::SimpleMsi);
        assert_eq!(project.manufacturer, "My Company");
    }

    #[test]
    fn test_project_builder() {
        let project = Project::new("TestApp", Template::SimpleMsi)
            .with_manufacturer("Acme Inc")
            .with_version("2.0.0")
            .with_description("Test description")
            .with_wix_version(WixVersion::V4);

        assert_eq!(project.manufacturer, "Acme Inc");
        assert_eq!(project.version, "2.0.0");
        assert_eq!(project.description, "Test description");
        assert_eq!(project.wix_version, WixVersion::V4);
    }

    #[test]
    fn test_validate_empty_name() {
        let project = Project::new("", Template::SimpleMsi);
        assert!(project.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_chars() {
        let project = Project::new("Test App!", Template::SimpleMsi);
        assert!(project.validate().is_err());
    }

    #[test]
    fn test_validate_valid_name() {
        let project = Project::new("Test_App-123", Template::SimpleMsi);
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_create_simple_msi() {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new("TestApp", Template::SimpleMsi);

        let result = project.create(temp_dir.path()).unwrap();

        assert!(result.path.exists());
        assert!(result.path.join("src").join("Product.wxs").exists());
        assert!(result.path.join("TestApp.wixproj").exists());
        assert!(result.path.join(".gitignore").exists());
    }

    #[test]
    fn test_create_feature_msi() {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new("FeatureApp", Template::FeatureMsi);

        let result = project.create(temp_dir.path()).unwrap();

        assert!(result.path.exists());
        let content = fs::read_to_string(result.path.join("src").join("Product.wxs")).unwrap();
        assert!(content.contains("DocsFeature"));
        assert!(content.contains("ExamplesFeature"));
    }

    #[test]
    fn test_create_bootstrapper() {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new("BundleApp", Template::Bootstrapper);

        let result = project.create(temp_dir.path()).unwrap();

        assert!(result.path.join("src").join("Bundle.wxs").exists());
        let content = fs::read_to_string(result.path.join("src").join("Bundle.wxs")).unwrap();
        assert!(content.contains("<Bundle"));
        assert!(content.contains("<Chain"));
    }

    #[test]
    fn test_create_service() {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new("ServiceApp", Template::Service);

        let result = project.create(temp_dir.path()).unwrap();

        let content = fs::read_to_string(result.path.join("src").join("Product.wxs")).unwrap();
        assert!(content.contains("<ServiceInstall"));
        assert!(content.contains("<ServiceControl"));
    }

    #[test]
    fn test_create_custom_ui() {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new("UiApp", Template::CustomUi);

        let result = project.create(temp_dir.path()).unwrap();

        assert!(result.path.join("src").join("Product.wxs").exists());
        assert!(result.path.join("src").join("Dialogs.wxs").exists());
    }

    #[test]
    fn test_create_merge_module() {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new("SharedLib", Template::MergeModule);

        let result = project.create(temp_dir.path()).unwrap();

        assert!(result.path.join("src").join("Module.wxs").exists());
        let content = fs::read_to_string(result.path.join("src").join("Module.wxs")).unwrap();
        assert!(content.contains("<Module"));
    }

    #[test]
    fn test_create_minimal() {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new("MinApp", Template::Minimal);

        let result = project.create(temp_dir.path()).unwrap();

        let content = fs::read_to_string(result.path.join("src").join("Product.wxs")).unwrap();
        // Minimal should be short
        assert!(content.lines().count() < 20);
    }

    #[test]
    fn test_create_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new("TestApp", Template::SimpleMsi);

        // Create first time
        project.create(temp_dir.path()).unwrap();

        // Try to create again - should fail
        let result = project.create(temp_dir.path());
        assert!(matches!(result, Err(InitError::DirectoryExists(_))));
    }

    #[test]
    fn test_wix_version_display() {
        assert_eq!(WixVersion::V4.to_string(), "v4");
        assert_eq!(WixVersion::V5.to_string(), "v5");
    }

    #[test]
    fn test_wixproj_version() {
        let temp_dir = TempDir::new().unwrap();

        let project_v4 = Project::new("V4App", Template::SimpleMsi)
            .with_wix_version(WixVersion::V4);
        let result = project_v4.create(temp_dir.path()).unwrap();
        let content = fs::read_to_string(result.path.join("V4App.wixproj")).unwrap();
        assert!(content.contains("4.0.0"));

        let project_v5 = Project::new("V5App", Template::SimpleMsi)
            .with_wix_version(WixVersion::V5);
        let result = project_v5.create(temp_dir.path()).unwrap();
        let content = fs::read_to_string(result.path.join("V5App.wixproj")).unwrap();
        assert!(content.contains("5.0.0"));
    }

    #[test]
    fn test_template_descriptions() {
        for template in Template::all() {
            assert!(!template.description().is_empty());
        }
    }
}
