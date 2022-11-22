$ErrorCount = 0
$Config = "release"
$CargoConfig = if ($Config -eq "release") { "--release" } else { "" }

Write-Host "Building $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

Write-Host "Building examples in $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

$target = "./../../target/$Config/oberon2"


# --------------------------------------------------------------------------------------------------
Write-Host "Running oberon2 on some example files..." -ForegroundColor Cyan
Get-ChildItem ./Oberon2Source/*.mod |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    &$target $_.FullName -q
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
