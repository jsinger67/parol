param(
    [ValidatePattern("debug|release")]
    $Config = "debug",
    [switch] $StopOnError
)

$ErrorCount = 0
$FailedExamples = @()
$CargoConfig = if ($Config -eq "release") { "--release" } else { $null }

# --------------------------------------------------------------------------------------------------
# Final message
# --------------------------------------------------------------------------------------------------
function FinalMessage {
    if ($Script:ErrorCount -gt 0) {
        $Msg = "$ErrorCount error(s) occurred."
        Write-Host -Object $Msg  -ForegroundColor Red
        $Script:FailedExamples | ForEach-Object { Write-Host $_ -ForegroundColor Red }
    }
    else {
        Write-Host "All examples successfully executed." -ForegroundColor Green
    }
}

# --------------------------------------------------------------------------------------------------
# Main
# --------------------------------------------------------------------------------------------------
Write-Host "Building $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount
    $FailedExamples += "Building parol"
    if ($StopOnError) {
        FinalMessage
        exit 1
    }
}

Write-Host "Building examples in $Config. Please wait..." -ForegroundColor Cyan
cargo build --examples $CargoConfig
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount
    $FailedExamples += "Building examples"
    if ($StopOnError) {
        FinalMessage
        exit 1
    }
}

$target = "../../target/$Config/parol"
$target_dir = "../../target/$Config/examples"

# --------------------------------------------------------------------------------------------------
Write-Host "Running parol on its own grammar..." -ForegroundColor Cyan
&$target -f ./src/parser/parol.par
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount
    $FailedExamples += "Running parol on its own parol"
    if ($StopOnError) {
        FinalMessage
        exit 1
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running parol on some example grammars..." -ForegroundColor Cyan
Get-ChildItem ./data/valid/*.par |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    &$target -f $_.FullName
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCount
        $FailedExamples += "Parsing $($_.FullName)"
        if ($StopOnError) {
            FinalMessage
            exit 1
        }
    }
}
Get-ChildItem ./data/invalid/*.par |
ForEach-Object {
    Write-Host "Parsing $($_.FullName) should fail..." -ForegroundColor Magenta
    &$target -f $_.FullName
    if ($LASTEXITCODE -eq 0) {
        ++$ErrorCount
        $FailedExamples += "Parsing $($_.FullName) should fail"
        if ($StopOnError) {
            FinalMessage
            exit 1
        }
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Calc example..." -ForegroundColor Cyan
&"$target_dir/calc" ../../examples/calc/calc_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount
    $FailedExamples += "Running Calc example"
    if ($StopOnError) {
        FinalMessage
        exit 1
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running List example..." -ForegroundColor Cyan
&"$target_dir/list" ../../examples/list/list_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount
    $FailedExamples += "Running List example"
    if ($StopOnError) {
        FinalMessage
        exit 1
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Oberon-0 example..." -ForegroundColor Cyan
&"$target_dir/oberon_0" ../../examples/oberon_0/Sample.mod
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount
    $FailedExamples += "Running Oberon-0 example"
    if ($StopOnError) {
        FinalMessage
        exit 1
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Scanner States example..." -ForegroundColor Cyan
&"$target_dir/scanner_states" ../../examples/scanner_states/scanner_states_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount
    $FailedExamples += "Running Scanner States example"
    if ($StopOnError) {
        FinalMessage
        exit 1
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Boolean Parser example..." -ForegroundColor Cyan
&"$target_dir/boolean_parser" ../../examples/boolean_parser/boolean_parser_test.txt
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCount
    $FailedExamples += "Running Boolean Parser example"
    if ($StopOnError) {
        FinalMessage
        exit 1
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Keywords example..." -ForegroundColor Cyan
Get-ChildItem ../../examples/keywords/testfiles/valid/*.txt |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    &"$target_dir/keywords" $_.FullName
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCount
        $FailedExamples += "Keywords: Parsing $($_.FullName)"
        if ($StopOnError) {
            FinalMessage
            exit 1
        }
    }
}
Get-ChildItem ../../examples/keywords/testfiles/invalid/*.txt |
ForEach-Object {
    Write-Host "Keywords: Parsing $($_.FullName) should fail..." -ForegroundColor Magenta
    &"$target_dir/keywords" $_.FullName
    if ($LASTEXITCODE -eq 0) {
        ++$ErrorCount
        $FailedExamples += "Keywords: Parsing $($_.FullName) should fail"
        if ($StopOnError) {
            FinalMessage
            exit 1
        }
    }
}

# --------------------------------------------------------------------------------------------------
Write-Host "Running Keywords2 example..." -ForegroundColor Cyan
Get-ChildItem ../../examples/keywords/testfiles/valid/*.txt |
ForEach-Object {
    Write-Host "Parsing $($_.FullName)..." -ForegroundColor Yellow
    &"$target_dir/keywords2" $_.FullName
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCount
        $FailedExamples += "Keywords2: Parsing $($_.FullName)"
        if ($StopOnError) {
            FinalMessage
            exit 1
        }
    }
}
Get-ChildItem ../../examples/keywords/testfiles/invalid/*.txt |
ForEach-Object {
    Write-Host "Keywords2: Parsing $($_.FullName) should fail..." -ForegroundColor Magenta
    &"$target_dir/keywords2" $_.FullName
    if ($LASTEXITCODE -eq 0) {
        ++$ErrorCount
        $FailedExamples += "Keywords2: Parsing $($_.FullName) should fail"
        if ($StopOnError) {
            FinalMessage
            exit 1
        }
    }
}

FinalMessage
