[CmdletBinding()]
param()

# Import the Task SDK
Import-Module "$PSScriptRoot\ps_modules\VstsTaskSdk\VstsTaskSdk.psm1" -ErrorAction SilentlyContinue

# Get inputs
$files = Get-VstsInput -Name 'files' -Require
$failOnSeverity = Get-VstsInput -Name 'failOnSeverity'
$minSeverity = Get-VstsInput -Name 'minSeverity'
$outputFormat = Get-VstsInput -Name 'outputFormat'
$sarifOutput = Get-VstsInput -Name 'sarifOutput'

Write-Host "##[section]WiX Security Scan Task"
Write-Host "Files: $files"
Write-Host "Fail on: $failOnSeverity"

# Find wix-security executable
$wixSecurity = Get-Command 'wix-security' -ErrorAction SilentlyContinue
if (-not $wixSecurity) {
    $wixSecurity = Join-Path $env:WIXCRAFT_PATH 'wix-security.exe'
    if (-not (Test-Path $wixSecurity)) {
        Write-Error "wix-security not found. Install WixCraft tools or set WIXCRAFT_PATH."
        exit 1
    }
}

# Resolve file pattern
$targetFiles = Get-ChildItem -Path $files -Recurse -File -ErrorAction SilentlyContinue
if (-not $targetFiles) {
    Write-Host "##[warning]No files matching pattern: $files"
    exit 0
}

Write-Host "Found $($targetFiles.Count) file(s) to scan"

$hasCritical = $false
$hasHigh = $false
$hasMedium = $false
$hasLow = $false
$allFindings = @()

foreach ($file in $targetFiles) {
    Write-Host "##[command]Scanning: $($file.FullName)"

    $args = @('scan', '--min-severity', $minSeverity)

    if ($outputFormat -eq 'sarif') {
        $args += '--format', 'sarif'
    } elseif ($outputFormat -eq 'json') {
        $args += '--format', 'json'
    }

    $args += $file.FullName

    $output = & $wixSecurity $args 2>&1

    if ($outputFormat -eq 'text') {
        Write-Host $output
    } else {
        try {
            $result = $output | ConvertFrom-Json
            $allFindings += $result
        } catch {
            Write-Host $output
        }
    }

    # Check for severity levels in output
    if ($output -match 'CRITICAL|Critical') { $hasCritical = $true }
    if ($output -match 'HIGH|High') { $hasHigh = $true }
    if ($output -match 'MEDIUM|Medium') { $hasMedium = $true }
    if ($output -match 'LOW|Low') { $hasLow = $true }
}

# Write SARIF output if requested
if ($outputFormat -eq 'sarif' -and $sarifOutput -and $allFindings) {
    $allFindings | ConvertTo-Json -Depth 10 | Out-File -FilePath $sarifOutput -Encoding utf8
    Write-Host "SARIF output written to: $sarifOutput"

    # Upload SARIF for code scanning
    Write-Host "##vso[artifact.upload artifactname=CodeAnalysisLogs]$sarifOutput"
}

# Summarize findings
Write-Host ""
Write-Host "##[section]Security Scan Summary"

$shouldFail = $false
switch ($failOnSeverity) {
    'critical' { if ($hasCritical) { $shouldFail = $true } }
    'high' { if ($hasCritical -or $hasHigh) { $shouldFail = $true } }
    'medium' { if ($hasCritical -or $hasHigh -or $hasMedium) { $shouldFail = $true } }
    'low' { if ($hasCritical -or $hasHigh -or $hasMedium -or $hasLow) { $shouldFail = $true } }
    'info' { if ($hasCritical -or $hasHigh -or $hasMedium -or $hasLow) { $shouldFail = $true } }
}

if ($hasCritical) {
    Write-Host "##[error]Critical security vulnerabilities found!"
}
if ($hasHigh) {
    Write-Host "##[warning]High severity findings detected"
}
if ($hasMedium) {
    Write-Host "##[warning]Medium severity findings detected"
}

if ($shouldFail) {
    Write-Host "##[error]Security scan failed - findings exceed threshold ($failOnSeverity)"
    exit 1
}

Write-Host "##[section]Security scan completed successfully"
