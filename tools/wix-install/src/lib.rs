//! wix-install - WiX development environment preparation tool
//!
//! Checks, installs, and configures prerequisites for WiX MSI development.
//! Supports offline installation packages and Windows Sandbox testing.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Prerequisite component for WiX development
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prerequisite {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required: bool,
    pub check_command: Option<String>,
    pub version_command: Option<String>,
    pub download_url: Option<String>,
    pub offline_filename: Option<String>,
    pub install_command: Option<String>,
    pub install_args: Option<Vec<String>>,
}

/// Installation status of a prerequisite
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrereqStatus {
    Installed,
    NotInstalled,
    Outdated,
    Unknown,
}

/// Check result for a prerequisite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub prerequisite: String,
    pub status: PrereqStatus,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub message: String,
}

impl CheckResult {
    pub fn installed(prereq: &str, version: &str, path: Option<PathBuf>) -> Self {
        Self {
            prerequisite: prereq.to_string(),
            status: PrereqStatus::Installed,
            version: Some(version.to_string()),
            path,
            message: format!("{} {} is installed", prereq, version),
        }
    }

    pub fn not_installed(prereq: &str) -> Self {
        Self {
            prerequisite: prereq.to_string(),
            status: PrereqStatus::NotInstalled,
            version: None,
            path: None,
            message: format!("{} is not installed", prereq),
        }
    }

    pub fn outdated(prereq: &str, current: &str, required: &str) -> Self {
        Self {
            prerequisite: prereq.to_string(),
            status: PrereqStatus::Outdated,
            version: Some(current.to_string()),
            path: None,
            message: format!("{} {} is outdated (requires {})", prereq, current, required),
        }
    }

    pub fn unknown(prereq: &str, error: &str) -> Self {
        Self {
            prerequisite: prereq.to_string(),
            status: PrereqStatus::Unknown,
            version: None,
            path: None,
            message: format!("{}: {}", prereq, error),
        }
    }
}

/// Environment check summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentCheck {
    pub results: Vec<CheckResult>,
    pub ready: bool,
    pub missing_required: Vec<String>,
    pub missing_optional: Vec<String>,
}

impl EnvironmentCheck {
    pub fn new(results: Vec<CheckResult>, prerequisites: &[Prerequisite]) -> Self {
        let mut missing_required = Vec::new();
        let mut missing_optional = Vec::new();

        for result in &results {
            if result.status != PrereqStatus::Installed {
                if let Some(prereq) = prerequisites.iter().find(|p| p.id == result.prerequisite) {
                    if prereq.required {
                        missing_required.push(result.prerequisite.clone());
                    } else {
                        missing_optional.push(result.prerequisite.clone());
                    }
                }
            }
        }

        let ready = missing_required.is_empty();

        Self {
            results,
            ready,
            missing_required,
            missing_optional,
        }
    }
}

/// Windows Sandbox configuration for MSI testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub name: String,
    pub memory_mb: Option<u32>,
    pub networking: bool,
    pub vgpu: bool,
    pub mapped_folders: Vec<MappedFolder>,
    pub logon_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappedFolder {
    pub host_path: PathBuf,
    pub sandbox_path: Option<String>,
    pub read_only: bool,
}

