# SafetyWare API

This repository contains the API of the SafetyWare application.

## Quickstart

Follow these steps if you need to run the API without making code changes.

1. Install [Docker Desktop](https://www.docker.com/products/docker-desktop).
2. Run the API.
   ```
   docker compose up --build
   ```
3. Navigate to http://localhost:3001.
4. Load sample data as described later in this document (optional).

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
   $env:RUST_LOG="info"
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

## Postman

Postman can be used to interact with the API manually. Follow these steps to access the API from Postman.

### Load API definitions

1. Install and open [Postman](https://www.postman.com/downloads/).
2. Navigate to **File | Import | Folder**.
3. Import the "postman" folder in this repository.

### Send requests

1. Start the API locally.
2. In Postman, activate the "local" environment.
3. Send requests using definitions in the "API" collection.

### Save API definitions

1. Export collections and environments individually. Overwrite files in **/postman**.

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