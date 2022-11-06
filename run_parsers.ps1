param(
    [ValidatePattern("debug|release")]
    $Config = "debug"
)

$ErrorCount = 0
$CargoConfig = if ($Config -eq "release") { "--release" } else { "" }

Write-Host "Building $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

Write-Host "Building examples in $Config. Please wait..." -ForegroundColor Cyan
cargo build --examples $CargoConfig
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running parol on its own parol-grammar..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./src/parser/parol-grammar.par -v
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running parol on some example grammars..." -ForegroundColor Cyan
Get-ChildItem ./data/valid/*.par |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    &"./target/$Config/parol" -f $_.FullName
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCount    
    }
}
Get-ChildItem ./data/invalid/*.par |
ForEach-Object {
    Write-Host "Parsing $($_.FullName) should fail..." -ForegroundColor Magenta
    &"./target/$Config/parol" -f $_.FullName
    if ($LASTEXITCODE -eq 0) {
        ++$ErrorCount    
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Calc example..." -ForegroundColor Cyan
&"./target/$Config/examples/calc" ./examples/calc/calc_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running CalcAuto example..." -ForegroundColor Cyan
&"./target/$Config/examples/calc_auto" ./examples/calc_auto/calc_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running List example..." -ForegroundColor Cyan
&"./target/$Config/examples/list" ./examples/list/list_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running ListAuto example..." -ForegroundColor Cyan
&"./target/$Config/examples/list_auto" ./examples/list_auto/list_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Oberon-0 example..." -ForegroundColor Cyan
&"./target/$Config/examples/oberon_0" ./examples/oberon_0/Sample.mod
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Scanner States example..." -ForegroundColor Cyan
&"./target/$Config/examples/scanner_states" ./examples/scanner_states/scanner_states_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Boolean Parser example..." -ForegroundColor Cyan
&"./target/$Config/examples/boolean_parser" ./examples/boolean_parser/boolean_parser_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Keywords example..." -ForegroundColor Cyan
Get-ChildItem ./examples/keywords/testfiles/valid/*.txt |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    &"./target/$Config/examples/keywords" $_.FullName
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCount    
    }
}
Get-ChildItem ./examples/keywords/testfiles/invalid/*.txt |
ForEach-Object {
    Write-Host "Parsing $($_.FullName) should fail..." -ForegroundColor Magenta
    &"./target/$Config/examples/keywords" $_.FullName
    if ($LASTEXITCODE -eq 0) {
        ++$ErrorCount    
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Keywords2 example..." -ForegroundColor Cyan
Get-ChildItem ./examples/keywords/testfiles/valid/*.txt |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    &"./target/$Config/examples/keywords2" $_.FullName
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCount    
    }
}
Get-ChildItem ./examples/keywords/testfiles/invalid/*.txt |
ForEach-Object {
    Write-Host "Parsing $($_.FullName) should fail..." -ForegroundColor Magenta
    &"./target/$Config/examples/keywords2" $_.FullName
    if ($LASTEXITCODE -eq 0) {
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
    Write-Host "All examples successfully executed." -ForegroundColor Green
}
