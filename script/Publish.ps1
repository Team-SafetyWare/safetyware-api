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

# Todo: Do not delete and re-create the database user on every deploy. Part of SAF-41.
$atlas_project = Get-AtlasProject -Name "$app-$env_name"
$atlas_db_username = "app-api"
$atlas_db_password = New-RandomPassword -Length 32
if ( $null -ne (Get-AtlasDatabaseUser -ProjectId $atlas_project.id -Username $atlas_db_username) ) {
    Remove-AtlasDatabaseUser -ProjectId $atlas_project.id -Username $atlas_db_username
}
Add-AtlasDatabaseUser -ProjectId $atlas_project.id -Username $atlas_db_username -Password $atlas_db_password
New-AtlasCidrWhitelist "0.0.0.0/0" -ProjectId $atlas_project.id
$atlas_db_uri_no_cred = Get-AtlasDatabaseUri -Cluster "db" -ProjectId $atlas_project.id
$atlas_db_uri = ConvertTo-DatabaseUriWithCredentials `
    -SrvUri $atlas_db_uri_no_cred.standardSrv `
    -Username $atlas_db_username `
    -Password $atlas_db_password

Build-ApiFunc
Publish-Database -App $app -EnvName $env_name
Publish-AzureResourceGroup -Name $rg_name
Publish-AzureTemplate -ResourceGroup $rg_name -EnvHash $env_hash -DbUri $atlas_db_uri
Publish-ApiFunc -EnvHash $env_hash

$elapsed_time = $(get-date) - $start_time

Write-Host "Publish finished in $($elapsed_time.TotalSeconds) s." 

Pop-Location
