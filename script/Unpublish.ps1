param(
    [Parameter(Mandatory = $True)]
    [System.String]
    $Org,

    [Parameter(Mandatory = $True)]
    [System.String]
    $App,

    [Parameter(Mandatory = $True)]
    [System.String]
    $Env
)

Set-StrictMode -Version 3
$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot

Import-Module .\Util.psm1 -Force

$org = $Org
$app = $App
$env_name = $Env
$env_hash = Get-StringHash -Plain "$org-$app-$env_name" -Length 13

$rg_name = "rg-$app-$env_name"

$start_time = $(get-date)

$vault_name = "kv-$env_hash"
Remove-AzureResourceGroup $rg_name
Remove-AzureDeletedKeyVault $vault_name

$elapsed_time = $(get-date) - $start_time

Write-Host "Unpublish finished in $($elapsed_time.TotalSeconds) s." 

Pop-Location
