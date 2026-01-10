[CmdletBinding()]
param()

# Import the Task SDK
Import-Module "$PSScriptRoot\ps_modules\VstsTaskSdk\VstsTaskSdk.psm1" -ErrorAction SilentlyContinue

# Get inputs
$files = Get-VstsInput -Name 'files' -Require
$failOnSeverity = Get-VstsInput -Name 'failOnSeverity'
$enableComparison = Get-VstsInput -Name 'enableComparison' -AsBool
$baselineFiles = Get-VstsInput -Name 'baselineFiles'
$baselineBranch = Get-VstsInput -Name 'baselineBranch'
$outputFormat = Get-VstsInput -Name 'outputFormat'

Write-Host "##[section]WiX Upgrade Validation Task"
Write-Host "Files: $files"
Write-Host "Fail on: $failOnSeverity"

# Find wix-upgrade executable
$wixUpgrade = Get-Command 'wix-upgrade' -ErrorAction SilentlyContinue
if (-not $wixUpgrade) {
    $wixUpgrade = Join-Path $env:WIXCRAFT_PATH 'wix-upgrade.exe'
    if (-not (Test-Path $wixUpgrade)) {
        Write-Error "wix-upgrade not found. Install WixCraft tools or set WIXCRAFT_PATH."
        exit 1
    }
}

# Resolve file pattern
$targetFiles = Get-ChildItem -Path $files -Recurse -File -ErrorAction SilentlyContinue
if (-not $targetFiles) {
    Write-Host "##[warning]No files matching pattern: $files"
    exit 0
}

Write-Host "Found $($targetFiles.Count) file(s) to validate"

$hasErrors = $false
$hasWarnings = $false

if ($enableComparison -and ($baselineFiles -or $baselineBranch)) {
    Write-Host "##[section]Running Version Comparison"

    # Get baseline files
    $baselineTargets = @()

    if ($baselineFiles) {
        $baselineTargets = Get-ChildItem -Path $baselineFiles -Recurse -File -ErrorAction SilentlyContinue
    } elseif ($baselineBranch) {
        # Fetch from git branch
        $tempDir = Join-Path $env:TEMP "wix-baseline-$([guid]::NewGuid().ToString('N').Substring(0,8))"
        New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

        foreach ($file in $targetFiles) {
            $relativePath = $file.FullName.Replace((Get-Location).Path + '\', '')
            $gitShowCmd = "git show ${baselineBranch}:$relativePath"
            $baselineContent = Invoke-Expression $gitShowCmd 2>$null

            if ($baselineContent) {
                $baselineFile = Join-Path $tempDir $file.Name
                $baselineContent | Out-File -FilePath $baselineFile -Encoding utf8
                $baselineTargets += Get-Item $baselineFile
            }
        }
    }

    if ($baselineTargets.Count -gt 0) {
        $oldFiles = ($baselineTargets | ForEach-Object { $_.FullName }) -join ','
        $newFiles = ($targetFiles | ForEach-Object { $_.FullName }) -join ','

        $args = @('compare')
        foreach ($bf in $baselineTargets) {
            $args += '--old', $bf.FullName
        }
        foreach ($nf in $targetFiles) {
            $args += '--new', $nf.FullName
        }

        if ($outputFormat -eq 'json') {
            $args += '--format', 'json'
        }

        Write-Host "##[command]Comparing versions..."
        $output = & $wixUpgrade $args 2>&1
        $exitCode = $LASTEXITCODE

        Write-Host $output

        if ($exitCode -ne 0) {
            $hasErrors = $true
        }

        if ($output -match 'WARNING|Warning') {
            $hasWarnings = $true
        }

        # Cleanup temp files
        if ($tempDir -and (Test-Path $tempDir)) {
            Remove-Item -Path $tempDir -Recurse -Force
        }
    } else {
        Write-Host "##[warning]No baseline files found for comparison"
    }
} else {
    # Just validate current files
    Write-Host "##[section]Validating Upgrade Readiness"

    foreach ($file in $targetFiles) {
        Write-Host "##[command]Validating: $($file.FullName)"

        $args = @('validate')
        if ($outputFormat -eq 'json') {
            $args += '--format', 'json'
        }
        $args += $file.FullName

        $output = & $wixUpgrade $args 2>&1
        $exitCode = $LASTEXITCODE

        Write-Host $output

        if ($exitCode -ne 0) {
            $hasErrors = $true
            Write-Host "##[error]Validation issues in: $($file.Name)"
        }

        if ($output -match 'WARNING|Warning') {
            $hasWarnings = $true
        }
    }
}

# Determine exit code based on threshold
$shouldFail = $false
switch ($failOnSeverity) {
    'error' { if ($hasErrors) { $shouldFail = $true } }
    'warning' { if ($hasErrors -or $hasWarnings) { $shouldFail = $true } }
    'info' { if ($hasErrors -or $hasWarnings) { $shouldFail = $true } }
}

if ($shouldFail) {
    Write-Host "##[error]Upgrade validation failed"
    exit 1
}

Write-Host "##[section]Upgrade validation completed successfully"
