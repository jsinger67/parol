$ErrorCount = 0
$Config = "release"
$CargoConfig = if ($Config -eq "release") { "--release" } else { $null }
$env:RUST_LOG = ""

Write-Host "Building $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

$target = "./../../target/$Config/basic"


# --------------------------------------------------------------------------------------------------
# Negative tests
# --------------------------------------------------------------------------------------------------
Get-ChildItem ./tests/data/invalid/*.bas |
ForEach-Object {
    Write-Host "Parsing $($_.FullName) should fail..." -ForegroundColor Magenta
    &$target $_.FullName -q
    if ($?) {
        ++$ErrorCount    
    }
}

# --------------------------------------------------------------------------------------------------
# Positive tests
# --------------------------------------------------------------------------------------------------
Get-ChildItem ./tests/data/valid/*.bas |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    &$target $_.FullName -q | Tee-Object -Variable output
    if (-not $?) {
        ++$ErrorCount    
    } else {
        $expected_file = [System.IO.Path]::ChangeExtension($_.FullName, "expected");
        if ([System.IO.File]::Exists($expected_file)) {
            [string] $expected = $(Get-Content $expected_file -Encoding utf8) -replace "\r?\n|\r", " "
            [string] $output = $output -replace "\r?\n|\r", " "
            if ($expected -cne $output) {
                Write-Host "Result mismatch for $($_.FullName)" -ForegroundColor Red
                Write-Host "Expecting '$expected'"
                Write-Host "Received  '$output'"
                ++$ErrorCount
            }
        } else {
            Write-Host "$expected_file not found" -ForegroundColor Red
            ++$ErrorCount
        }
    }
}

# --------------------------------------------------------------------------------------------------
# Final message
# --------------------------------------------------------------------------------------------------
if ($ErrorCount -gt 0) {
    $Msg = "$ErrorCount error(s) occurred."
    Write-Host -Object $Msg  -ForegroundColor Red
} else {
    Write-Host "All examples successfully executed." -ForegroundColor Green
}
