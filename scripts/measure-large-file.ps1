[CmdletBinding()]
param(
    [int]$SizeGiB = 1,
    [int]$Port = 8765,
    [string]$FixtureDirectory = "$env:TEMP\zynero-d06-fixture"
)

$ErrorActionPreference = "Stop"
$sizeBytes = [int64]$SizeGiB * 1GB
New-Item -ItemType Directory -Force -Path $FixtureDirectory | Out-Null
$fixture = Join-Path $FixtureDirectory "zynero-$SizeGiB-gib.bin"

if (-not (Test-Path $fixture) -or ((Get-Item $fixture).Length -ne $sizeBytes)) {
    Write-Host "Creating $SizeGiB GiB fixture at $fixture"
    fsutil file createnew $fixture $sizeBytes | Out-Null
}

$hash = (Get-FileHash -Algorithm SHA256 -Path $fixture).Hash
$python = Get-Command python -ErrorAction SilentlyContinue
if (-not $python) {
    throw "Python is required to serve the local fixture. Install Python or start an equivalent local HTTP server manually."
}

$server = Start-Process -FilePath $python.Source `
    -ArgumentList @("-m", "http.server", "$Port", "--bind", "127.0.0.1") `
    -WorkingDirectory $FixtureDirectory `
    -PassThru

try {
    Write-Host "Fixture URL: http://127.0.0.1:$Port/$([IO.Path]::GetFileName($fixture))"
    Write-Host "Fixture bytes: $sizeBytes"
    Write-Host "Fixture SHA-256: $hash"
    Write-Host "HTTP server PID: $($server.Id)"
    Write-Host "Start ZYNERO and execute the one-connection and multi-connection runs now. Press Enter after both runs are complete."
    Read-Host | Out-Null
}
finally {
    if ($server -and -not $server.HasExited) {
        Stop-Process -Id $server.Id -Force
    }
}
