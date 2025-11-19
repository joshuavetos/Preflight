Param()
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")
Set-Location (Join-Path $RepoRoot "web")
npm install
npm run build
Set-Location $RepoRoot
cargo run --manifest-path (Join-Path $RepoRoot "core/Cargo.toml") -- dashboard
