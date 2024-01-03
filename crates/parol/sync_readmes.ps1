<#
.SYNOPSIS
This script synchronizes README files with the CommonReadMe.md.

.DESCRIPTION
This script inserts the content of the CommonReadMe.md into the two README files in
    * crates/parol/README.md
    * README.md

.PARAMETER ParameterName
The script takes no parameters.

.INPUTS
No input objects are needed.

.OUTPUTS
The script does not return any output.

.EXAMPLE
./crates/parol/sync_readmes.ps1
Runs the script from the root of the repository.

.NOTES
I wrote this script because I wanted to have a common README file for the crate and the repository.
The crate README file is located in the crates/parol/README.md file and the repository README file
is located in the README.md file.
Unfortunately there is no include directive available in Markdown, so I had to write this script to
synchronize the two files.
#>


$scriptPath = [System.IO.Path]::GetDirectoryName($MyInvocation.MyCommand.Path)
$filePath = [System.IO.Path]::Combine($scriptPath, "CommonReadMe.md")

$nl = [Environment]::NewLine

$commonContent = Get-Content $filePath -Raw -Encoding UTF8

$startPattern = "Unfortunately there is no include directive available in Markdown -->"
$commonContent = $startPattern + $nl + $nl + $commonContent

$readmeFiles = [System.IO.Path]::Combine($scriptPath, "README.md"),
[System.IO.Path]::Combine($scriptPath, "..", "..", "README.md")

function takeWhile {
    param (
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [object[]]$InputObject,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Predicate
    )

    begin {
        $found = $false
    }

    process {
        if (!$found) {
            if (& $Predicate $_) {
                $_
            }
            else {
                $found = $true
            }
        }
    }
}

$readmeFiles | ForEach-Object {
    $readmeContent = Get-Content $_  | takeWhile -Predicate { $_ -ne $startPattern }
    
    Set-Content $_ $readmeContent -Encoding UTF8
    Add-Content $_ $commonContent -Encoding UTF8
}

