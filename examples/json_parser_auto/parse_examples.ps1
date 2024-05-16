$ErrorCount = 0
$Config = "release"
$CargoConfig = if ($Config -eq "release") { "--release" } else { $null }

Write-Host "Building $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig --example json_parser_auto
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

Get-ChildItem ..\json\*.json | ForEach-Object {
    Write-Host "Parsing example $($_.FullName)..." -ForegroundColor Cyan
    cargo run $CargoConfig --example json_parser  $_.FullName
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
