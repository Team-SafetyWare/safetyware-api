# SafetyWare API

This repository contains the API of the SafetyWare application.

## Quickstart

Follow these steps if you need to run the API without making code changes.

1. Install [Docker Desktop](https://www.docker.com/products/docker-desktop).
2. Run the API.
   ```
   docker-compose up --build
   ```
3. Navigate to http://localhost:3001.

## Development

Follow this section if you need to make code changes.

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
   docker-compose up -d --build mongo
   ```
2. Set environmental variables (assumes PowerShell).
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
   docker-compose down
   ```
