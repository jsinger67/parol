# The following example invocations assume that you have cloned
# [toml-test](https://github.com/BurntSushi/toml-test.git) parallel to this crate.
#
# ./test.ps1 -Verbose -Path ../toml-test/tests/invalid/ -NegativeTests
# ./test.ps1 -Verbose -Path ../toml-test/tests/valid/ -StopAtFirstError

[CmdletBinding()]
param(
    [Parameter(Mandatory)][string] $Path = "../toml-test/tests/valid/",
    [string][ValidatePattern("debug|release")] $Config = "debug",
    [switch] $NegativeTests,
    [switch] $StopAtFirstError)

$CargoConfig = if ($Config -eq "release") { "--release" } else { "" }
$Verbose = $VerbosePreference -eq "continue"

$ErrorCount = 0
$FileCount = 0

$FilesWithErrors = @()

Write-Host "Building $Config. Please wait..." -ForegroundColor Cyan
cargo build $CargoConfig
if ($LASTEXITCODE -ne 0) {
    Exit 1
}


function  Invoke-SingleFile {
    param (
        [string] $fileName
    )
    if ($StopAtFirstError -and $Script:ErrorCount -gt 0) {
        return
    }
    $Script:FileCount++
    if ($Script:Verbose) {
        Write-Host "./target/$Config/parol_toml.exe $fileName -q" -ForegroundColor Yellow
    }
    &"./target/$Config/parol_toml.exe" $fileName -q
    if (-not $?) {
        # Failed
        if (-not $NegativeTests) {
            # Should not fail
            $Script:ErrorCount++
            $Script:FilesWithErrors += $fileName
        }
    } else {
        # Succeeded
        if ($NegativeTests) {
            # Should actually fail
            $Script:ErrorCount++
            $Script:FilesWithErrors += $fileName
        }
    }
}

Get-ChildItem -Path $Path -Include *.toml -Recurse |
ForEach-Object { Invoke-SingleFile $_.FullName }

if ($ErrorCount -gt 0) {
    if ($Script:Verbose) {
        Write-Host "$FileCount files parsed. $ErrorCount error(s) occurred" -ForegroundColor Red
    }
}
else {
    if ($Script:Verbose) {
        Write-Host "$FileCount files parsed. No errors occurred" -ForegroundColor Green
    }
}

$FilesWithErrors
