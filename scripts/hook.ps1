# Env Switcher Hook Script
# This file is auto-generated and should not be edited manually.
# Use the Env Switcher application to manage configurations.

$EnvSwitcherConfigPath = "$env:LOCALAPPDATA\env-switcher\config.json"
$EnvSwitcherDataDir = "$env:LOCALAPPDATA\env-switcher"

# Environment state tracking
$script:CurrentEnvPath = $null
$script:CurrentNodeHome = $null
$script:CurrentJavaHome = $null
$script:OriginalPath = $env:PATH

function Find-PathMapping {
    param(
        [string]$targetPath,
        [string]$configPath
    )

    if (-not (Test-Path $configPath)) {
        return $null
    }

    try {
        $config = Get-Content $configPath -Raw | ConvertFrom-Json
        $targetNormalized = $targetPath.ToLower().Replace('/', '\')

        $bestMatch = $null
        $bestPrefixLen = 0

        foreach ($mapping in $config.path_mappings) {
            $mappingNormalized = $mapping.path.ToLower().Replace('/', '\')

            # Exact match
            if ($mappingNormalized -eq $targetNormalized) {
                $bestMatch = $mapping
                break
            }

            # Prefix match (subdirectory inheritance)
            if ($targetNormalized.StartsWith($mappingNormalized + '\')) {
                if ($mappingNormalized.Length -gt $bestPrefixLen) {
                    $bestPrefixLen = $mappingNormalized.Length
                    $bestMatch = $mapping
                }
            }
        }

        return $bestMatch
    }
    catch {
        Write-Warning "EnvSwitcher: Error reading config: $_"
        return $null
    }
}

function Switch-Environment {
    param([string]$targetPath)

    # Skip if path unchanged
    if ($script:CurrentEnvPath -eq $targetPath) {
        return
    }

    $mapping = Find-PathMapping -targetPath $targetPath -configPath $EnvSwitcherConfigPath

    if ($mapping) {
        # Reset PATH to original
        $env:PATH = $script:OriginalPath

        # Read config for version details
        $config = Get-Content $configPath -Raw | ConvertFrom-Json

        # Apply Node.js environment
        if ($mapping.node_version -and $config.node_versions) {
            $nodeConfig = $config.node_versions.($mapping.node_version)
            if ($nodeConfig -and $nodeConfig.path) {
                $nodePath = $nodeConfig.path
                $env:PATH = "$nodePath;$env:PATH"
                $env:NODE_HOME = $nodePath
                $script:CurrentNodeHome = $nodePath

                Write-Host "🟢 Node: $($mapping.node_version)" -ForegroundColor Green
            }
        }
        else {
            $env:NODE_HOME = $null
            $script:CurrentNodeHome = $null
        }

        # Apply Java environment
        if ($mapping.java_version -and $config.java_versions) {
            $javaConfig = $config.java_versions.($mapping.java_version)
            if ($javaConfig -and $javaConfig.path) {
                $javaHome = $javaConfig.path
                $env:JAVA_HOME = $javaHome
                $env:PATH = "$javaHome\bin;$env:PATH"
                $script:CurrentJavaHome = $javaHome

                Write-Host "🟢 Java: $($mapping.java_version)" -ForegroundColor Cyan
            }
        }
        else {
            $env:JAVA_HOME = $null
            $script:CurrentJavaHome = $null
        }

        $script:CurrentEnvPath = $targetPath
    }
    else {
        # No mapping found - restore original environment
        if ($null -ne $script:OriginalPath) {
            $env:PATH = $script:OriginalPath
        }
        $env:NODE_HOME = $null
        $env:JAVA_HOME = $null
        $script:CurrentEnvPath = $targetPath

        Write-Host "⚪ Using system environment" -ForegroundColor Gray
    }
}

# Hook into Set-Location (cd command)
$wrappedCmd = Get-Command Set-Location -CommandType Cmdlet

function global:Set-Location {
    param(
        [string]$Path,
        [switch]$PassThru,
        [string]$StackName
    )

    # Execute the actual cd command
    if ($Path) {
        & $wrappedCmd -Path $Path -PassThru:$PassThru -StackName $StackName
    }
    else {
        & $wrappedCmd -PassThru:$PassThru -StackName $StackName
    }

    # Switch environment based on new location
    if ($PWD) {
        Switch-Environment -targetPath $PWD.Path
    }
}

# Hook Push-Location (pushd)
$wrappedPush = Get-Command Push-Location -CommandType Cmdlet
function global:Push-Location {
    param(
        [string]$Path,
        [switch]$PassThru,
        [string]$StackName
    )

    if ($Path) {
        & $wrappedPush -Path $Path -PassThru:$PassThru -StackName $StackName
    }
    else {
        & $wrappedPush -PassThru:$PassThru -StackName $StackName
    }

    if ($PWD) {
        Switch-Environment -targetPath $PWD.Path
    }
}

# Hook Pop-Location (popd)
$wrappedPop = Get-Command Pop-Location -CommandType Cmdlet
function global:Pop-Location {
    param(
        [string]$StackName
    )

    & $wrappedPop -StackName $StackName

    if ($PWD) {
        Switch-Environment -targetPath $PWD.Path
    }
}

# Initial environment check on load
Write-Host "⚙️  Env Switcher loaded" -ForegroundColor Gray
if ($PWD) {
    Switch-Environment -targetPath $PWD.Path
}

# Debug helper function
function global:Get-EnvSwitcherStatus {
    [PSCustomObject]@{
        CurrentPath = $script:CurrentEnvPath
        NodeHome = $script:CurrentNodeHome
        JavaHome = $script:CurrentJavaHome
        ConfigPath = $EnvSwitcherConfigPath
    }
}
