@description('A 13-character string unique to the organization, app, and environment.')
param envHash string

@description('Maximum number of workers functions can scale out to.')
param scaleLimit int = 1

var location = resourceGroup().location

module apiFunctionModule 'function.bicep' = {
  name: 'api-function'
  params: {
    envHash: envHash
    name: 'api'
    scaleLimit: scaleLimit
  }
}

resource keyVaultName 'Microsoft.KeyVault/vaults@2019-09-01' = {
  name: 'kv-${envHash}'
  location: location
  properties: {
    tenantId: subscription().tenantId
    accessPolicies: array({
      tenantId: apiFunctionModule.outputs.functionApp.identity.tenantId
      objectId: apiFunctionModule.outputs.functionApp.identity.principalId
      permissions: {
        secrets: [
          'get'
          'list'
        ]
      }
    })
    sku: {
      name: 'standard'
      family: 'A'
    }
    networkAcls: {
      defaultAction: 'Allow'
      bypass: 'AzureServices'
    }
  }
}
