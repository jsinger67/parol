$ErrorCount = 0
$Config = "release"
$CargoConfig = if ($Config -eq "release") { "--release" } else { $null }

Write-Host "Building $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

$target = "./../../target/$Config/json_parser_auto"

Get-ChildItem .\json\*.json | ForEach-Object {
    Write-Host "Parsing example $($_.FullName)..." -ForegroundColor Cyan
    &$target  $_.FullName
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
