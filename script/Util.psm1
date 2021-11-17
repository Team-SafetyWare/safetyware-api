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

function Remove-AtlasProject {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $ProjectId
    )

    Process {
        mongocli iam project delete $ProjectId `
            --force
        Confirm-LastExitCode
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

function Remove-AtlasCluster {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name,
        [Parameter(Mandatory = $true)]
        [string] $ProjectId
    )

    Process {
        Write-Host "Removing Atlas database '$Name' in project '$ProjectId'."

        mongocli atlas cluster delete $Name `
            --force `
            --projectId $ProjectId
        Confirm-LastExitCode
    }
}

function New-AtlasDatabaseUser {
    [CmdletBinding()]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingPlainTextForPassword', '', Scope='Function')]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $ProjectId,
        [Parameter(Mandatory = $true)]
        [string] $Username,
        [Parameter(Mandatory = $true)]
        [string] $Password
    )

    Process {
        Write-Host "Creating new Atlas database user '$Username'."

        $user = mongocli atlas dbuser create `
            --output json `
            --password $Password `
            --projectId $ProjectId `
            --role readWriteAnyDatabase `
            --username $Username `
        | ConvertFrom-JSON
        Confirm-LastExitCode
        return $user
    }
}

function Get-AtlasDatabaseUser {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $ProjectId,
        [Parameter(Mandatory = $true)]
        [string] $Username
    )

    Process {
        $users = mongocli atlas dbuser list `
            --output json `
            --projectId $ProjectId `
        | ConvertFrom-JSON
        Confirm-LastExitCode

        $user = $users `
        | Where-Object { $_.username -eq $Username } `
        | Select-Object -First 1
        Confirm-LastExitCode

        return $user
    }
}

function Remove-AtlasDatabaseUser {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $ProjectId,
        [Parameter(Mandatory = $true)]
        [string] $Username
    )

    Process {
        Write-Host "Removing Atlas database user '$Username'."

        mongocli atlas dbuser delete $Username `
            --force `
            --projectId $ProjectId
        Confirm-LastExitCode
    }
}

function New-AtlasCidrWhitelist {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $CidrBlock,
        [Parameter(Mandatory = $true)]
        [string] $ProjectId
    )

    Process {
        Write-Host "Whitelisting CIDR block '$CidrBlock'."

        $rule = mongocli atlas whitelist create "$CidrBlock" `
            --output json `
            --projectId $ProjectId `
            --type cidrBlock `
        | ConvertFrom-JSON
        Confirm-LastExitCode
        return $rule
    }
}

function Get-AtlasDatabaseUri {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Cluster,
        [Parameter(Mandatory = $true)]
        [string] $ProjectId
    )

    Process {
        $uri = mongocli atlas cluster connectionString describe $Cluster `
            --output json `
            --projectId $ProjectId `
        | ConvertFrom-JSON
        Confirm-LastExitCode
        return $uri
    }
}

function ConvertTo-DatabaseUriWithCredentials {
    [CmdletBinding()]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingPlainTextForPassword', '', Scope='Function')]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $SrvUri,
        [Parameter(Mandatory = $true)]
        [string] $Username,
        [Parameter(Mandatory = $true)]
        [string] $Password
    )

    Process {
        return $SrvUri.Insert(14, "${Username}:$([uri]::EscapeDataString($Password))@")
    }
}

function New-RandomPassword {
    param(
        [int]
        $Length
    )

    $symbols = '!@#$%^&*'.ToCharArray()
    $character_list = 'a'..'z' + 'A'..'Z' + '0'..'9' + $symbols

    do {
        $password = -join (0..$Length | ForEach-Object { $character_list | Get-Random })
        [int]$has_lower_char = $password -cmatch '[a-z]'
        [int]$has_upper_char = $password -cmatch '[A-Z]'
        [int]$has_digit = $password -match '[0-9]'
        [int]$has_symbol = $password.IndexOfAny($symbols) -ne -1

    }
    until (($has_lower_char + $has_upper_char + $has_digit + $has_symbol) -ge 3)

    $password
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

function Remove-Database {
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

        $project = Get-AtlasProject $project_name

        Remove-AtlasCluster $cluster_name -ProjectId $project.id
        try {
            # The watch will throw an error once the cluster is deleted.
            Watch-AtlasCluster $cluster_name -ProjectId $project.id   
        }
        catch {}
        Remove-AtlasProject -ProjectId $project.id
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

function New-AzureResourceGroup {
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

function Remove-AzureResourceGroup {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    Process {
        Write-Host "Removing Azure resource group '$Name'."

        az group delete `
            --name $Name `
            --yes 
        Confirm-LastExitCode
    }
}

function Remove-AzureDeletedKeyVault {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    Process {
        Write-Host "Purging Azure KeyVault '$Name'."

        az keyvault purge `
            --name $Name
        Confirm-LastExitCode
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
