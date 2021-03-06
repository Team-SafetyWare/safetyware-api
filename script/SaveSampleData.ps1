param(
    [Parameter(Mandatory = $True)]
    [System.String]
    $File
)

Set-StrictMode -Version 3
$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot

Import-Module .\Util.psm1 -Force

Push-Location $(Get-ProjectLocation)

Start-Container mongo

docker compose exec mongo mongodump --db sw --gzip --archive=$File
Confirm-LastExitCode

docker compose cp mongo:$File $File
Confirm-LastExitCode

Pop-Location

Pop-Location
