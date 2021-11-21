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

Build-ApiFunc
Publish-Database -App $app -EnvName $env_name

# Todo: Do not delete and re-create the database user on every deploy. Part of SAF-41.
$atlas_project = Get-AtlasProject -Name "$app-$env_name"
$db_username = "app-api"
$db_password = New-RandomPassword -Length 32
if ( $null -ne (Get-AtlasDatabaseUser -ProjectId $atlas_project.id -Username $db_username) ) {
    Remove-AtlasDatabaseUser -ProjectId $atlas_project.id -Username $db_username
}
New-AtlasDatabaseUser -ProjectId $atlas_project.id -Username $db_username -Password $db_password
New-AtlasCidrWhitelist "0.0.0.0/0" -ProjectId $atlas_project.id
$db_uri_no_cred = Get-AtlasDatabaseUri -Cluster "db" -ProjectId $atlas_project.id
$db_uri = ConvertTo-DatabaseUriWithCredentials `
    -SrvUri $db_uri_no_cred.standardSrv `
    -Username $db_username `
    -Password $db_password

New-AzureResourceGroup -Name $rg_name
Publish-AzureTemplate -ResourceGroup $rg_name -EnvHash $env_hash -DbUri $db_uri
Publish-DatabaseUri -DbUri $db_uri -ResourceGroup $rg_name -EnvHash $env_hash
Publish-ApiFunc -EnvHash $env_hash

$elapsed_time = $(get-date) - $start_time

Write-Host "Publish finished in $($elapsed_time.TotalSeconds) s." 

Pop-Location