impl SandboxConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            memory_mb: Some(4096),
            networking: true,
            vgpu: false,
            mapped_folders: Vec::new(),
            logon_command: None,
        }
    }

    pub fn for_msi_testing(name: &str, msi_folder: PathBuf) -> Self {
        Self::new(name)
            .with_memory(4096)
            .with_networking(false)
            .add_folder(msi_folder, Some("C:\\TestMSI".to_string()), true)
    }

    pub fn for_wix_development(name: &str, project_folder: PathBuf) -> Self {
        Self::new(name)
            .with_memory(8192)
            .with_networking(true)
            .add_folder(project_folder, Some("C:\\Project".to_string()), false)
            .with_logon_command("powershell -ExecutionPolicy Bypass -File C:\\Project\\setup.ps1")
    }

    pub fn with_memory(mut self, mb: u32) -> Self {
        self.memory_mb = Some(mb);
        self
    }

    pub fn with_networking(mut self, enabled: bool) -> Self {
        self.networking = enabled;
        self
    }

    pub fn with_vgpu(mut self, enabled: bool) -> Self {
        self.vgpu = enabled;
        self
    }

    pub fn add_folder(mut self, host: PathBuf, sandbox: Option<String>, read_only: bool) -> Self {
        self.mapped_folders.push(MappedFolder {
            host_path: host,
            sandbox_path: sandbox,
            read_only,
        });
        self
    }

    pub fn with_logon_command(mut self, command: &str) -> Self {
        self.logon_command = Some(command.to_string());
        self
    }

    /// Generate .wsb file content
    pub fn to_wsb(&self) -> String {
        let mut xml = String::from("<Configuration>\n");

        if let Some(mem) = self.memory_mb {
            xml.push_str(&format!("  <MemoryInMB>{}</MemoryInMB>\n", mem));
        }

        xml.push_str(&format!(
            "  <Networking>{}</Networking>\n",
            if self.networking { "Enable" } else { "Disable" }
        ));

        xml.push_str(&format!(
            "  <vGPU>{}</vGPU>\n",
            if self.vgpu { "Enable" } else { "Disable" }
        ));

        if !self.mapped_folders.is_empty() {
            xml.push_str("  <MappedFolders>\n");
            for folder in &self.mapped_folders {
                xml.push_str("    <MappedFolder>\n");
                xml.push_str(&format!(
                    "      <HostFolder>{}</HostFolder>\n",
                    folder.host_path.display()
                ));
                if let Some(ref sandbox_path) = folder.sandbox_path {
                    xml.push_str(&format!(
                        "      <SandboxFolder>{}</SandboxFolder>\n",
                        sandbox_path
                    ));
                }
                xml.push_str(&format!(
                    "      <ReadOnly>{}</ReadOnly>\n",
                    if folder.read_only { "true" } else { "false" }
                ));
                xml.push_str("    </MappedFolder>\n");
            }
            xml.push_str("  </MappedFolders>\n");
        }

        if let Some(ref cmd) = self.logon_command {
            xml.push_str("  <LogonCommand>\n");
            xml.push_str(&format!("    <Command>{}</Command>\n", cmd));
            xml.push_str("  </LogonCommand>\n");
        }

        xml.push_str("</Configuration>\n");
        xml
    }
}

/// Offline package configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflinePackage {
    pub name: String,
    pub output_dir: PathBuf,
    pub components: Vec<OfflineComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineComponent {
    pub id: String,
    pub name: String,
    pub download_url: String,
    pub filename: String,
    pub size_mb: Option<u32>,
    pub sha256: Option<String>,
}

impl OfflinePackage {
    pub fn new(name: &str, output_dir: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            output_dir,
            components: Vec::new(),
        }
    }

    pub fn minimal(output_dir: PathBuf) -> Self {
        let mut pkg = Self::new("wix-minimal", output_dir);
        pkg.components = vec![
            OfflineComponent {
                id: "dotnet8".to_string(),
                name: ".NET 8.0 SDK".to_string(),
                download_url: "https://download.visualstudio.microsoft.com/download/pr/dotnet-sdk-8.0.400-win-x64.exe".to_string(),
                filename: "dotnet-sdk-8.0.400-win-x64.exe".to_string(),
                size_mb: Some(220),
                sha256: None,
            },
        ];
        pkg
    }

    pub fn standard(output_dir: PathBuf) -> Self {
        let mut pkg = Self::minimal(output_dir);
        pkg.name = "wix-standard".to_string();
        pkg.components.push(OfflineComponent {
            id: "git".to_string(),
            name: "Git for Windows".to_string(),
            download_url: "https://github.com/git-for-windows/git/releases/download/v2.43.0.windows.1/Git-2.43.0-64-bit.exe".to_string(),
            filename: "Git-2.43.0-64-bit.exe".to_string(),
            size_mb: Some(60),
            sha256: None,
        });
        pkg
    }

    pub fn full(output_dir: PathBuf) -> Self {
        let mut pkg = Self::standard(output_dir);
        pkg.name = "wix-full".to_string();
        pkg.components.push(OfflineComponent {
            id: "vsbuildtools".to_string(),
            name: "Visual Studio Build Tools 2022".to_string(),
            download_url: "https://aka.ms/vs/17/release/vs_buildtools.exe".to_string(),
            filename: "vs_buildtools.exe".to_string(),
            size_mb: Some(3),
            sha256: None,
        });
        pkg.components.push(OfflineComponent {
            id: "windowssdk".to_string(),
            name: "Windows SDK".to_string(),
            download_url: "https://go.microsoft.com/fwlink/?linkid=2272610".to_string(),
            filename: "winsdksetup.exe".to_string(),
            size_mb: Some(2),
            sha256: None,
        });
        pkg
    }

    pub fn add_component(&mut self, component: OfflineComponent) {
        self.components.push(component);
    }

    pub fn total_size_mb(&self) -> u32 {
        self.components.iter().filter_map(|c| c.size_mb).sum()
    }
}

