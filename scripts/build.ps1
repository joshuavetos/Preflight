Param()
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")
Set-Location $RepoRoot
cargo build --workspace
Set-Location (Join-Path $RepoRoot "web")
npm install
npm run build
