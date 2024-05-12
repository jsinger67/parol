$ErrorCount = 0
$Config = "release"
$CargoConfig = if ($Config -eq "release") { "--release" } else { $null }

Write-Host "Building $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig --example oberon2
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

Write-Host "Building examples in $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running oberon2 on some example files..." -ForegroundColor Cyan
Get-ChildItem ./Oberon2Source/*.mod |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    cargo run $CargoConfig --example oberon2 $_.FullName -q
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCount    
    }
}

# --------------------------------------------------------------------------------------------------
# Final message
# --------------------------------------------------------------------------------------------------
if ($ErrorCount -gt 0) {
    $Msg = "$ErrorCount error(s) occurred."
    Write-Host -Object $Msg  -ForegroundColor Red
} else {
    Write-Host "All examples successfully parsed." -ForegroundColor Green
}
