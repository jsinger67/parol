Get-ChildItem *.par |
ForEach-Object {
    $file = $_.Name
    $expected = "./$([System.IO.Path]::GetFileNameWithoutExtension($_)).expected"
    $raw = "./$([System.IO.Path]::GetFileNameWithoutExtension($_)).raw"
    $expanded = "./$([System.IO.Path]::GetFileNameWithoutExtension($_)).exp"
    Write-Host "&parol -v -f $file -i $expected -u $raw -e $expanded" -ForegroundColor Cyan
    cargo run --bin parol -- -v -f $file -i $expected -u $raw -e $expanded
    # Write-Host "&parol -v -f $file -i $expected" -ForegroundColor Cyan
    # cargo run --bin parol -- -v -f $file -i $expected
}