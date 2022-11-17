param(
    [ValidatePattern("debug|release")]
    $Config = "debug"
)

$ErrorCont = 0
$CargoConfig = if ($Config -eq "release") { "--release" } else { $null }

Write-Host "Building $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}


# --------------------------------------------------------------------------------------------------
Write-Host "Building parol..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./src/parser/parol-grammar.par -e ./src/parser/parol-grammar-exp.par -p ./src/parser/parol_parser.rs -a ./src/parser/parol_grammar_trait.rs -t ParolGrammar -m parser::parol_grammar -g
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Building Calc example..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./examples/calc/calc.par -e ./examples/calc/calc-exp.par -p ./examples/calc/calc_parser.rs -a ./examples/calc/calc_grammar_trait.rs -t CalcGrammar -m calc_grammar
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Building CalcAuto example..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./examples/calc_auto/calc.par -e ./examples/calc_auto/calc-exp.par -p ./examples/calc_auto/calc_parser.rs -a ./examples/calc_auto/calc_grammar_trait.rs -t CalcGrammar -m calc_grammar -g
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Building List example..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./examples/list/list.par -e ./examples/list/list-exp.par -p ./examples/list/list_parser.rs -a ./examples/list/list_grammar_trait.rs -t ListGrammar -m list_grammar
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Building ListAuto example..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./examples/list_auto/list.par -e ./examples/list_auto/list-exp.par -p ./examples/list_auto/list_parser.rs -a ./examples/list_auto/list_grammar_trait.rs -t ListGrammar -m list_grammar -g
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Building Oberon-0 example..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./examples/oberon_0/oberon_0.par -e ./examples/oberon_0/oberon_0-exp.par -p ./examples/oberon_0/oberon_0_parser.rs -a ./examples/oberon_0/oberon_0_grammar_trait.rs -t Oberon0Grammar -m oberon_0_grammar
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Building Scanner States example..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./examples/scanner_states/scanner_states.par -e ./examples/scanner_states/scanner_states-exp.par -p ./examples/scanner_states/scanner_states_parser.rs -a ./examples/scanner_states/scanner_states_grammar_trait.rs -t ScannerStatesGrammar -m scanner_states_grammar
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Building Boolean Parser example..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./examples/boolean_parser/boolean-parser.par -e ./examples/boolean_parser/boolean-parser-exp.par -p ./examples/boolean_parser/boolean_parser.rs -a ./examples/boolean_parser/boolean_grammar_trait.rs -t BooleanGrammar -m boolean_grammar
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Building Keywords example..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./examples/keywords/keywords.par -e ./examples/keywords/keywords-exp.par -p ./examples/keywords/keywords_parser.rs -a ./examples/keywords/keywords_grammar_trait.rs -t KeywordsGrammar -m keywords_grammar
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

# --------------------------------------------------------------------------------------------------
Write-Host "Building Keywords2 example..." -ForegroundColor Cyan
&"./target/$Config/parol" -f ./examples/keywords2/keywords.par -e ./examples/keywords2/keywords-exp.par -p ./examples/keywords2/keywords_parser.rs -a ./examples/keywords2/keywords_grammar_trait.rs -t KeywordsGrammar -m keywords_grammar
if ($LASTEXITCODE -ne 0) {
    ++$ErrorCont    
}

# --------------------------------------------------------------------------------------------------
# Final message
# --------------------------------------------------------------------------------------------------
if ($ErrorCont -gt 0) {
    $Msg = "$ErrorCount error(s) occurred during builds."
    Write-Host -Object $Msg  -ForegroundColor Red
} else {
    Write-Host "All builds successfully executed." -ForegroundColor Green
}
