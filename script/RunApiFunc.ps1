Set-StrictMode -Version 3
$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot

Import-Module .\Util.psm1 -Force

Build-ApiFunc
Start-DockerContainer
Invoke-ApiFunc
Stop-DockerContainer

Pop-Location