/// Default prerequisites for WiX development
pub fn default_prerequisites() -> Vec<Prerequisite> {
    vec![
        Prerequisite {
            id: "dotnet".to_string(),
            name: ".NET SDK".to_string(),
            description: ".NET SDK 6.0+ required for WiX v4/v5".to_string(),
            required: true,
            check_command: Some("dotnet --version".to_string()),
            version_command: Some("dotnet --version".to_string()),
            download_url: Some("https://dotnet.microsoft.com/download".to_string()),
            offline_filename: Some("dotnet-sdk-installer.exe".to_string()),
            install_command: None,
            install_args: None,
        },
        Prerequisite {
            id: "wix".to_string(),
            name: "WiX Toolset".to_string(),
            description: "WiX Toolset for building MSI packages".to_string(),
            required: true,
            check_command: Some("wix --version".to_string()),
            version_command: Some("wix --version".to_string()),
            download_url: Some("https://wixtoolset.org/".to_string()),
            offline_filename: None,
            install_command: Some("dotnet".to_string()),
            install_args: Some(vec!["tool".to_string(), "install".to_string(), "--global".to_string(), "wix".to_string()]),
        },
        Prerequisite {
            id: "vsbuildtools".to_string(),
            name: "Visual Studio Build Tools".to_string(),
            description: "C++ build tools and Windows SDK".to_string(),
            required: false,
            check_command: Some("vswhere -latest -property installationPath".to_string()),
            version_command: Some("vswhere -latest -property installationVersion".to_string()),
            download_url: Some("https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022".to_string()),
            offline_filename: Some("vs_buildtools.exe".to_string()),
            install_command: None,
            install_args: None,
        },
        Prerequisite {
            id: "windowssdk".to_string(),
            name: "Windows SDK".to_string(),
            description: "Windows SDK for signtool".to_string(),
            required: false,
            check_command: Some("where signtool".to_string()),
            version_command: None,
            download_url: Some("https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/".to_string()),
            offline_filename: Some("winsdksetup.exe".to_string()),
            install_command: None,
            install_args: None,
        },
        Prerequisite {
            id: "git".to_string(),
            name: "Git".to_string(),
            description: "Git version control".to_string(),
            required: false,
            check_command: Some("git --version".to_string()),
            version_command: Some("git --version".to_string()),
            download_url: Some("https://git-scm.com/download/win".to_string()),
            offline_filename: Some("Git-installer.exe".to_string()),
            install_command: None,
            install_args: None,
        },
    ]
}

/// PowerShell and batch scripts for setup
pub mod scripts {
    /// PowerShell: Check environment
    pub const CHECK_ENV_PS1: &str = r#"# WiX Development Environment Check
# Run: powershell -ExecutionPolicy Bypass -File check-env.ps1

Write-Host ""
Write-Host "WiX Development Environment Check" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""

$allGood = $true
$missing = @()

# Check .NET SDK
Write-Host "Checking .NET SDK... " -NoNewline
try {
    $version = & dotnet --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "OK ($version)" -ForegroundColor Green
    } else {
        Write-Host "NOT FOUND" -ForegroundColor Red
        $allGood = $false
        $missing += ".NET SDK"
    }
} catch {
    Write-Host "NOT FOUND" -ForegroundColor Red
    $allGood = $false
    $missing += ".NET SDK"
}

