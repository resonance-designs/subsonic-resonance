[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot

function Exit-UnableToContinue {
    Write-Host
    Write-Host 'Unable to continue with application build and run. Press any key to return to terminal prompt.'
    [void][Console]::ReadKey($true)
    exit 0
}

function Confirm-StopProcesses {
    param(
        [Parameter(Mandatory)]
        [int[]]$ProcessIds,

        [Parameter(Mandatory)]
        [string]$Reason
    )

    $processSummary = $ProcessIds -join ', '
    Write-Warning "$Reason Process ID(s): $processSummary."
    $confirmation = Read-Host 'Do you want to stop these processes and continue? (y/N)'
    if ($confirmation -notmatch '^[Yy]$') {
        return $false
    }

    Stop-Process -Id $ProcessIds -ErrorAction Stop
    Start-Sleep -Milliseconds 500
    return $true
}

function Confirm-PortAvailable {
    param(
        [Parameter(Mandatory)]
        [int]$Port,

        [Parameter(Mandatory)]
        [string]$Service
    )

    $listeners = @(Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue)
    if ($listeners.Count -eq 0) {
        return $true
    }

    $processIds = @($listeners | Select-Object -ExpandProperty OwningProcess -Unique)
    if (-not (Confirm-StopProcesses -ProcessIds $processIds -Reason "Cannot start $Service because port $Port is already in use.")) {
        return $false
    }

    $remainingListeners = @(Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue)
    if ($remainingListeners.Count -gt 0) {
        Write-Warning "Port $Port is still in use after attempting to stop its processes."
        return $false
    }

    return $true
}

function Confirm-BuildArtifactAvailable {
    $serverExecutable = Join-Path $repositoryRoot 'target\debug\subsonic-resonance-server.exe'
    $lockingProcesses = @(Get-Process -Name 'subsonic-resonance-server' -ErrorAction SilentlyContinue | Where-Object {
        $_.Path -and [System.IO.Path]::GetFullPath($_.Path) -eq [System.IO.Path]::GetFullPath($serverExecutable)
    })

    if ($lockingProcesses.Count -eq 0) {
        return $true
    }

    $processIds = @($lockingProcesses | Select-Object -ExpandProperty Id -Unique)
    return (Confirm-StopProcesses -ProcessIds $processIds -Reason 'Cannot build Resonance because the existing server executable is still running.')
}

if (-not (Confirm-BuildArtifactAvailable)) {
    Exit-UnableToContinue
}
if (-not (Confirm-PortAvailable -Port 3000 -Service 'the Subsonic Resonance API server')) {
    Exit-UnableToContinue
}

Write-Host -NoNewline 'Building Subsonic Resonance application in '
foreach ($count in 3..1) {
    Write-Host -NoNewline "$count..."
    if ($count -gt 1) {
        Write-Host -NoNewline ' '
    }
    Start-Sleep -Seconds 1
}
Write-Host
Write-Host

Push-Location $repositoryRoot
try {
    & cargo build --workspace
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo build failed with exit code $LASTEXITCODE."
    }

    Write-Host
    Write-Host 'Do you want to:'
    Write-Host
    Write-Host '1. Start the server and run the app'
    Write-Host '2. Just start the server'
    Write-Host

    do {
        $selection = Read-Host 'Enter 1 or 2'
        if ($selection -notin @('1', '2')) {
            Write-Warning 'Please enter 1 or 2.'
        }
    } while ($selection -notin @('1', '2'))

    Write-Host
    if ($selection -eq '1') {
        if (-not (Confirm-PortAvailable -Port 3000 -Service 'the Subsonic Resonance API server')) {
            Exit-UnableToContinue
        }
        if (-not (Confirm-PortAvailable -Port 8088 -Service 'the Subsonic Resonance browser UI')) {
            Exit-UnableToContinue
        }
        Write-Host 'Starting the API server and browser application. Press Ctrl+C to stop.'
        & npm run app:start
    }
    else {
        if (-not (Confirm-PortAvailable -Port 3000 -Service 'the Subsonic Resonance API server')) {
            Exit-UnableToContinue
        }
        Write-Host 'Starting the API server. Press Ctrl+C to stop.'
        & cargo run -p subsonic-resonance-server
    }

    if ($LASTEXITCODE -ne 0) {
        throw "The selected startup command exited with code $LASTEXITCODE."
    }
}
finally {
    Pop-Location
}
