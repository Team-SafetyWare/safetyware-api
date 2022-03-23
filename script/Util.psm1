[System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSUseShouldProcessForStateChangingFunctions', '')]
param()

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
        Write-Output "Building API Azure Function."

        Build-Api

        Push-Location $(Get-ProjectLocation)
        Copy-Item target\release\api.exe api\func\handler.exe
        Copy-Item doc api\func -Recurse -Force
        Pop-Location
    }
}

function Invoke-ApiFunc {
    [CmdletBinding()]
    Param(
    )

    Process {
        Write-Output "Starting API Azure Function. Press Ctrl + C to stop."

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
        Write-Output "Creating new Atlas project '$Name'."

        $project = mongocli iam project create $Name `
            --output json `
        | ConvertFrom-JSON
        Confirm-LastExitCode
        return $project
    }
}

function Get-AtlasProject {
    [CmdletBinding()]
    [System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSReviewUnusedParameter', 'Name')]
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
        Write-Output "Removing Atlas project '$ProjectId'."

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
        Write-Output "Creating new Atlas cluster."

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
        Write-Output "Waiting for new Atlas cluster to be ready."

        mongocli atlas clusters watch $Name `
            --projectId $ProjectId
        Confirm-LastExitCode

        return $cluster
    }
}

function Get-AtlasCluster {
    [CmdletBinding()]
    [System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSReviewUnusedParameter', 'Name')]
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
        Write-Output "Removing Atlas database '$Name' in project '$ProjectId'."

        mongocli atlas cluster delete $Name `
            --force `
            --projectId $ProjectId
        Confirm-LastExitCode
    }
}

function New-AtlasDatabaseUser {
    [CmdletBinding()]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingPlainTextForPassword', '')]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingUsernameAndPasswordParams', '')]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $ProjectId,
        [Parameter(Mandatory = $true)]
        [string] $Username,
        [Parameter(Mandatory = $true)]
        [string] $Password
    )

    Process {
        Write-Output "Creating new Atlas database user '$Username'."

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
    [System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSReviewUnusedParameter', 'Username')]
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
        Write-Output "Removing Atlas database user '$Username'."

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
        Write-Output "Whitelisting CIDR block '$CidrBlock'."

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

function ConvertTo-DatabaseUriWithCredential {
    [CmdletBinding()]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingPlainTextForPassword', '')]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingUsernameAndPasswordParams', '')]
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
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [int] $Length
    )

    Process {
        return -join (((48..57)+(65..90)+(97..122)) * 80 |Get-Random -Count $Length |ForEach-Object{[char]$_})
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

        $atlas_project = Get-AtlasProject -Name $project_name
        New-AtlasCidrWhitelist "0.0.0.0/0" -ProjectId $atlas_project.id

        return $cluster
    }
}

function Remove-Database {
    [CmdletBinding()]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingEmptyCatchBlock', '')]
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

        if ($null -ne $project) {
            $cluster = Get-AtlasCluster $cluster_name -ProjectId $project.id

            if ($null -ne $cluster) {
                Remove-AtlasCluster $cluster_name -ProjectId $project.id
                try {
                    # The watch will throw an error once the cluster is deleted.
                    Watch-AtlasCluster $cluster_name -ProjectId $project.id
                }
                catch {}
            }

            Remove-AtlasProject -ProjectId $project.id
        }
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
        Write-Output "Publishing Azure resource group '$Name'."

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
        $exists = az group exists `
            --name $Name
        Confirm-LastExitCode

        if ([boolean]::Parse($exists)) {
            Write-Output "Removing Azure resource group '$Name'."

            az group delete `
                --name $Name `
                --yes
            Confirm-LastExitCode
        }
    }
}

function Get-AzureDeletedKeyVault {
    [CmdletBinding()]
    [System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSReviewUnusedParameter', 'Name')]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    Process {
        $vaults = az keyvault list-deleted `
        | ConvertFrom-Json
        Confirm-LastExitCode

        $vault = $vaults `
        | Where-Object { $_.name -eq $Name } `
        | Select-Object -First 1

        return $vault
    }
}

function Remove-AzureDeletedKeyVault {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name
    )

    Process {
        if ($null -ne (Get-AzureDeletedKeyVault $Name)) {
            Write-Output "Purging Azure KeyVault '$Name'."

            az keyvault purge `
                --name $Name
            Confirm-LastExitCode
        }
    }
}

function Publish-AzureTemplate {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $ResourceGroup,
        [Parameter(Mandatory = $true)]
        [string] $EnvHash
    )

    Process {
        Write-Output "Publishing Azure template."

        Push-Location "$(Get-ProjectLocation)\infrastructure"

        $deploy = az deployment group create `
            --name deploy `
            --resource-group $ResourceGroup `
            --template-file azuredeploy.bicep `
            --parameters envHash="$EnvHash" `
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
        Write-Output "Publishing API Azure Function."

        Push-Location "$(Get-ProjectLocation)\api\func"

        func azure functionapp publish "func-api-$EnvHash"
        Confirm-LastExitCode

        Pop-Location
        return $func
    }
}

function Get-AzureUserInfo {
    [CmdletBinding()]
    Param(
    )

    Process {
        $user_info = az ad signed-in-user show `
        | ConvertFrom-Json
        Confirm-LastExitCode
        return $user_info
    }
}

