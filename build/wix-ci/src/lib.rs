//! CI/CD pipeline templates for WiX projects
//!
//! Generates workflow configurations for GitHub Actions, Azure Pipelines, and GitLab CI.
//!
//! # Example
//!
//! ```
//! use wix_ci::{CiGenerator, CiPlatform, CiOptions};
//!
//! let options = CiOptions::default();
//! let generator = CiGenerator::new(options);
//! let workflow = generator.generate(CiPlatform::GitHubActions);
//! assert!(workflow.contains("wix"));
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// CI/CD generation errors
#[derive(Error, Debug)]
pub enum CiError {
    #[error("Failed to write file: {0}")]
    WriteError(String),
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
}

/// CI/CD platform to generate for
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CiPlatform {
    GitHubActions,
    AzurePipelines,
    GitLabCi,
}

impl CiPlatform {
    pub fn all() -> Vec<CiPlatform> {
        vec![
            CiPlatform::GitHubActions,
            CiPlatform::AzurePipelines,
            CiPlatform::GitLabCi,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CiPlatform::GitHubActions => "GitHub Actions",
            CiPlatform::AzurePipelines => "Azure Pipelines",
            CiPlatform::GitLabCi => "GitLab CI",
        }
    }

    pub fn file_path(&self) -> &'static str {
        match self {
            CiPlatform::GitHubActions => ".github/workflows/build.yml",
            CiPlatform::AzurePipelines => "azure-pipelines.yml",
            CiPlatform::GitLabCi => ".gitlab-ci.yml",
        }
    }
}

/// Options for CI/CD generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiOptions {
    /// Project name
    pub project_name: String,
    /// WiX version to use
    pub wix_version: String,
    /// .NET version for WiX
    pub dotnet_version: String,
    /// Include code signing step
    pub code_signing: bool,
    /// Include unit tests
    pub run_tests: bool,
    /// Include artifact upload
    pub upload_artifacts: bool,
    /// Branch to build on
    pub build_branch: String,
    /// Include release publishing
    pub publish_release: bool,
    /// Windows runner image
    pub windows_image: String,
}

impl Default for CiOptions {
    fn default() -> Self {
        Self {
            project_name: "MyInstaller".to_string(),
            wix_version: "5.0.2".to_string(),
            dotnet_version: "8.0".to_string(),
            code_signing: false,
            run_tests: true,
            upload_artifacts: true,
            build_branch: "main".to_string(),
            publish_release: false,
            windows_image: "windows-latest".to_string(),
        }
    }
}

/// CI/CD workflow generator
pub struct CiGenerator {
    options: CiOptions,
}

impl Default for CiGenerator {
    fn default() -> Self {
        Self::new(CiOptions::default())
    }
}

impl CiGenerator {
    pub fn new(options: CiOptions) -> Self {
        Self { options }
    }

    /// Generate workflow for a platform
    pub fn generate(&self, platform: CiPlatform) -> String {
        match platform {
            CiPlatform::GitHubActions => self.github_actions(),
            CiPlatform::AzurePipelines => self.azure_pipelines(),
            CiPlatform::GitLabCi => self.gitlab_ci(),
        }
    }

