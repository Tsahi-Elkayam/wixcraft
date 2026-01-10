# WixCraft - WiX Toolset Quality Gates

Quality gates for WiX Toolset projects in Azure DevOps pipelines.

## Features

### WiX Lint
Lint WiX source files for common issues and best practices:
- Invalid element nesting
- Missing required attributes
- Deprecated elements
- Component rule violations
- GUID format issues

### WiX Security Scan
Scan for privilege escalation and security vulnerabilities:
- Deferred custom actions without impersonation (CVE-2024-38014)
- Temp folder extraction vulnerabilities (CVE-2023-26078)
- Services running as LocalSystem
- Command injection risks
- Registry persistence mechanisms

### WiX Upgrade Validator
Validate upgrade readiness and version compatibility:
- Component GUID consistency
- UpgradeCode stability
- Version format validation
- Feature tree compatibility
- Minor vs major upgrade requirements

## Usage

Add the tasks to your `azure-pipelines.yml`:

```yaml
steps:
- task: WixLint@0
  inputs:
    files: '**/*.wxs'
    failOnWarning: false

- task: WixSecurity@0
  inputs:
    files: '**/*.wxs'
    failOnSeverity: 'high'
    outputFormat: 'sarif'
    sarifOutput: '$(Build.ArtifactStagingDirectory)/wix-security.sarif'

- task: WixUpgrade@0
  inputs:
    files: '**/*.wxs'
    enableComparison: true
    baselineBranch: 'main'
```

## Requirements

- WixCraft tools must be installed on the build agent
- Set `WIXCRAFT_PATH` environment variable to the tools directory

## SARIF Integration

Security scan results can be exported in SARIF format for:
- GitHub Advanced Security
- Azure DevOps code scanning
- IDE integration

## Links

- [WixCraft on GitHub](https://github.com/wixcraft/wixcraft)
- [WiX Toolset Documentation](https://wixtoolset.org/docs/)
