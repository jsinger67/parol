$ErrorCount = 0

Get-ChildItem *.par |
ForEach-Object {
    $file = $_.Name
    $expected = "./$([System.IO.Path]::GetFileNameWithoutExtension($_)).expected"
    $raw = "./$([System.IO.Path]::GetFileNameWithoutExtension($_)).raw"
    $expanded = "./$([System.IO.Path]::GetFileNameWithoutExtension($_)).exp"
    Write-Host "&parol -v -f $file -i $expected -u $raw -e $expanded" -ForegroundColor Cyan
    cargo run -- -v -f $file -i $expected -u $raw -e $expanded
    if ($LASTEXITCODE -ne 0) {
        ++$ErrorCount    
    }
    # Write-Host "&parol -v -f $file -i $expected" -ForegroundColor Cyan
    # cargo run --bin -- -v -f $file -i $expected
}

# --------------------------------------------------------------------------------------------------
# Final message
# --------------------------------------------------------------------------------------------------
if ($ErrorCount -gt 0) {
    $Msg = "$ErrorCount error(s) occurred."
    Write-Host -Object $Msg  -ForegroundColor Red
} else {
    Write-Host "All files successfully updated." -ForegroundColor Green
}