    /// Write workflow to file
    pub fn write(&self, platform: CiPlatform, base_path: &Path) -> Result<String, CiError> {
        let content = self.generate(platform);
        let file_path = base_path.join(platform.file_path());

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CiError::WriteError(e.to_string()))?;
        }

        std::fs::write(&file_path, &content)
            .map_err(|e| CiError::WriteError(e.to_string()))?;

        Ok(file_path.display().to_string())
    }

    fn github_actions(&self) -> String {
        let mut workflow = String::new();

        workflow.push_str(&format!(
            r#"name: Build {}

on:
  push:
    branches: [ {} ]
  pull_request:
    branches: [ {} ]
  workflow_dispatch:

env:
  WIX_VERSION: '{}'
  DOTNET_VERSION: '{}'

jobs:
  build:
    runs-on: {}

    steps:
    - name: Checkout
      uses: actions/checkout@v4

    - name: Setup .NET
      uses: actions/setup-dotnet@v4
      with:
        dotnet-version: ${{{{ env.DOTNET_VERSION }}}}

    - name: Install WiX Toolset
      run: |
        dotnet tool install --global wix --version ${{{{ env.WIX_VERSION }}}}
        wix --version
"#,
            self.options.project_name,
            self.options.build_branch,
            self.options.build_branch,
            self.options.wix_version,
            self.options.dotnet_version,
            self.options.windows_image,
        ));

        if self.options.run_tests {
            workflow.push_str(
                r#"
    - name: Run Tests
      run: dotnet test --verbosity normal
"#,
            );
        }

        workflow.push_str(
            r#"
    - name: Build MSI
      run: |
        wix build -o output/${{ github.event.repository.name }}.msi *.wxs
"#,
        );

        if self.options.code_signing {
            workflow.push_str(
                r#"
    - name: Sign MSI
      env:
        CERTIFICATE_BASE64: ${{ secrets.CERTIFICATE_BASE64 }}
        CERTIFICATE_PASSWORD: ${{ secrets.CERTIFICATE_PASSWORD }}
      run: |
        $cert = [System.Convert]::FromBase64String($env:CERTIFICATE_BASE64)
        [System.IO.File]::WriteAllBytes("certificate.pfx", $cert)
        & signtool sign /f certificate.pfx /p $env:CERTIFICATE_PASSWORD /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 output/*.msi
        Remove-Item certificate.pfx
"#,
            );
        }

        if self.options.upload_artifacts {
            workflow.push_str(
                r#"
    - name: Upload MSI Artifact
      uses: actions/upload-artifact@v4
      with:
        name: installer
        path: output/*.msi
        retention-days: 30
"#,
            );
        }

        if self.options.publish_release {
            workflow.push_str(
                r#"
  release:
    needs: build
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/')

    steps:
    - name: Download Artifact
      uses: actions/download-artifact@v4
      with:
        name: installer

    - name: Create Release
      uses: softprops/action-gh-release@v2
      with:
        files: "*.msi"
        generate_release_notes: true
"#,
            );
        }

        workflow
    }

    fn azure_pipelines(&self) -> String {
        let mut pipeline = String::new();

        pipeline.push_str(&format!(
            r#"trigger:
  branches:
    include:
    - {}

pool:
  vmImage: '{}'

variables:
  WIX_VERSION: '{}'
  DOTNET_VERSION: '{}'

stages:
- stage: Build
  displayName: 'Build MSI'
  jobs:
  - job: BuildJob
    displayName: 'Build {} Installer'
    steps:
    - task: UseDotNet@2
      displayName: 'Setup .NET'
      inputs:
        packageType: 'sdk'
        version: '$(DOTNET_VERSION)'

    - script: |
        dotnet tool install --global wix --version $(WIX_VERSION)
        wix --version
      displayName: 'Install WiX Toolset'
"#,
            self.options.build_branch,
            self.options.windows_image,
            self.options.wix_version,
            self.options.dotnet_version,
            self.options.project_name,
        ));

        if self.options.run_tests {
            pipeline.push_str(
                r#"
    - task: DotNetCoreCLI@2
      displayName: 'Run Tests'
      inputs:
        command: 'test'
        projects: '**/*.csproj'
"#,
            );
        }

        pipeline.push_str(
            r#"
    - script: |
        wix build -o $(Build.ArtifactStagingDirectory)/installer.msi *.wxs
      displayName: 'Build MSI'
"#,
        );

        if self.options.code_signing {
            pipeline.push_str(
                r#"
    - task: DownloadSecureFile@1
      name: certificate
      displayName: 'Download Code Signing Certificate'
      inputs:
        secureFile: 'certificate.pfx'

    - script: |
        signtool sign /f $(certificate.secureFilePath) /p $(CertificatePassword) /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $(Build.ArtifactStagingDirectory)/*.msi
      displayName: 'Sign MSI'
"#,
            );
        }

        if self.options.upload_artifacts {
            pipeline.push_str(
                r#"
    - task: PublishBuildArtifacts@1
      displayName: 'Publish MSI Artifact'
      inputs:
        PathtoPublish: '$(Build.ArtifactStagingDirectory)'
        ArtifactName: 'installer'
        publishLocation: 'Container'
"#,
            );
        }

        pipeline
    }

    fn gitlab_ci(&self) -> String {
        let mut ci = String::new();

        ci.push_str(&format!(
            r#"stages:
  - build
{}{}
variables:
  WIX_VERSION: "{}"
  DOTNET_VERSION: "{}"

build:
  stage: build
  tags:
    - windows
  before_script:
    - choco install dotnet-sdk --version=$DOTNET_VERSION -y
    - dotnet tool install --global wix --version $WIX_VERSION
    - $env:PATH += ";$env:USERPROFILE\.dotnet\tools"
"#,
            if self.options.run_tests { "  - test\n" } else { "" },
            if self.options.publish_release { "  - release\n" } else { "" },
            self.options.wix_version,
            self.options.dotnet_version,
        ));

        if self.options.run_tests {
            ci.push_str(
                r#"  script:
    - dotnet test --verbosity normal
    - wix build -o output/installer.msi *.wxs
"#,
            );
        } else {
            ci.push_str(
                r#"  script:
    - wix build -o output/installer.msi *.wxs
"#,
            );
        }

        if self.options.code_signing {
            ci.push_str(
                r#"    - |
      $cert = [System.Convert]::FromBase64String($CERTIFICATE_BASE64)
      [System.IO.File]::WriteAllBytes("certificate.pfx", $cert)
      signtool sign /f certificate.pfx /p $CERTIFICATE_PASSWORD /fd SHA256 output/*.msi
      Remove-Item certificate.pfx
"#,
            );
        }

        if self.options.upload_artifacts {
            ci.push_str(
                r#"  artifacts:
    paths:
      - output/*.msi
    expire_in: 30 days
"#,
            );
        }

        ci.push_str(&format!(
            r#"  only:
    - {}
"#,
            self.options.build_branch
        ));

        if self.options.publish_release {
            ci.push_str(
                r#"
release:
  stage: release
  script:
    - echo "Creating release..."
  only:
    - tags
  when: manual
"#,
            );
        }

        ci
    }
}

/// Summary of generated files
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationSummary {
    pub files_created: Vec<String>,
    pub platform: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = CiOptions::default();
        assert_eq!(options.project_name, "MyInstaller");
        assert!(options.run_tests);
        assert!(options.upload_artifacts);
    }

    #[test]
    fn test_github_actions_basic() {
        let generator = CiGenerator::default();
        let workflow = generator.generate(CiPlatform::GitHubActions);

        assert!(workflow.contains("name: Build"));
        assert!(workflow.contains("wix build"));
        assert!(workflow.contains("actions/checkout@v4"));
        assert!(workflow.contains("dotnet tool install"));
    }

    #[test]
    fn test_github_actions_with_signing() {
        let options = CiOptions {
            code_signing: true,
            ..Default::default()
        };
        let generator = CiGenerator::new(options);
        let workflow = generator.generate(CiPlatform::GitHubActions);

        assert!(workflow.contains("signtool"));
        assert!(workflow.contains("CERTIFICATE"));
    }

    #[test]
    fn test_github_actions_without_tests() {
        let options = CiOptions {
            run_tests: false,
            ..Default::default()
        };
        let generator = CiGenerator::new(options);
        let workflow = generator.generate(CiPlatform::GitHubActions);

        assert!(!workflow.contains("dotnet test"));
    }

    #[test]
    fn test_github_actions_with_release() {
        let options = CiOptions {
            publish_release: true,
            ..Default::default()
        };
        let generator = CiGenerator::new(options);
        let workflow = generator.generate(CiPlatform::GitHubActions);

        assert!(workflow.contains("release:"));
        assert!(workflow.contains("softprops/action-gh-release"));
    }

    #[test]
    fn test_azure_pipelines_basic() {
        let generator = CiGenerator::default();
        let pipeline = generator.generate(CiPlatform::AzurePipelines);

        assert!(pipeline.contains("trigger:"));
        assert!(pipeline.contains("pool:"));
        assert!(pipeline.contains("wix build"));
        assert!(pipeline.contains("UseDotNet@2"));
    }

    #[test]
    fn test_azure_pipelines_with_signing() {
        let options = CiOptions {
            code_signing: true,
            ..Default::default()
        };
        let generator = CiGenerator::new(options);
        let pipeline = generator.generate(CiPlatform::AzurePipelines);

        assert!(pipeline.contains("DownloadSecureFile"));
        assert!(pipeline.contains("signtool"));
    }

    #[test]
    fn test_gitlab_ci_basic() {
        let generator = CiGenerator::default();
        let ci = generator.generate(CiPlatform::GitLabCi);

        assert!(ci.contains("stages:"));
        assert!(ci.contains("build:"));
        assert!(ci.contains("wix build"));
    }

    #[test]
    fn test_gitlab_ci_with_artifacts() {
        let options = CiOptions {
            upload_artifacts: true,
            ..Default::default()
        };
        let generator = CiGenerator::new(options);
        let ci = generator.generate(CiPlatform::GitLabCi);

        assert!(ci.contains("artifacts:"));
        assert!(ci.contains("expire_in:"));
    }

    #[test]
    fn test_platform_file_paths() {
        assert_eq!(
            CiPlatform::GitHubActions.file_path(),
            ".github/workflows/build.yml"
        );
        assert_eq!(
            CiPlatform::AzurePipelines.file_path(),
            "azure-pipelines.yml"
        );
        assert_eq!(CiPlatform::GitLabCi.file_path(), ".gitlab-ci.yml");
    }

    #[test]
    fn test_platform_names() {
        assert_eq!(CiPlatform::GitHubActions.as_str(), "GitHub Actions");
        assert_eq!(CiPlatform::AzurePipelines.as_str(), "Azure Pipelines");
        assert_eq!(CiPlatform::GitLabCi.as_str(), "GitLab CI");
    }

    #[test]
    fn test_all_platforms() {
        let platforms = CiPlatform::all();
        assert_eq!(platforms.len(), 3);
    }

    #[test]
    fn test_custom_project_name() {
        let options = CiOptions {
            project_name: "CustomApp".to_string(),
            ..Default::default()
        };
        let generator = CiGenerator::new(options);

        let github = generator.generate(CiPlatform::GitHubActions);
        assert!(github.contains("Build CustomApp"));

        let azure = generator.generate(CiPlatform::AzurePipelines);
        assert!(azure.contains("CustomApp"));
    }

    #[test]
    fn test_custom_versions() {
        let options = CiOptions {
            wix_version: "4.0.0".to_string(),
            dotnet_version: "6.0".to_string(),
            ..Default::default()
        };
        let generator = CiGenerator::new(options);

        let workflow = generator.generate(CiPlatform::GitHubActions);
        assert!(workflow.contains("WIX_VERSION: '4.0.0'"));
        assert!(workflow.contains("DOTNET_VERSION: '6.0'"));
    }

    #[test]
    fn test_custom_branch() {
        let options = CiOptions {
            build_branch: "develop".to_string(),
            ..Default::default()
        };
        let generator = CiGenerator::new(options);

        let workflow = generator.generate(CiPlatform::GitHubActions);
        assert!(workflow.contains("branches: [ develop ]"));
    }

    #[test]
    fn test_write_workflow() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let generator = CiGenerator::default();

        let path = generator.write(CiPlatform::GitHubActions, dir.path()).unwrap();
        assert!(path.contains(".github/workflows/build.yml"));
        assert!(dir.path().join(".github/workflows/build.yml").exists());
    }

    #[test]
    fn test_custom_windows_image() {
        let options = CiOptions {
            windows_image: "windows-2022".to_string(),
            ..Default::default()
        };
        let generator = CiGenerator::new(options);

        let workflow = generator.generate(CiPlatform::GitHubActions);
        assert!(workflow.contains("runs-on: windows-2022"));

        let azure = generator.generate(CiPlatform::AzurePipelines);
        assert!(azure.contains("vmImage: 'windows-2022'"));
    }

    #[test]
    fn test_generation_summary() {
        let summary = GenerationSummary {
            files_created: vec!["file1.yml".to_string()],
            platform: "GitHub Actions".to_string(),
        };
        assert_eq!(summary.files_created.len(), 1);
    }

    #[test]
    fn test_all_options_enabled() {
        let options = CiOptions {
            project_name: "FullApp".to_string(),
            code_signing: true,
            run_tests: true,
            upload_artifacts: true,
            publish_release: true,
            ..Default::default()
        };
        let generator = CiGenerator::new(options);

        for platform in CiPlatform::all() {
            let output = generator.generate(platform);
            assert!(!output.is_empty());
            assert!(output.contains("wix"));
        }
    }

    #[test]
    fn test_minimal_options() {
        let options = CiOptions {
            code_signing: false,
            run_tests: false,
            upload_artifacts: false,
            publish_release: false,
            ..Default::default()
        };
        let generator = CiGenerator::new(options);

        let workflow = generator.generate(CiPlatform::GitHubActions);
        assert!(!workflow.contains("signtool"));
        assert!(!workflow.contains("dotnet test"));
        assert!(!workflow.contains("upload-artifact"));
        assert!(!workflow.contains("release:"));
    }
}