# Check WiX Toolset
Write-Host "Checking WiX Toolset... " -NoNewline
try {
    $version = & wix --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "OK ($version)" -ForegroundColor Green
    } else {
        Write-Host "NOT FOUND" -ForegroundColor Red
        $allGood = $false
        $missing += "WiX Toolset"
    }
} catch {
    Write-Host "NOT FOUND" -ForegroundColor Red
    $allGood = $false
    $missing += "WiX Toolset"
}

# Check VS Build Tools (optional)
Write-Host "Checking VS Build Tools... " -NoNewline
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
    $vsPath = & $vswhere -latest -property installationPath 2>$null
    if ($vsPath) {
        Write-Host "OK" -ForegroundColor Green
    } else {
        Write-Host "NOT FOUND (optional)" -ForegroundColor Yellow
    }
} else {
    Write-Host "NOT FOUND (optional)" -ForegroundColor Yellow
}

# Check signtool (optional)
Write-Host "Checking signtool... " -NoNewline
$signtool = Get-Command signtool -ErrorAction SilentlyContinue
if ($signtool) {
    Write-Host "OK" -ForegroundColor Green
} else {
    Write-Host "NOT FOUND (optional)" -ForegroundColor Yellow
}

# Check Git (optional)
Write-Host "Checking Git... " -NoNewline
try {
    $version = & git --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "OK" -ForegroundColor Green
    } else {
        Write-Host "NOT FOUND (optional)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "NOT FOUND (optional)" -ForegroundColor Yellow
}

Write-Host ""
if ($allGood) {
    Write-Host "Environment is ready for WiX development!" -ForegroundColor Green
} else {
    Write-Host "Missing required components:" -ForegroundColor Red
    foreach ($item in $missing) {
        Write-Host "  - $item" -ForegroundColor Red
    }
    Write-Host ""
    Write-Host "Run 'wix-install setup' to install missing components." -ForegroundColor Yellow
}
Write-Host ""
"#;

    /// PowerShell: Install WiX
    pub const INSTALL_WIX_PS1: &str = r#"# Install WiX Toolset
# Run: powershell -ExecutionPolicy Bypass -File install-wix.ps1

Write-Host ""
Write-Host "Installing WiX Toolset" -ForegroundColor Cyan
Write-Host "======================" -ForegroundColor Cyan
Write-Host ""

# Check .NET SDK
Write-Host "Checking .NET SDK... " -NoNewline
try {
    $version = & dotnet --version 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "NOT FOUND" -ForegroundColor Red
        Write-Host ""
        Write-Host "Error: .NET SDK is required to install WiX." -ForegroundColor Red
        Write-Host "Download from: https://dotnet.microsoft.com/download" -ForegroundColor Yellow
        exit 1
    }
    Write-Host "OK ($version)" -ForegroundColor Green
} catch {
    Write-Host "NOT FOUND" -ForegroundColor Red
    exit 1
}

# Install WiX
Write-Host ""
Write-Host "Installing WiX as .NET global tool..." -ForegroundColor Cyan
& dotnet tool install --global wix

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    $wixVersion = & wix --version 2>$null
    Write-Host "WiX Toolset $wixVersion installed successfully!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "Installation failed. Try updating:" -ForegroundColor Yellow
    Write-Host "  dotnet tool update --global wix" -ForegroundColor White
}
Write-Host ""
"#;

    /// Batch: Offline installer
    pub const OFFLINE_INSTALL_BAT: &str = r#"@echo off
REM WiX Development Environment - Offline Installation
REM Run as Administrator

echo.
echo ========================================
echo WiX Development Environment Setup
echo ========================================
echo.

REM Check admin rights
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo Error: Administrator privileges required.
    echo Right-click and select "Run as administrator"
    pause
    exit /b 1
)

REM Install .NET SDK
if exist "dotnet-sdk-*.exe" (
    echo Installing .NET SDK...
    for %%f in (dotnet-sdk-*.exe) do (
        start /wait %%f /install /quiet /norestart
    )
    echo .NET SDK installed.
) else (
    echo Warning: .NET SDK installer not found
)

REM Install Git (optional)
if exist "Git-*.exe" (
    echo Installing Git...
    for %%f in (Git-*.exe) do (
        start /wait %%f /VERYSILENT /NORESTART
    )
    echo Git installed.
)

