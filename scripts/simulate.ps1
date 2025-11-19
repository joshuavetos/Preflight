Param(
  [Parameter(Mandatory=$true, Position=0)]
  [string]$Command
)
preflight simulate "$Command"
