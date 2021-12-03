$ErrorCont = 0

Write-Host "Building release. Please wait..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

Get-ChildItem .\json\*.json | ForEach-Object {
    Write-Host "Parsing example $($_.FullName)..." -ForegroundColor Cyan
    ./target/release/json_parser $_.FullName
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCont    
    }
}

# --------------------------------------------------------------------------------------------------
# Final message
# --------------------------------------------------------------------------------------------------
if ($ErrorCont -gt 0) {
    $Msg = "$ErrorCount error(s) occurred."
    Write-Host -Object $Msg  -ForegroundColor Red
} else {
    Write-Host "All examples successfully parsed." -ForegroundColor Green
}
