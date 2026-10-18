Write-Host "=== Playground Tooling Health-Check ===" -ForegroundColor Cyan

$required = @(
    @{ Name = "docker"; Winget = "Docker.DockerDesktop"; Required = $true },
    @{ Name = "kind"; Winget = "Kubernetes.kind"; Required = $true },
    @{ Name = "kubectl"; Winget = "Kubernetes.kubectl"; Required = $true },
    @{ Name = "terraform"; Winget = "Hashicorp.Terraform"; Required = $true },
    @{ Name = "flux"; Winget = "FluxCD.Flux"; Required = $true },
    @{ Name = "task"; Winget = "Task.Task"; Required = $true },
    @{ Name = "dotnet"; Winget = "Microsoft.DotNet.SDK.9"; Required = $false },
    @{ Name = "cargo"; Winget = "Rustlang.Rustup"; Required = $false }
)

$missing = @()

foreach ($tool in $required) {
    $cmd = Get-Command $tool.Name -ErrorAction SilentlyContinue
    if ($cmd) {
        Write-Host "  [OK] $($tool.Name.PadRight(12)) -> $($cmd.Source)" -ForegroundColor Green
    } else {
        $status = if ($tool.Required) { "[MISSING - REQUIRED]" } else { "[MISSING - OPTIONAL]" }
        Write-Host "  $status $($tool.Name.PadRight(12)) -> Install via: winget install $($tool.Winget)" -ForegroundColor Yellow
        if ($tool.Required) { $missing += $tool }
    }
}

if ($missing.Count -gt 0) {
    Write-Host "`nInstall the missing required tools above to proceed with full cluster automation." -ForegroundColor Red
    exit 1
} else {
    Write-Host "`nAll core tools are available! You are ready to provision." -ForegroundColor Green
    exit 0
}
