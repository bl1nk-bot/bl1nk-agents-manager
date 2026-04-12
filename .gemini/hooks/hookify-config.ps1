<#
.SYNOPSIS
    Hookify Configuration Manager (Gemini Edition)
    Best Practice: Manages the "disabled" list in .gemini/settings.json

.DESCRIPTION
    Reads .gemini/settings.json, lists all defined hooks, and allows interactive toggling
    of their enabled/disabled state by updating the 'hooks.disabled' list.
#>

$ErrorActionPreference = "Stop"
$SettingsFile = ".gemini/settings.json"

# --- 1. Validation ---
if (-not (Test-Path $SettingsFile)) {
    Write-Warning "Settings file not found at $SettingsFile"
    Write-Host "Creating a default one..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Force -Path ".gemini" | Out-Null
    Set-Content -Path $SettingsFile -Value '{ "hooks": { "disabled": [] } }'
}

Write-Host "🔍 Reading Gemini settings..." -ForegroundColor Cyan

# --- 2. Read & Parse JSON ---
try {
    $JsonContent = Get-Content -Path $SettingsFile -Raw
    $Settings = $JsonContent | ConvertFrom-Json
} catch {
    Write-Error "Failed to parse $SettingsFile. Is it valid JSON?"
    exit 1
}

# Initialize structure if missing
if (-not $Settings.PSObject.Properties['hooks']) {
    $Settings | Add-Member -MemberType NoteProperty -Name "hooks" -Value @{}
}
if (-not $Settings.hooks.PSObject.Properties['disabled']) {
    $Settings.hooks | Add-Member -MemberType NoteProperty -Name "disabled" -Value @()
}

# --- 3. Extract Data ---
$AllHooks = @()
$DisabledHooks = $Settings.hooks.disabled

# Iterate through all hook categories (BeforeTool, AfterModel, etc.)
foreach ($Prop in $Settings.hooks.PSObject.Properties) {
    if ($Prop.Name -eq "disabled") { continue }
    
    # Check if property value is an array (Matcher Groups)
    if ($Prop.Value -is [System.Array]) {
        foreach ($Group in $Prop.Value) {
            if ($Group.hooks -is [System.Array]) {
                foreach ($Hook in $Group.hooks) {
                    if ($Hook.name) {
                        $AllHooks += $Hook.name
                    }
                }
            }
        }
    }
}

$AllHooks = $AllHooks | Select-Object -Unique | Sort-Object

if ($AllHooks.Count -eq 0) {
    Write-Warning "No hooks defined in $SettingsFile."
    Write-Host "Add some hooks to 'BeforeTool', 'AfterModel', etc. first."
    exit 0
}

# --- 4. Interactive UI ---
Write-Host "`n📋 Available Hooks:" -ForegroundColor Green
Write-Host "------------------------------------------------"
Write-Host ("{0,-5} {1,-25} {2,-10}" -f "ID", "NAME", "STATE")
Write-Host "------------------------------------------------"

$HookMap = @{}
$i = 1

foreach ($Name in $AllHooks) {
    $State = "✅ ENABLED"
    if ($DisabledHooks -contains $Name) {
        $State = "❌ DISABLED"
    }
    
    Write-Host ("{0,-5} {1,-25} {2,-10}" -f $i, $Name, $State)
    $HookMap[$i] = $Name
    $i++
}

Write-Host "------------------------------------------------"
$Selection = Read-Host "`nEnter the IDs of hooks to TOGGLE (comma separated, e.g., '1,3')"

if ([string]::IsNullOrWhiteSpace($Selection)) {
    Write-Host "No changes made."
    exit 0
}

# --- 5. Process Changes ---
$ChangesMade = $false
$CurrentDisabled = [System.Collections.ArrayList]::new()
if ($DisabledHooks) { $CurrentDisabled.AddRange($DisabledHooks) }

foreach ($Id in $Selection -split ",") {
    $Id = $Id.Trim()
    if ($HookMap.ContainsKey([int]$Id)) {
        $TargetHook = $HookMap[[int]$Id]
        
        if ($CurrentDisabled.Contains($TargetHook)) {
            Write-Host "🔄 Enabling: $TargetHook" -ForegroundColor Cyan
            $CurrentDisabled.Remove($TargetHook)
        } else {
            Write-Host "🔄 Disabling: $TargetHook" -ForegroundColor Yellow
            $CurrentDisabled.Add($TargetHook)
        }
        $ChangesMade = $true
    } else {
        Write-Warning "Invalid ID: $Id (Skipping)"
    }
}

# --- 6. Save ---
if ($ChangesMade) {
    # Convert ArrayList back to standard array for JSON serialization
    $Settings.hooks.disabled = $CurrentDisabled.ToArray() | Sort-Object -Unique
    
    # ConvertTo-Json depth is important for nested objects
    $NewJson = $Settings | ConvertTo-Json -Depth 10
    Set-Content -Path $SettingsFile -Value $NewJson
    Write-Host "`n✅ Configuration updated successfully!" -ForegroundColor Green
} else {
    Write-Host "No valid changes selected."
}