REM Refresh PATH
call refreshenv >nul 2>&1

REM Install WiX via dotnet
echo Installing WiX Toolset...
dotnet tool install --global wix

echo.
echo ========================================
echo Setup complete!
echo ========================================
echo.
echo Verify with: wix --version
echo.
pause
"#;

    /// PowerShell: Create sandbox
    pub const CREATE_SANDBOX_PS1: &str = r#"# Create Windows Sandbox for MSI testing
# Usage: .\create-sandbox.ps1 -MsiPath "C:\path\to\installer.msi"

param(
    [Parameter(Mandatory=$true)]
    [string]$MsiPath,

    [string]$SandboxName = "WixTest",

    [int]$MemoryMB = 4096,

    [switch]$NoNetworking
)

$msiFolder = Split-Path -Parent $MsiPath
$msiFile = Split-Path -Leaf $MsiPath

$wsb = @"
<Configuration>
  <MemoryInMB>$MemoryMB</MemoryInMB>
  <Networking>$(if ($NoNetworking) { "Disable" } else { "Enable" })</Networking>
  <vGPU>Disable</vGPU>
  <MappedFolders>
    <MappedFolder>
      <HostFolder>$msiFolder</HostFolder>
      <SandboxFolder>C:\TestMSI</SandboxFolder>
      <ReadOnly>true</ReadOnly>
    </MappedFolder>
  </MappedFolders>
  <LogonCommand>
    <Command>cmd /c echo MSI ready at C:\TestMSI\$msiFile</Command>
  </LogonCommand>
</Configuration>
"@

$wsbPath = "$env:TEMP\$SandboxName.wsb"
$wsb | Out-File -FilePath $wsbPath -Encoding UTF8

