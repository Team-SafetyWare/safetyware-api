# SafetyWare API

This repository contains the API of the SafetyWare application. [CLion](https://www.jetbrains.com/clion/) is recommended
for development. CLI commands are also supported.

## Prerequisites

### Install tools

1. Install [Docker Desktop](https://www.docker.com/products/docker-desktop).
2. Install [Rust Programming Language](https://www.rust-lang.org/).

### Install CLion (optional)

1. Install [CLion](https://www.jetbrains.com/clion/).
2. Open this repository in CLion.
3. Install required plugins when prompted.
4. Configure CLion to use Rustfmt.
    1. Press **Ctrl + Shift + A** to search for actions.
    2. Search "Rustfmt" and open the first result.
    3. Enable "Use Rustfmt instead of built-in formatter" and "Run rustfmt on Save".

## Run locally

### CLion (recommended)

1. Execute the "Run" configuration. Docker containers are started automatically.
2. Navigate to http://localhost:3001/hello/alice.
3. Stop the Docker containers in the Services tab when your sitting is over.

### CLI

1. Start the Docker containers once per sitting.
   ```
   docker-compose up -d
   ```
2. Build and run the API server.
   ```
   cargo run -p api
   ```
3. Navigate to http://localhost:3001.
4. Stop the Docker containers when your sitting is over.
   ```
   docker-compose down
   ```
