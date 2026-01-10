[CmdletBinding()]
param()

# Import the Task SDK
Import-Module "$PSScriptRoot\ps_modules\VstsTaskSdk\VstsTaskSdk.psm1" -ErrorAction SilentlyContinue

# Get inputs
$files = Get-VstsInput -Name 'files' -Require
$wixDataPath = Get-VstsInput -Name 'wixDataPath'
$failOnWarning = Get-VstsInput -Name 'failOnWarning' -AsBool
$outputFormat = Get-VstsInput -Name 'outputFormat'
$sarifOutput = Get-VstsInput -Name 'sarifOutput'

Write-Host "##[section]WiX Lint Task"
Write-Host "Files: $files"

# Find wix-lint executable
$wixLint = Get-Command 'wix-lint' -ErrorAction SilentlyContinue
if (-not $wixLint) {
    $wixLint = Join-Path $env:WIXCRAFT_PATH 'wix-lint.exe'
    if (-not (Test-Path $wixLint)) {
        Write-Error "wix-lint not found. Install WixCraft tools or set WIXCRAFT_PATH."
        exit 1
    }
}

# Resolve file pattern
$targetFiles = Get-ChildItem -Path $files -Recurse -File -ErrorAction SilentlyContinue
if (-not $targetFiles) {
    Write-Host "##[warning]No files matching pattern: $files"
    exit 0
}

Write-Host "Found $($targetFiles.Count) file(s) to lint"

$hasErrors = $false
$hasWarnings = $false
$allResults = @()

foreach ($file in $targetFiles) {
    Write-Host "##[command]Linting: $($file.FullName)"

    $args = @()
    if ($wixDataPath) {
        $args += '--wix-data', $wixDataPath
    }
    if ($outputFormat -eq 'json' -or $outputFormat -eq 'sarif') {
        $args += '--format', 'json'
    }
    $args += $file.FullName

    $output = & $wixLint $args 2>&1
    $exitCode = $LASTEXITCODE

    if ($outputFormat -eq 'text') {
        Write-Host $output
    } else {
        $allResults += $output | ConvertFrom-Json -ErrorAction SilentlyContinue
    }

    if ($exitCode -ne 0) {
        $hasErrors = $true
        Write-Host "##[error]Lint issues found in: $($file.Name)"
    }

    # Check for warnings in output
    if ($output -match 'warning|WARNING') {
        $hasWarnings = $true
    }
}

# Write SARIF output if requested
if ($outputFormat -eq 'sarif' -and $sarifOutput -and $allResults) {
    $sarif = @{
        '$schema' = 'https://json.schemastore.org/sarif-2.1.0.json'
        version = '2.1.0'
        runs = @(
            @{
                tool = @{
                    driver = @{
                        name = 'wix-lint'
                        version = '0.1.0'
                    }
                }
                results = $allResults
            }
        )
    }
    $sarif | ConvertTo-Json -Depth 10 | Out-File -FilePath $sarifOutput -Encoding utf8
    Write-Host "SARIF output written to: $sarifOutput"

    # Upload SARIF for GitHub Advanced Security
    Write-Host "##vso[artifact.upload artifactname=CodeAnalysisLogs]$sarifOutput"
}

# Determine exit code
if ($hasErrors) {
    Write-Host "##[error]WiX Lint found errors"
    exit 1
}

if ($hasWarnings -and $failOnWarning) {
    Write-Host "##[error]WiX Lint found warnings (failOnWarning is enabled)"
    exit 1
}

Write-Host "##[section]WiX Lint completed successfully"
