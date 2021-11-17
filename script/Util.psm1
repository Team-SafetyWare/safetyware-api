function Confirm-LastExitCode {
    [CmdletBinding()]
    Param(
    )

    Process {
        if ($LastExitCode -ne 0) {
            throw "Encountered non-zero exit code"
        }
    }
}

function Get-ProjectLocation {
    [CmdletBinding()]
    Param(
    )

    Process {
        return (Split-Path $PSScriptRoot)
    }
}

function Build-Api {
    [CmdletBinding()]
    Param(
    )

    Process {
        Push-Location $(Get-ProjectLocation)
        cargo build --package api --release
        Confirm-LastExitCode
        Pop-Location
    }
}

function Build-ApiFunc {
    [CmdletBinding()]
    Param(
    )

    Process {
        Write-Host "Building API Azure Function."

        Build-Api

        Push-Location $(Get-ProjectLocation)
        Copy-Item target\release\api.exe api\func\handler.exe
        Pop-Location
    }
}

function Invoke-ApiFunc {
    [CmdletBinding()]
    Param(
    )

    Process {
        Write-Host "Starting API Azure Function. Press Ctrl + C to stop."

        Push-Location "$(Get-ProjectLocation)\api\func"
        func start --port 3001
        Confirm-LastExitCode
        Pop-Location
    }
}

function Start-DockerContainer {
    [CmdletBinding()]
    Param(
    )

    Process {
        Push-Location $(Get-ProjectLocation)
        docker-compose up -d --build mongo
        Confirm-LastExitCode
        Pop-Location
    }
}

function Stop-DockerContainer {
    [CmdletBinding()]
    Param(
    )

    Process {
        Push-Location $(Get-ProjectLocation)
        docker-compose down
        Confirm-LastExitCode
        Pop-Location
    }
}

function New-AtlasProject {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    Process {
        Write-Host "Creating new Atlas project '$Name'."

        $project = mongocli iam project create $Name `
            --output json `
        | ConvertFrom-JSON
        Confirm-LastExitCode
        return $project
    }
}

function Get-AtlasProject {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    Process {
        $projects = mongocli iam project list `
            --output json `
        | ConvertFrom-JSON
        Confirm-LastExitCode

        $project = $projects.results `
        | Where-Object { $_.name -eq $Name } `
        | Select-Object -First 1
        Confirm-LastExitCode

        return $project
    }
}

function New-AtlasCluster {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name,
        [Parameter(Mandatory = $true)]
        [string] $ProjectId
    )

    Process {
        Write-Host "Creating new Atlas cluster."

        $cluster = mongocli atlas cluster create $Name `
            --output json `
            --projectId $ProjectId `
            --provider AZURE `
            --region CANADA_CENTRAL `
            --tier M0 `
        | ConvertFrom-JSON
        Confirm-LastExitCode
        return $cluster
    }
}

function Watch-AtlasCluster {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name,
        [Parameter(Mandatory = $true)]
        [string] $ProjectId
    )

    Process {
        Write-Host "Waiting for new Atlas cluster to be ready."

        mongocli atlas clusters watch $Name `
            --projectId $ProjectId
        Confirm-LastExitCode

        return $cluster
    }
}

function Get-AtlasCluster {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name,
        [Parameter(Mandatory = $true)]
        [string] $ProjectId
    )

    Process {
        $clusters = mongocli atlas cluster list `
            --output json `
            --projectId $ProjectId `
        | ConvertFrom-JSON
        Confirm-LastExitCode

        $cluster = $clusters.results `
        | Where-Object { $_.name -eq $Name } `
        | Select-Object -First 1
        Confirm-LastExitCode

        return $cluster
    }
}

function Publish-Database {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $App,
        [Parameter(Mandatory = $true)]
        [string] $EnvName
    )

    Process {
        $project_name = "$App-$EnvName"
        $cluster_name = "db"

        $project = (Get-AtlasProject $project_name) `
            ?? (New-AtlasProject $project_name)

        $cluster = (Get-AtlasCluster $cluster_name -ProjectId $project.id) `
            ?? (New-AtlasCluster $cluster_name -ProjectId $project.id)

        Watch-AtlasCluster $cluster_name -ProjectId $project.id

        return $cluster
    }
}

function Get-StringHash {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Plain,
        [Parameter(Mandatory = $true)]
        [int] $Length
    )

    Process {
        $hash_array = (new-object System.Security.Cryptography.SHA512Managed).ComputeHash($Plain.ToCharArray())
        -Join ($hash_array[1..$Length] | ForEach-Object { [char]($_ % 26 + [byte][char]'a') })
    }
}

function Publish-AzureResourceGroup {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    Process {
        Write-Host "Publishing Azure resource group '$Name'."

        $rg = az group create `
            --name $Name `
            --location canadacentral `
        | ConvertFrom-JSON
        Confirm-LastExitCode
        return $rg
    }
}

function Publish-AzureTemplate {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $ResourceGroup,
        [Parameter(Mandatory = $true)]
        [string] $EnvHash,
        # Todo: Remove this as part of SAF-41.
        [Parameter(Mandatory = $true)]
        [string] $DbUri
    )

    Process {
        Write-Host "Publishing Azure template."

        Push-Location "$(Get-ProjectLocation)\infrastructure"

        $deploy = az deployment group create `
            --name deploy `
            --resource-group $ResourceGroup `
            --template-file azuredeploy.bicep `
            --parameters envHash="$EnvHash" dbUri="$DbUri" `
        | ConvertFrom-JSON
        Confirm-LastExitCode

        Pop-Location
        return $deploy
    }
}

function Publish-ApiFunc {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $EnvHash
    )

    Process {
        Write-Host "Publishing API Azure Function."

        Push-Location "$(Get-ProjectLocation)\api\func"

        func azure functionapp publish "func-api-$EnvHash"
        Confirm-LastExitCode

        Pop-Location
        return $func
    }
}
