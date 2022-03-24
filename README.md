# SafetyWare API

This repository contains the API of the SafetyWare application.

## Quickstart

Follow these steps if you need to run the API without making code changes.

1. Install [Docker Desktop](https://www.docker.com/products/docker-desktop).
2. Run the API.
   ```
   docker compose up --build
   ```
3. Navigate to http://localhost:3001/playground.

## Execute a GraphQL query

The API exposes all operations through [GraphQL](https://graphql.org/). A web interface is included for testing queries.

1. Start the API as described in the 'Quickstart' section.
2. Load sample data as described later in this document (optional).
3. Navigate to http://localhost:3001/playground.
4. Execute the following example query.
   ```
   {
     companies {
       id
       name
       people {
         id
         name
         locationReadings {
           timestamp
           coordinates
         }
       }
     }
   }
   ```

## Develop

This section describes how to make code changes.

### Install tools

1. Install [Docker Desktop](https://www.docker.com/products/docker-desktop).
2. Install [Rust Programming Language](https://www.rust-lang.org/).

### Configure CLion (optional)

[CLion](https://www.jetbrains.com/clion/) is the recommended IDE. CLI commands are also supported.

1. Open this repository in CLion.
2. Install required plugins when prompted.
3. Confirm the "Start containers" configuration is able to run.
4. Configure CLion to use Rustfmt.
    1. Press **Ctrl + Shift + A** to search actions.
    2. Search "Rustfmt" and open the first result.
    3. Enable "Use Rustfmt instead of built-in formatter" and "Run rustfmt on Save".

### Run in CLion

1. Execute the "Run" configuration. Docker containers are started automatically.
2. Navigate to http://localhost:3001.
3. Stop the Docker containers in the Services tab before leaving.

### Run with CLI

1. Start the Docker containers once per sitting.
   ```
   docker compose up -d --build mongo
   ```
2. Set environmental variables.
   ```
   $env:SW_DB_URI="mongodb://localhost:42781"
   $env:SW_PRIVATE_KEY="secret"
   $env:RUST_LOG="info"
   $env:RUST_BACKTRACE="1"
   ```
3. Build and run.
   ```
   cargo run -p api
   ```
4. Navigate to http://localhost:3001.
5. Stop the Docker containers before leaving.
   ```
   docker compose down
   ```

## Test

1. Run the tests.
   ```
   .\script\Test.ps1
   ```

## Sample data

This section describes how to load and save sample
data. [PowerShell Core](https://docs.microsoft.com/en-us/powershell/scripting/install/installing-powershell) is
required.

### Load sample data

1. Load sample data. Your existing database contents will be dropped.
   ```
   .\script\LoadSampleData.ps1 sample-data.gz
   ```

### Save sample data

1. Save database contents as sample data.
   ```
   .\script\SaveSampleData.ps1 sample-data.gz
   ```

## Deploy

This section describes how to deploy the API to the cloud.

### Install tools

1. Install [Azure CLI](https://docs.microsoft.com/en-us/cli/azure/install-azure-cli).
2. Install [Azure Functions Core Tools](https://docs.microsoft.com/en-us/azure/azure-functions/functions-run-local).
3. Install [Docker Desktop](https://www.docker.com/products/docker-desktop).
6. Install [MongoDB CLI for Cloud](https://www.mongodb.com/try/download/mongocli).
5. Install [PowerShell Core](https://docs.microsoft.com/en-us/powershell/scripting/install/installing-powershell) (not
   preinstalled PowerShell).
6. Install [Rust Programming Language](https://www.rust-lang.org/).

### Sign up for cloud providers

1. Create a [Microsoft Azure](https://azure.microsoft.com/en-ca/free/) account.
2. Create a [MongoDB Atlas](https://www.mongodb.com/cloud/atlas/register) account.

### Configure CLI tools

1. Connect the Azure CLI to your Azure account.
   ```
   az login
   ```
2. Connect the MongoDB Cloud CLI to your MongoDB Atlas account.
   ```
   mongocli config
   ```

### Publish the API

1. Publish the API to the specified environment. It will be publicly accessible.
   ```
   .\script\Publish.ps1 -Org cap -App sw -Env dev
   ```
    - You can pick something other than `dev` as the environment name, such as your name, to avoid name conflicts.

### Unpublish the API

1. Unpublish the API and delete all cloud resources.
   ```
   .\script\Unpublish.ps1 -Org cap -App sw -Env dev
   ```
   
## Generate GraphQL documentation
[SpectaQL](https://github.com/anvilco/spectaql) is used to automatically generate GraphQL documentation. The 
generated documentation is saved in the repository. It should not be edited manually.

1. Install [NodeJS](https://nodejs.org/en/download/).
2. Install [SpectaQL](https://github.com/anvilco/spectaql).
   ```
   npm install -g spectaql@0.12
   ```
3. Start the API locally.
4. Navigate to the doc directory.
   ```
   cd doc
   ```
5. Generate documentation.
   ```
   npx spectaql config.yml
   ```
6. Navigate to `/doc/` to see documentation.