Write-Host "Starting Windows Sandbox..." -ForegroundColor Cyan
Start-Process $wsbPath
Write-Host "Sandbox configuration: $wsbPath" -ForegroundColor Gray
"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result_installed() {
        let result = CheckResult::installed("wix", "5.0.0", None);
        assert_eq!(result.status, PrereqStatus::Installed);
        assert_eq!(result.version, Some("5.0.0".to_string()));
    }

    #[test]
    fn test_check_result_not_installed() {
        let result = CheckResult::not_installed("wix");
        assert_eq!(result.status, PrereqStatus::NotInstalled);
        assert!(result.version.is_none());
    }

    #[test]
    fn test_check_result_outdated() {
        let result = CheckResult::outdated("dotnet", "5.0.0", "6.0.0");
        assert_eq!(result.status, PrereqStatus::Outdated);
        assert!(result.message.contains("outdated"));
    }

    #[test]
    fn test_check_result_unknown() {
        let result = CheckResult::unknown("test", "command failed");
        assert_eq!(result.status, PrereqStatus::Unknown);
    }

    #[test]
    fn test_environment_check_ready() {
        let prereqs = default_prerequisites();
        let results = vec![
            CheckResult::installed("dotnet", "8.0.0", None),
            CheckResult::installed("wix", "5.0.0", None),
        ];
        let check = EnvironmentCheck::new(results, &prereqs);
        assert!(check.ready);
        assert!(check.missing_required.is_empty());
    }

    #[test]
    fn test_environment_check_not_ready() {
        let prereqs = default_prerequisites();
        let results = vec![
            CheckResult::installed("dotnet", "8.0.0", None),
            CheckResult::not_installed("wix"),
        ];
        let check = EnvironmentCheck::new(results, &prereqs);
        assert!(!check.ready);
        assert!(check.missing_required.contains(&"wix".to_string()));
    }

    #[test]
    fn test_sandbox_config_new() {
        let config = SandboxConfig::new("test");
        assert_eq!(config.name, "test");
        assert_eq!(config.memory_mb, Some(4096));
        assert!(config.networking);
    }

    #[test]
    fn test_sandbox_config_for_msi_testing() {
        let config = SandboxConfig::for_msi_testing("test", PathBuf::from("C:\\MSI"));
        assert!(!config.networking);
        assert_eq!(config.mapped_folders.len(), 1);
        assert!(config.mapped_folders[0].read_only);
    }

    #[test]
    fn test_sandbox_config_for_wix_development() {
        let config = SandboxConfig::for_wix_development("dev", PathBuf::from("C:\\Project"));
        assert!(config.networking);
        assert_eq!(config.memory_mb, Some(8192));
        assert!(config.logon_command.is_some());
    }

    #[test]
    fn test_sandbox_config_builder() {
        let config = SandboxConfig::new("test")
            .with_memory(8192)
            .with_networking(false)
            .with_vgpu(true);
        assert_eq!(config.memory_mb, Some(8192));
        assert!(!config.networking);
        assert!(config.vgpu);
    }

    #[test]
    fn test_sandbox_config_to_wsb() {
        let config = SandboxConfig::new("test")
            .with_memory(4096)
            .with_networking(true);
        let wsb = config.to_wsb();
        assert!(wsb.contains("<Configuration>"));
        assert!(wsb.contains("<MemoryInMB>4096</MemoryInMB>"));
        assert!(wsb.contains("<Networking>Enable</Networking>"));
    }

    #[test]
    fn test_sandbox_with_mapped_folder() {
        let config = SandboxConfig::new("test")
            .add_folder(
                PathBuf::from("C:\\Projects\\MyApp"),
                Some("C:\\Sandbox\\MyApp".to_string()),
                true,
            );
        let wsb = config.to_wsb();
        assert!(wsb.contains("<MappedFolders>"));
        assert!(wsb.contains("<HostFolder>C:\\Projects\\MyApp</HostFolder>"));
        assert!(wsb.contains("<ReadOnly>true</ReadOnly>"));
    }

    #[test]
    fn test_sandbox_with_logon_command() {
        let config = SandboxConfig::new("test")
            .with_logon_command("powershell -File setup.ps1");
        let wsb = config.to_wsb();
        assert!(wsb.contains("<LogonCommand>"));
        assert!(wsb.contains("<Command>powershell -File setup.ps1</Command>"));
    }

    #[test]
    fn test_offline_package_new() {
        let pkg = OfflinePackage::new("test", PathBuf::from("C:\\Offline"));
        assert_eq!(pkg.name, "test");
        assert!(pkg.components.is_empty());
    }

    #[test]
    fn test_offline_package_minimal() {
        let pkg = OfflinePackage::minimal(PathBuf::from("C:\\Offline"));
        assert_eq!(pkg.name, "wix-minimal");
        assert!(!pkg.components.is_empty());
        assert!(pkg.components.iter().any(|c| c.id == "dotnet8"));
    }

    #[test]
    fn test_offline_package_standard() {
        let pkg = OfflinePackage::standard(PathBuf::from("C:\\Offline"));
        assert!(pkg.components.iter().any(|c| c.id == "dotnet8"));
        assert!(pkg.components.iter().any(|c| c.id == "git"));
    }

    #[test]
    fn test_offline_package_full() {
        let pkg = OfflinePackage::full(PathBuf::from("C:\\Offline"));
        assert!(pkg.components.iter().any(|c| c.id == "dotnet8"));
        assert!(pkg.components.iter().any(|c| c.id == "git"));
        assert!(pkg.components.iter().any(|c| c.id == "vsbuildtools"));
    }

    #[test]
    fn test_offline_package_total_size() {
        let pkg = OfflinePackage::minimal(PathBuf::from("C:\\Offline"));
        assert!(pkg.total_size_mb() > 0);
    }

    #[test]
    fn test_default_prerequisites() {
        let prereqs = default_prerequisites();
        assert!(!prereqs.is_empty());

        let dotnet = prereqs.iter().find(|p| p.id == "dotnet").unwrap();
        assert!(dotnet.required);

        let wix = prereqs.iter().find(|p| p.id == "wix").unwrap();
        assert!(wix.required);
        assert!(wix.install_args.is_some());

        let git = prereqs.iter().find(|p| p.id == "git").unwrap();
        assert!(!git.required);
    }

    #[test]
    fn test_scripts_not_empty() {
        assert!(!scripts::CHECK_ENV_PS1.is_empty());
        assert!(!scripts::INSTALL_WIX_PS1.is_empty());
        assert!(!scripts::OFFLINE_INSTALL_BAT.is_empty());
        assert!(!scripts::CREATE_SANDBOX_PS1.is_empty());
    }
}
