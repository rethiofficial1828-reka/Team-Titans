# sign.ps1 — Code signing script for FortiChain binaries
# Usage: .\sign.ps1 -CertThumbprint <thumbprint> [-TimestampServer <url>]
#
# Signs all FortiChain executables with an Authenticode certificate.
# Called by CI/CD pipeline after the build step.

param(
    [Parameter(Mandatory=$true)]
    [string]$CertThumbprint,

    [string]$TimestampServer = "http://timestamp.digicert.com"
)

$ErrorActionPreference = 'Stop'

$binaries = @(
    "..\..\target\release\FortiChain.exe",
    "..\..\target\release\FortiChainSvc.exe",
    "..\..\target\release\FortiChainGate.exe"
)

foreach ($bin in $binaries) {
    $fullPath = Resolve-Path $bin -ErrorAction SilentlyContinue
    if (-not $fullPath) {
        Write-Warning "Binary not found: $bin (skipping)"
        continue
    }
    Write-Host "Signing $fullPath ..."
    & signtool.exe sign /sha1 $CertThumbprint /fd SHA256 /tr $TimestampServer /td SHA256 $fullPath
    if ($LASTEXITCODE -ne 0) {
        throw "signtool failed for $fullPath"
    }
    Write-Host "  Signed successfully."
}

Write-Host "All binaries signed."
