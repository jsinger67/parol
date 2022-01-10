$ErrorCount = 0

Write-Host "Building release. Please wait..." -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

Write-Host "Building examples in release. Please wait..." -ForegroundColor Cyan
cargo build --examples --release
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running parol on its own parol-grammar..." -ForegroundColor Cyan
./target/release/parol -f ./src/parser/parol-grammar.par -v
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Calc example..." -ForegroundColor Cyan
./target/release/examples/calc ./examples/calc/calc_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running List example..." -ForegroundColor Cyan
./target/release/examples/list ./examples/list/list_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Oberon-0 example..." -ForegroundColor Cyan
./target/release/examples/oberon_0 ./examples/oberon_0/Sample.mod
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Scanner States example..." -ForegroundColor Cyan
./target/release/examples/scanner_states ./examples/scanner_states/scanner_states_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Boolean Parser example..." -ForegroundColor Cyan
./target/release/examples/boolean_parser ./examples/boolean_parser/boolean_parser_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Keywords example..." -ForegroundColor Cyan
Get-ChildItem ./examples/keywords/testfiles/valid/*.txt |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    ./target/release/examples/keywords $_.FullName
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCount    
    }
}
Get-ChildItem ./examples/keywords/testfiles/invalid/*.txt |
ForEach-Object {
    Write-Host "Parsing $($_.FullName) should fail..." -ForegroundColor Magenta
    ./target/release/examples/keywords $_.FullName
    if ($LASTEXITCODE -eq 0) {
        ++$ErrorCount    
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Keywords2 example..." -ForegroundColor Cyan
Get-ChildItem ./examples/keywords/testfiles/valid/*.txt |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    ./target/release/examples/keywords2 $_.FullName
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCount    
    }
}
Get-ChildItem ./examples/keywords/testfiles/invalid/*.txt |
ForEach-Object {
    Write-Host "Parsing $($_.FullName) should fail..." -ForegroundColor Magenta
    ./target/release/examples/keywords2 $_.FullName
    if ($LASTEXITCODE -eq 0) {
        ++$ErrorCount    
    }
}

# Some of the example grammars will fail because they don't pass the basic grammar checks.
# Get-ChildItem ./data/*.par | ForEach-Object { Write-Host $_.FullName -ForegroundColor Blue; ./target/release/parol -f $_.FullName }

# --------------------------------------------------------------------------------------------------
# Final message
# --------------------------------------------------------------------------------------------------
if ($ErrorCount -gt 0) {
    $Msg = "$ErrorCount error(s) occurred."
    Write-Host -Object $Msg  -ForegroundColor Red
} else {
    Write-Host "All examples successfully executed." -ForegroundColor Green
}
