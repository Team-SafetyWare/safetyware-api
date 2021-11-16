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