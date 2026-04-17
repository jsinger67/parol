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

$commonContentTemplate = Get-Content $filePath -Raw -Encoding UTF8

$repoSupplementPath = [System.IO.Path]::Combine($scriptPath, "RepositoryReadMeSupplement.md")
$repoSupplement = if (Test-Path $repoSupplementPath) {
    (Get-Content $repoSupplementPath -Raw -Encoding UTF8).Trim()
}
else {
    ""
}

$repoSupplementMarker = "<!-- REPO_README_SUPPLEMENT -->"
$crateContent = $commonContentTemplate.Replace($repoSupplementMarker + $nl, "")
$repoContent = if ([string]::IsNullOrWhiteSpace($repoSupplement)) {
    $commonContentTemplate.Replace($repoSupplementMarker + $nl, "")
}
else {
    $commonContentTemplate.Replace($repoSupplementMarker, $repoSupplement)
}

# CommonReadMe uses crate-local links. Rewrite them for the repository README.
$repoContent = $repoContent.Replace("(../../book/src/ExportModelContract.md)", "(./book/src/ExportModelContract.md)")
$repoContent = $repoContent.Replace("(./schemas/parser-export-model.v1.schema.json)", "(./crates/parol/schemas/parser-export-model.v1.schema.json)")

$startPattern = "Unfortunately there is no include directive available in Markdown -->"

$crateCommonContent = $startPattern + $nl + $nl + $crateContent
$repoCommonContent = $startPattern + $nl + $nl + $repoContent

$crateReadme = [System.IO.Path]::Combine($scriptPath, "README.md")
$repoReadme = [System.IO.Path]::Combine($scriptPath, "..", "..", "README.md")

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

@(@{ Path = $crateReadme; Content = $crateCommonContent }, @{ Path = $repoReadme; Content = $repoCommonContent }) | ForEach-Object {
    $readmeContent = Get-Content $_.Path  | takeWhile -Predicate { $_ -ne $startPattern }
    
    Set-Content $_.Path $readmeContent -Encoding UTF8
    Add-Content $_.Path $_.Content -Encoding UTF8
}

