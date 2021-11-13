# SW-BackEnd

This repository contains the backend for the SafetyWare application. [CLion](https://www.jetbrains.com/clion/) is
recommended for development. CLI commands are also supported.

## Prerequisites

### Install tools

1. Install [Rust Programming Language](https://www.rust-lang.org/).

### Install CLion (optional)

1. Install [CLion](https://www.jetbrains.com/clion/).
2. Open this repository in CLion.
3. Install required plugins when prompted.
4. Configure CLion to use Rustfmt.
    1. Press **Ctrl + Shift + A** to search for actions.
    2. Search "Rustfmt" and open the first result.
    3. Enable "Use Rustfmt instead of built-in formatter" and "Run rustfmt on Save".

## Run locally

### CLI

1. Build and run the API server.
   ```
   cargo run -p api
   ```
2. Navigate to http://localhost:3001/hello/alice.

### CLion

1. Execute the "Run api" configuration.
2. Navigate to http://localhost:3001/hello/alice.
