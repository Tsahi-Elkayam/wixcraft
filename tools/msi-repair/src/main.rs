//! msi-repair CLI - MSI repair analyzer and troubleshooter
//!
//! Usage:
//!   msi-repair analyze repair.log     # Analyze repair log
//!   msi-repair guide                  # Show general repair guide
//!   msi-repair error 1603             # Get solutions for error code
//!   msi-repair check {ProductCode}    # Check repair readiness (Windows only)

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use msi_repair::{RepairAnalyzer, Severity};

#[derive(Parser)]
#[command(name = "msi-repair")]
#[command(about = "MSI repair analyzer and troubleshooter")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze an MSI repair log file
    Analyze {
        /// Path to verbose MSI log file
        log_file: PathBuf,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Show all log entries (not just issues)
        #[arg(short, long)]
        verbose: bool,
    },

    /// Get solutions for a specific MSI error code
    Error {
        /// MSI error code (e.g., 1603, 1706)
        code: u32,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show general repair troubleshooting guide
    Guide {
        /// Specific topic (source, cache, uac, permissions)
        #[arg(short, long)]
        topic: Option<String>,
    },

    /// Generate repair command for a product
    Command {
        /// Product code (GUID) or path to MSI
        product: String,

        /// Repair mode: all, files, registry, shortcuts
        #[arg(short, long, default_value = "all")]
        mode: String,

        /// Include verbose logging
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show common repair scenarios and solutions
    Scenarios,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let analyzer = RepairAnalyzer::new();

    match cli.command {
        Commands::Analyze {
            log_file,
            format,
            verbose,
        } => {
            let content = fs::read_to_string(&log_file)?;
            let result = analyzer.analyze_log(&content);

            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("MSI Repair Log Analysis: {}", log_file.display());
                println!("{}", "=".repeat(60));
                println!();

                // Summary
                println!("Repair Type: {}", result.repair_type);
                println!("Status: {}", if result.success { "SUCCESS" } else { "FAILED" });
                if let Some(code) = result.error_code {
                    println!("Error Code: {}", code);
                }
                if let Some(ref action) = result.failed_action {
                    println!("Failed Action: {}", action);
                }
                println!();

                // Issues
                if result.issues.is_empty() {
                    println!("No specific issues detected.");
                } else {
                    println!("Issues Found: {}", result.issues.len());
                    println!("{}", "-".repeat(60));

                    for issue in &result.issues {
                        let severity = match issue.severity {
                            Severity::Critical => "CRITICAL",
                            Severity::Error => "ERROR",
                            Severity::Warning => "WARNING",
                            Severity::Info => "INFO",
                        };

                        println!("[{}] {:?}", severity, issue.category);
                        println!("  {}", issue.message);
                        println!("  Suggestion: {}", issue.suggestion);
                        if let Some(ref kb) = issue.kb_article {
                            println!("  Reference: {}", kb);
                        }
                        println!();
                    }
                }

                // Verbose log entries
                if verbose && !result.entries.is_empty() {
                    println!("Key Log Entries:");
                    println!("{}", "-".repeat(60));
                    for entry in &result.entries {
                        println!(
                            "Line {}: [{}] {}",
                            entry.line_number,
                            entry.level,
                            truncate(&entry.message, 70)
                        );
                    }
                    println!();
                }

                // Generate troubleshooting guide
                if !result.issues.is_empty() {
                    println!();
                    let guide = analyzer.generate_troubleshooting_guide(&result.issues);
                    println!("{}", guide);
                }
            }
        }

        Commands::Error { code, format } => {
            let solutions = analyzer.get_solutions_for_error(code);

            if format == "json" {
                let output = serde_json::json!({
                    "error_code": code,
                    "solutions": solutions
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("MSI Error Code: {}", code);
                println!("{}", "=".repeat(40));
                println!();

                let description = get_error_description(code);
                println!("Description: {}", description);
                println!();

                println!("Possible Solutions:");
                for (i, solution) in solutions.iter().enumerate() {
                    println!("  {}. {}", i + 1, solution);
                }
                println!();

                // Additional resources
                println!("Resources:");
                println!("  - Microsoft Docs: https://docs.microsoft.com/windows/win32/msi/error-codes");
                println!("  - Search KB: https://support.microsoft.com/search?query=error+{}", code);
            }
        }

        Commands::Guide { topic } => {
            match topic.as_deref() {
                Some("source") => {
                    println!("{}", GUIDE_SOURCE);
                }
                Some("cache") => {
                    println!("{}", GUIDE_CACHE);
                }
                Some("uac") => {
                    println!("{}", GUIDE_UAC);
                }
                Some("permissions") => {
                    println!("{}", GUIDE_PERMISSIONS);
                }
                _ => {
                    println!("{}", GUIDE_GENERAL);
                }
            }
        }

        Commands::Command {
            product,
            mode,
            verbose,
        } => {
            let repair_flags = match mode.as_str() {
                "files" => "f",
                "registry" => "um",
                "shortcuts" => "s",
                "all" | _ => "vomus",
            };

            let product_ref = if product.starts_with('{') {
                product.clone()
            } else {
                format!("\"{}\"", product)
            };

            let log_flag = if verbose {
                " /l*v repair.log"
            } else {
                ""
            };

            println!("Repair Command:");
            println!();
            println!("  msiexec /f{} {}{}", repair_flags, product_ref, log_flag);
            println!();

            println!("Repair Flags Reference:");
            println!("  p - Reinstall if file missing");
            println!("  o - Reinstall if older or missing");
            println!("  e - Reinstall if equal or older");
            println!("  d - Reinstall if different version");
            println!("  c - Verify checksum, reinstall if corrupt");
            println!("  a - Reinstall all files");
            println!("  u - Rewrite user registry keys");
            println!("  m - Rewrite machine registry keys");
            println!("  s - Recreate shortcuts");
            println!("  v - Recache from source");
            println!();

            println!("Common Combinations:");
            println!("  /fa      - Full reinstall of all files");
            println!("  /fvomus  - Full repair (files + registry + shortcuts + recache)");
            println!("  /fu      - Repair user settings only");
        }

        Commands::Scenarios => {
            println!("{}", SCENARIOS);
        }
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

fn get_error_description(code: u32) -> &'static str {
    match code {
        1601 => "Windows Installer service could not be accessed",
        1602 => "User cancelled installation",
        1603 => "Fatal error during installation",
        1604 => "Installation suspended, incomplete",
        1605 => "This action is only valid for products that are currently installed",
        1606 => "Feature ID not registered",
        1607 => "Component ID not registered",
        1608 => "Unknown property",
        1609 => "Handle is in an invalid state",
        1610 => "Configuration data for this product is corrupt",
        1611 => "Component qualifier not present",
        1612 => "Installation source for this product is not available",
        1613 => "This installation package cannot be installed by Windows Installer service",
        1614 => "Product is uninstalled",
        1615 => "SQL query syntax invalid",
        1616 => "Record field does not exist",
        1618 => "Another installation is already in progress",
        1619 => "This installation package could not be opened",
        1620 => "This installation package could not be opened (invalid format)",
        1621 => "Windows Installer service failed to start",
        1622 => "Error opening installation log file",
        1623 => "Language not supported by this installation",
        1624 => "Error applying transforms",
        1625 => "Installation is forbidden by system policy",
        1626 => "Function could not be executed",
        1627 => "Function failed during execution",
        1628 => "Invalid or unknown table specified",
        1629 => "Data supplied is of wrong type",
        1630 => "Data of this type is not supported",
        1631 => "Windows Installer service failed to start",
        1632 => "Temp folder is full or inaccessible",
        1633 => "Installation package not supported by this processor type",
        1634 => "Component not used on this computer",
        1635 => "Patch package could not be opened",
        1636 => "Patch package cannot be applied",
        1637 => "Patch package cannot be applied - no transformations",
        1638 => "Another version of this product is already installed",
        1639 => "Invalid command line argument",
        1640 => "Only administrators have permission to add/remove in Terminal Services",
        1641 => "Requested operation completed successfully - reboot required",
        1642 => "Patch package cannot be applied (superseded or incorrect order)",
        1643 => "Patch package cannot be applied (requires newer version)",
        1644 => "One or more customizations are not permitted by software restriction policy",
        1645 => "Windows Installer cannot install upgrade patch (not found)",
        1646 => "Patch package cannot be applied (invalid for uninstall)",
        1647 => "Patch is not applied to this product",
        1648 => "No valid sequence could be found for the set of updates",
        1649 => "Patch removal was disallowed by policy",
        1650 => "XML patch data is invalid",
        1651 => "Windows Installer is not accessible in Safe Mode",
        1652 => "Rollback has been disabled",
        1653 => "Rollback is in progress",
        1654 => "Required resources could not be locked",
        1655 => "Unable to apply patch",
        1656 => "Invalid command line argument combination",
        1657 => "Restart Manager failed to start",
        1658 => "Restart Manager session already exists",
        1659 => "Restart Manager could not restart system",
        1660 => "Restart Manager session has a pending restart",
        1706 => "No valid source could be found for this product",
        1707 => "Installation was aborted by user",
        _ => "Unknown error code",
    }
}

const GUIDE_GENERAL: &str = r#"# MSI Repair Troubleshooting Guide

## Overview

MSI repair restores installed products to their original state by:
- Reinstalling missing or modified files
- Restoring registry entries
- Recreating shortcuts
- Verifying and recaching the installation package

## Quick Reference

### Repair Commands

```
msiexec /fa {ProductCode}     # Full file repair
msiexec /fvomus {ProductCode} # Complete repair (recommended)
msiexec /fu {ProductCode}     # User registry only
msiexec /fm {ProductCode}     # Machine registry only
```

### Always Use Verbose Logging

```
msiexec /fa {ProductCode} /l*v repair.log
```

## Common Issues

### 1. Source Not Available (Error 1706)

The original installation media is required but not found.

**Solutions:**
- Provide source path: `SOURCELIST="C:\Path\To\Source"`
- Create administrative install point
- Recache MSI: `msiexec /fv package.msi`

### 2. Cached MSI Missing

The MSI in `C:\Windows\Installer` is missing.

**Solutions:**
- Recache with original MSI
- Use Microsoft Fixit tool
- Reinstall the product

### 3. UAC Prompts During Silent Repair

Windows security updates may cause unexpected prompts.

**Solutions:**
- Use scheduled task with SYSTEM account
- Deploy via SCCM/Intune
- Explicit elevation before repair

### 4. Permission Errors

Access denied to files or registry.

**Solutions:**
- Run as Administrator
- Take ownership of files
- Check NTFS permissions
- Disable antivirus temporarily

## Topics

Use `msi-repair guide --topic <topic>` for detailed guides:
- `source` - Source availability issues
- `cache` - Windows Installer cache
- `uac` - UAC and elevation issues
- `permissions` - Permission troubleshooting
"#;

const GUIDE_SOURCE: &str = r#"# Source Availability Guide

## Problem

The Windows Installer needs access to the original installation source
to perform repairs, but the source is no longer available.

## Common Causes

1. Network share moved or renamed
2. USB/DVD media removed
3. Downloaded installer deleted
4. Installation source path changed

## Solutions

### 1. Register Alternate Source

```
msiexec /i {ProductCode} REINSTALL=ALL REINSTALLMODE=vomus \
        SOURCELIST="C:\NewPath\To\Source"
```

### 2. Administrative Install Point

Create a network source that's always available:

```
msiexec /a package.msi TARGETDIR=\\server\share\app
```

Then repair using:

```
msiexec /i {ProductCode} REINSTALL=ALL \
        SOURCELIST="\\server\share\app"
```

### 3. Recache MSI

If you have the original MSI but it's not registered:

```
msiexec /fv path\to\original.msi
```

### 4. Check Current Sources

Registry location:
```
HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Installer\UserData\
<SID>\Products\<ProductCode>\InstallProperties
```

Check `InstallSource` value.

### 5. For Enterprise

- Host installers on always-available network share
- Use SCCM/Intune for deployment
- Consider embedded CAB files in MSI
"#;

const GUIDE_CACHE: &str = r#"# Windows Installer Cache Guide

## Overview

Windows stores cached MSI packages in `C:\Windows\Installer` (hidden folder).
These are used for repairs, patches, and uninstalls.

## Problem

If cached MSI is deleted or corrupted:
- Repairs fail
- Patches cannot apply
- Uninstall may fail

## Check Cache Status

1. Find product's cached MSI:
   - Registry: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Installer\UserData\<SID>\Products\<ProductCode>\InstallProperties`
   - Look for `LocalPackage` value

2. Verify file exists:
   - Navigate to `C:\Windows\Installer`
   - Check for the .msi file

## Solutions

### 1. Recache from Source

```
msiexec /fv path\to\original.msi
```

### 2. Microsoft Program Install/Uninstall Troubleshooter

Download from: https://support.microsoft.com/help/17588

This tool can:
- Repair corrupted registry entries
- Remove stuck installations
- Clean orphaned cache entries

### 3. Manual Repair

If you have the original MSI:

```
copy original.msi C:\Windows\Installer\<CachedName>.msi
```

Note: The cached name is a random hex string.

### 4. Prevention

- Never manually delete files from `C:\Windows\Installer`
- Use proper uninstall procedures
- Keep installation media available
"#;

const GUIDE_UAC: &str = r#"# UAC and Elevation Guide

## Background

Starting with Windows security updates in mid-2024, MSI repair operations
may trigger unexpected UAC prompts even during "silent" repairs.

## Affected Scenarios

- Silent repairs initiated by applications
- Repair operations via management tools
- Self-healing functionality

## Related Updates

- KB5041585 (August 2024)
- Related security patches

## Solutions

### 1. Use Scheduled Task

Create a task that runs with highest privileges:

```powershell
$action = New-ScheduledTaskAction -Execute "msiexec.exe" `
    -Argument "/fa {ProductCode} /qn"
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" `
    -LogonType ServiceAccount -RunLevel Highest
$task = New-ScheduledTask -Action $action -Principal $principal
Register-ScheduledTask -TaskName "MSI Repair" -InputObject $task
Start-ScheduledTask -TaskName "MSI Repair"
```

### 2. Enterprise Deployment

Use deployment tools that run in SYSTEM context:
- SCCM/ConfigMgr
- Microsoft Intune
- Group Policy Software Installation

### 3. Explicit Elevation

Before repair, elevate the command prompt:

```
runas /user:Administrator "msiexec /fa {ProductCode} /qn"
```

### 4. Check Event Log

Windows Event Viewer > Application and Services Logs > Microsoft > Windows > UAC

Look for repair-related elevation requests.

## Microsoft Guidance

See: https://support.microsoft.com/kb/5041585
"#;

const GUIDE_PERMISSIONS: &str = r#"# Permission Troubleshooting Guide

## Common Permission Errors

- Error 1925: Insufficient privileges
- Access Denied during file copy
- Cannot modify registry key

## Diagnostic Steps

### 1. Check Installation Folder

```cmd
icacls "C:\Program Files\YourApp"
```

Expected: SYSTEM and Administrators should have Full Control

### 2. Check Registry

Use regedit to check:
- `HKLM\SOFTWARE\YourCompany\YourApp`
- Verify SYSTEM has Full Control

### 3. Check User Permissions

Run `whoami /groups` to verify group memberships.

## Solutions

### 1. Run as Administrator

Always try elevated command prompt first:

1. Right-click Command Prompt
2. Select "Run as administrator"
3. Execute repair command

### 2. Take Ownership of Files

```cmd
takeown /f "C:\Program Files\YourApp" /r
icacls "C:\Program Files\YourApp" /grant Administrators:F /t
```

### 3. Reset Registry Permissions

Using SubInACL (Microsoft tool):

```cmd
subinacl /subkeyreg HKEY_LOCAL_MACHINE\SOFTWARE\YourApp /grant=Administrators=f
```

### 4. Check for Locked Files

Use tools like Process Explorer or Handle to find what's locking files:

```cmd
handle.exe "C:\Program Files\YourApp"
```

### 5. Safe Mode Repair

Boot into Safe Mode to repair without third-party interference:

1. Boot to Safe Mode with Networking
2. Run repair from elevated prompt
3. Restart normally

### 6. Antivirus Interference

Temporarily disable real-time protection if antivirus is blocking:
- Windows Defender
- Third-party AV software

Remember to re-enable after repair.
"#;

const SCENARIOS: &str = r#"# Common MSI Repair Scenarios

## Scenario 1: Application Won't Start After Update

**Symptoms:**
- Application fails to start
- Missing DLL errors
- "Application configuration incorrect"

**Repair:**
```
msiexec /fa {ProductCode} /l*v repair.log
```

**If that fails:**
- Check repair.log for errors
- Try full repair: `msiexec /fvomus {ProductCode}`

---

## Scenario 2: Shortcuts Missing

**Symptoms:**
- Desktop/Start Menu shortcuts gone
- Only after Windows update

**Repair:**
```
msiexec /fs {ProductCode}
```

---

## Scenario 3: Settings Reset After Repair

**Symptoms:**
- User preferences lost
- Configuration needs redo

**Cause:**
- Repair reinstalled config files

**Prevention:**
- Use proper component design
- Config files should be `NeverOverwrite`

---

## Scenario 4: Repair Prompts for Source

**Symptoms:**
- Dialog asking for installation media
- Cannot find source path

**Solutions:**
```
msiexec /i {ProductCode} REINSTALL=ALL REINSTALLMODE=vomus \
        SOURCELIST="C:\Path\To\Source"
```

---

## Scenario 5: Silent Repair Shows UAC Prompt

**Symptoms:**
- Automated repair interrupted by UAC
- Scripts hang waiting for user input

**Workaround:**
1. Use scheduled task with SYSTEM account
2. Elevate explicitly before repair
3. Deploy via SCCM/Intune

See: `msi-repair guide --topic uac`

---

## Scenario 6: Repair Hangs or Takes Forever

**Symptoms:**
- msiexec.exe at 0% for extended time
- Repair never completes

**Diagnostics:**
1. Check for hung msiexec processes
2. Verify disk space
3. Check for file locks

**Solutions:**
```
taskkill /f /im msiexec.exe
net stop msiserver
net start msiserver
```

Then retry repair.

---

## Scenario 7: Partial Repair Needed

**Symptoms:**
- Only some files corrupted
- Don't want full reinstall

**Targeted Repair:**
```
# Just missing files
msiexec /fp {ProductCode}

# Verify checksums
msiexec /fc {ProductCode}

# Different version files only
msiexec /fd {ProductCode}
```

---

## Scenario 8: Enterprise Deployment Issue

**Symptoms:**
- Repair works manually but fails via SCCM
- Silent install returns error

**Check:**
1. Run from elevated SYSTEM context
2. Verify source is accessible to SYSTEM
3. Check for user-specific paths

**SCCM Detection:**
- Use return code 1641 as success (reboot required)
- Use return code 3010 as soft reboot
"#;