function Update-AzureAppConfig {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $AppName,
        [Parameter(Mandatory = $true)]
        [string] $ResourceGroup
    )

    Process {
        az webapp config appsettings delete `
            --name $AppName `
            --resource-group $ResourceGroup `
            --setting-names nonexistant`
        | Out-Null
        Confirm-LastExitCode
    }
}

function Set-AzureKeyVaultPolicy {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $VaultName,
        [Parameter(Mandatory = $true)]
        [string] $ResourceGroup
    )

    Process {
        $user_info = Get-AzureUserInfo

        az keyvault set-policy `
            --name $VaultName `
            --object-id $user_info.objectId `
            --resource-group $ResourceGroup `
            --secret-permissions all `
        | Out-Null
        Confirm-LastExitCode
    }
}

function Set-AzureKeyVaultSecret {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name,
        [Parameter(Mandatory = $true)]
        [string] $Value,
        [Parameter(Mandatory = $true)]
        [string] $VaultName,
        [Parameter(Mandatory = $true)]
        [string] $ResourceGroup
    )

    Process {
        Set-AzureKeyVaultPolicy -VaultName $VaultName -ResourceGroup $ResourceGroup

        az keyvault secret set `
            --name $Name `
            --vault-name $VaultName `
            --value $Value `
        | Out-Null
        Confirm-LastExitCode
    }
}

function Test-AzureKeyVaultSecretDefinitelyNotExist {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $Name,
        [Parameter(Mandatory = $true)]
        [string] $VaultName
    )

    Process {
        Set-AzureKeyVaultPolicy -VaultName $VaultName -ResourceGroup $ResourceGroup

        & az keyvault secret show `
            --name $Name `
            --vault-name $VaultName `
            2>&1 `
            | Tee-Object -Variable showOutput | Out-Null

        $error_record = $showOutput[0]
        $definitely_not_exist = $error_record.ToString().StartsWith("ERROR: (SecretNotFound)")

        if (-Not $definitely_not_exist) {
            Write-Error $error_record
            Confirm-LastExitCode
        }

        return $definitely_not_exist
    }
}

function Publish-DatabaseUri {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $ResourceGroup,
        [Parameter(Mandatory = $true)]
        [string] $App,
        [Parameter(Mandatory = $true)]
        [string] $EnvName,
        [Parameter(Mandatory = $true)]
        [string] $EnvHash
    )

    Process {
        $project_name = "$App-$EnvName"
        $cluster_name = "db"
        $atlas_project = Get-AtlasProject -Name $project_name
        $db_username = "app-api"

        if ( $null -ne (Get-AtlasDatabaseUser -ProjectId $atlas_project.id -Username $db_username) ) {
            return;
        }

        Write-Output "Publishing database URI."

        $db_password = New-RandomPassword -Length 32

        New-AtlasDatabaseUser -ProjectId $atlas_project.id -Username $db_username -Password $db_password
        $db_uri_no_cred = Get-AtlasDatabaseUri -Cluster $cluster_name -ProjectId $atlas_project.id
        $db_uri = ConvertTo-DatabaseUriWithCredential `
            -SrvUri $db_uri_no_cred.standardSrv `
            -Username $db_username `
            -Password $db_password

        $vault_name = "kv-$EnvHash"
        Set-AzureKeyVaultSecret `
            -Name "db-uri" `
            -Value $db_uri `
            -VaultName $vault_name `
            -ResourceGroup $ResourceGroup `
        | Out-Null

        $func_name = "func-api-$EnvHash"
        Update-AzureAppConfig `
            -AppName $func_name `
            -ResourceGroup $ResourceGroup `
        | Out-Null
    }
}

function Publish-PrivateKey {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string] $ResourceGroup,
        [Parameter(Mandatory = $true)]
        [string] $EnvHash
    )

    Process {
        $vault_name = "kv-$EnvHash"
        $secret_name = "private-key"

        $definitely_not_exist = Test-AzureKeyVaultSecretDefinitelyNotExist `
            -VaultName $vault_name `
            -Name $secret_name

        if (-Not $definitely_not_exist) {
            return;
        }

        Write-Output "Publishing private key."

        $private_key = New-RandomPassword -Length 128
        Set-AzureKeyVaultSecret `
            -Name $secret_name `
            -Value $private_key `
            -VaultName $vault_name `
            -ResourceGroup $ResourceGroup `
        | Out-Null

        $func_name = "func-api-$EnvHash"
        Update-AzureAppConfig `
            -AppName $func_name `
            -ResourceGroup $ResourceGroup `
        | Out-Null
    }
}

function Start-Container {
    [CmdletBinding()]
    Param(
        [Parameter(Mandatory = $true)]
        [string[]] $Containers
    )

    Process {
        Write-Output "Starting containers: $Containers"

        docker compose up -d $Containers
        Confirm-LastExitCode
    }
}
