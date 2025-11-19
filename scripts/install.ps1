Param()
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")
Set-Location $RepoRoot
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  Write-Error "cargo is required to install Preflight"
  exit 1
}
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
  Write-Error "npm is required to build the dashboard"
  exit 1
}
if (Test-Path (Join-Path $RepoRoot "Cargo.lock")) {
  cargo install --path core --locked
} else {
  cargo install --path core
}
Set-Location (Join-Path $RepoRoot "web")
npm install
npm run build
Set-Location $RepoRoot
Write-Host "Preflight installed. Run 'preflight scan' then 'preflight dashboard'."
