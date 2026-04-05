[CmdletBinding()]
param(
    [string]$Version,
    [string]$Repo = $(if ($env:JIRA_CLI_REPO) { $env:JIRA_CLI_REPO } else { "Rana-Faraz/jira-cli" }),
    [string]$BinDir = $(if ($env:JIRA_CLI_INSTALL_DIR) { $env:JIRA_CLI_INSTALL_DIR } else { Join-Path $HOME "AppData\Local\Programs\jira-cli\bin" }),
    [switch]$SkipPathUpdate
)

$ErrorActionPreference = "Stop"

function Get-LatestVersion {
    param([string]$Repository)

    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repository/releases/latest"
    if (-not $release.tag_name) {
        throw "Could not resolve the latest release tag for $Repository"
    }

    return [string]$release.tag_name
}

$Target = "x86_64-pc-windows-msvc"
if (-not $Version) {
    $Version = Get-LatestVersion -Repository $Repo
}

$Asset = "jira-cli-$Version-$Target.zip"
$Url = "https://github.com/$Repo/releases/download/$Version/$Asset"
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("jira-cli-" + [System.Guid]::NewGuid().ToString("N"))

New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    $ZipPath = Join-Path $TempDir $Asset
    Write-Host "Downloading $Url"
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath

    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

    $Binary = Get-ChildItem -Path $TempDir -Recurse -Filter "jira.exe" | Select-Object -First 1
    if (-not $Binary) {
        throw "Downloaded archive did not contain jira.exe"
    }

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    Copy-Item -Path $Binary.FullName -Destination (Join-Path $BinDir "jira.exe") -Force

    if (-not $SkipPathUpdate) {
        $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
        $PathEntries = @()
        if ($UserPath) {
            $PathEntries = $UserPath.Split(';', [System.StringSplitOptions]::RemoveEmptyEntries)
        }

        if ($PathEntries -notcontains $BinDir) {
            $NewPath = ($PathEntries + $BinDir) -join ';'
            [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
            Write-Host "Added $BinDir to the user PATH."
        }

        $CurrentPathEntries = @()
        if ($env:Path) {
            $CurrentPathEntries = $env:Path.Split(';', [System.StringSplitOptions]::RemoveEmptyEntries)
        }

        if ($CurrentPathEntries -notcontains $BinDir) {
            $env:Path = (($CurrentPathEntries + $BinDir) -join ';')
        }
    }

    Write-Host "jira-cli $Version installed to $(Join-Path $BinDir 'jira.exe')"
    Write-Host "jira is available in this PowerShell session now."
}
finally {
    if (Test-Path $TempDir) {
        Remove-Item -LiteralPath $TempDir -Recurse -Force
    }
}
