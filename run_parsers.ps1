cargo run --bin parol -- -f ./src/parser/parol-grammar.par -v
cargo run --example calc -- ./examples/calc/calc_test.txt
cargo run --example list -- ./examples/list/list_test.txt
cargo run --example oberon_0 -- ./examples/oberon_0/Sample.mod

# Some of the example grammars will fail because they don't pass the basic grammar checks.
# Get-ChildItem .\data\*.par | ForEach-Object { cargo run --bin parol -- -f $_.FullName }
