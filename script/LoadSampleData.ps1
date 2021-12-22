param(
    [Parameter(Mandatory = $False)]
    [System.String]
    $File='sample-data.gz'
)

Set-StrictMode -Version 3
$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot

Import-Module .\Util.psm1 -Force

Push-Location $(Get-ProjectLocation)

Start-Containers mongo

docker compose cp $File mongo:$File
Confirm-LastExitCode

docker compose exec mongo mongorestore --drop --gzip --archive=$File
Confirm-LastExitCode

Pop-Location

Pop-Location
