Set-StrictMode -Version 3
$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot

Import-Module .\Util.psm1 -Force

$project_name = "sw-dev"
$cluster_name = "db"

$project = (Get-AtlasProject $project_name) `
    ?? (New-AtlasProject $project_name)

$cluster = (Get-AtlasCluster $cluster_name -ProjectId $project.id) `
    ?? (New-AtlasCluster $cluster_name -ProjectId $project.id)

$cluster

Pop-Location
