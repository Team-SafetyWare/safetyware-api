param()

Set-StrictMode -Version 3
$ErrorActionPreference = "Stop"
Push-Location $PSScriptRoot

Import-Module .\Util.psm1 -Force

Push-Location $(Get-ProjectLocation)

cargo build
Confirm-LastExitCode

Start-Container mongo

cargo test
Confirm-LastExitCode

$env:SW_DB_URI="mongodb://localhost:42781"
$env:RUST_LOG="info"
$env:RUST_BACKTRACE="1"
$server_job = Start-Job -ScriptBlock { cargo run }

docker run --rm --network host --volume $pwd/postman:/etc/newman -t postman/newman:5.3-alpine `
    run API.postman_collection.json --global-var "base=http://host.docker.internal:3001"

# Stop server before confirming Newman results.
Stop-Job $server_job.Id
Receive-Job -Keep $server_job.Id
Confirm-LastExitCode

Pop-Location

Pop-Location
