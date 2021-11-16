Set-StrictMode -Version 3
$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot

Import-Module .\Util.psm1 -Force

$org = "cap"
$app = "sw"
$env_name = "dev"
$env_hash = Get-StringHash -Plain "$org-$app-$env_name" -Length 13

$rg_name = "rg-$app-$env_name"

Build-ApiFunc
Publish-Database -App $app -EnvName $env_name
Publish-AzureResourceGroup -Name $rg_name
Publish-AzureTemplate -ResourceGroup $rg_name -EnvHash $env_hash
Publish-ApiFunc -EnvHash $env_hash

Pop-Location
