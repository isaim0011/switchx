<#
.SYNOPSIS
    Universal AI Account Switcher Engine - Real Profile Switcher
.DESCRIPTION
    Handles real session snapshots, saves, swaps, and restarts for Antigravity, Antigravity IDE, and Cursor.
#>

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("Save", "Load", "List", "Delete")]
    [string]$Action,
    
    [Parameter(Mandatory=$false)]
    [string]$ProfileName,
    
    [Parameter(Mandatory=$false)]
    [int]$MaxProfiles = 10
)

# Detect whether we are running inside Antigravity IDE, Cursor, or classic Antigravity
$AppCandidateDirs = @(
    "$env:APPDATA\Antigravity IDE",
    "$env:APPDATA\Antigravity",
    "$env:APPDATA\Cursor",
    "$env:APPDATA\Code"
)

$ActiveAppDir = $null
foreach ($dir in $AppCandidateDirs) {
    if (Test-Path "$dir\User") {
        $ActiveAppDir = $dir
        break
    }
}

if (-not $ActiveAppDir) {
    $ActiveAppDir = "$env:APPDATA\Antigravity IDE"
}

$ProfilesStorePath = Join-Path $ActiveAppDir "Profiles"
$UserDataPath = Join-Path $ActiveAppDir "User"

if (-not (Test-Path $ProfilesStorePath)) {
    New-Item -ItemType Directory -Path $ProfilesStorePath -Force | Out-Null
}

function Get-Executable {
    $candidates = @(
        "$env:LOCALAPPDATA\Programs\Antigravity IDE\Antigravity IDE.exe",
        "$env:LOCALAPPDATA\Programs\antigravity\Antigravity.exe",
        "$env:PROGRAMFILES\Antigravity IDE\Antigravity IDE.exe",
        "$env:LOCALAPPDATA\Programs\Cursor\Cursor.exe"
    )
    foreach ($cand in $candidates) {
        if (Test-Path $cand) { return $cand }
    }
    return $null
}

function Get-Profiles {
    $profiles = @()
    if (Test-Path $ProfilesStorePath) {
        Get-ChildItem -Path $ProfilesStorePath -Directory | ForEach-Object {
            $profiles += @{
                Name = $_.Name
                Created = $_.CreationTime.ToString("yyyy-MM-dd HH:mm")
                Size = [math]::Round((Get-ChildItem $_.FullName -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1MB, 2)
            }
        }
    }
    return $profiles
}

function Save-Profile {
    param([string]$Name)
    if ($Name -match '[\\/:*?"<>|]') {
        Write-Error "Profile name contains invalid characters"
        exit 1
    }
    if (-not (Test-Path $UserDataPath)) {
        Write-Error "User data directory not found: $UserDataPath"
        exit 1
    }
    $targetPath = Join-Path $ProfilesStorePath $Name
    if (Test-Path $targetPath) {
        Remove-Item -Path $targetPath -Recurse -Force -ErrorAction SilentlyContinue
    }
    Copy-Item -Path $UserDataPath -Destination $targetPath -Recurse -Force
    @{ Success = $true; Message = "Profile '$Name' saved successfully" } | ConvertTo-Json -Compress
}

function Switch-Profile {
    param([string]$Name)
    $profilePath = Join-Path $ProfilesStorePath $Name
    if (-not (Test-Path $profilePath)) {
        Write-Error "Profile '$Name' does not exist"
        exit 1
    }

    $exePath = Get-Executable
    $procNames = @("Antigravity IDE", "Antigravity", "Cursor")

    # Stop processes
    foreach ($pName in $procNames) {
        $procs = Get-Process -Name $pName -ErrorAction SilentlyContinue
        if ($procs) {
            $procs | Stop-Process -Force -ErrorAction SilentlyContinue
        }
    }
    Start-Sleep -Seconds 2

    # Swap User directory
    $backupPath = "${UserDataPath}_backup"
    if (Test-Path $backupPath) {
        Remove-Item -Path $backupPath -Recurse -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $UserDataPath) {
        Rename-Item -Path $UserDataPath -NewName "${UserDataPath}_backup" -Force -ErrorAction SilentlyContinue
    }

    Copy-Item -Path $profilePath -Destination $UserDataPath -Recurse -Force

    # Restart
    if ($exePath) {
        Start-Sleep -Seconds 1
        Start-Process "explorer.exe" -ArgumentList "`"$exePath`""
    }

    @{ Success = $true; Message = "Profile '$Name' loaded and editor restarted"; Restarted = ($exePath -ne $null) } | ConvertTo-Json -Compress
}

function Remove-Profile {
    param([string]$Name)
    $profilePath = Join-Path $ProfilesStorePath $Name
    if (Test-Path $profilePath) {
        Remove-Item -Path $profilePath -Recurse -Force -ErrorAction SilentlyContinue
    }
    @{ Success = $true; Message = "Profile '$Name' deleted" } | ConvertTo-Json -Compress
}

function List-Profiles {
    $profiles = @(Get-Profiles)
    @{ Profiles = $profiles; Count = $profiles.Length } | ConvertTo-Json -Depth 3 -Compress
}

switch ($Action) {
    "Save"   { Save-Profile -Name $ProfileName }
    "Load"   { Switch-Profile -Name $ProfileName }
    "List"   { List-Profiles }
    "Delete" { Remove-Profile -Name $ProfileName }
}
