<#
.SYNOPSIS
Checks Dependabot security and update status for a GitHub repository.

.DESCRIPTION
Provides quick checks for:
- Open Dependabot security alerts
- Open Dependabot pull requests
- Recently failed GitHub Actions workflow runs

If no mode switch is provided, the script defaults to -Summary.

.PARAMETER Repo
Repository in owner/name format.
Default: jsinger67/parol

.PARAMETER Summary
Prints a compact traffic-light style summary line with counts.

.PARAMETER OpenAlerts
Prints detailed information for open Dependabot security alerts.

.PARAMETER FailedRuns
Prints recent failed workflow runs from GitHub Actions.

.PARAMETER RunLimit
Number of failed workflow runs to query when -Summary or -FailedRuns is used.
Default: 20

.EXAMPLE
./tools/dependabot-check.ps1 -Summary

Prints summary counts for open alerts, open Dependabot PRs, and failed runs.

.EXAMPLE
./tools/dependabot-check.ps1 -OpenAlerts

Prints details of open Dependabot security alerts, or a message when none exist.

.EXAMPLE
./tools/dependabot-check.ps1 -FailedRuns -RunLimit 10

Prints the 10 most recent failed workflow runs.

.EXAMPLE
./tools/dependabot-check.ps1 -Repo owner/repo -Summary

Runs the summary check for another repository.

.NOTES
Requires GitHub CLI (gh) configured and authenticated.
#>
[CmdletBinding()]
param(
    [string]$Repo = "jsinger67/parol",
    [switch]$Summary,
    [switch]$OpenAlerts,
    [switch]$FailedRuns,
    [int]$RunLimit = 20
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    throw "GitHub CLI (gh) is required but was not found in PATH."
}

# If no mode is selected, default to summary.
if (-not ($Summary -or $OpenAlerts -or $FailedRuns)) {
    $Summary = $true
}

function Invoke-GhJson {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Args
    )

    $output = & gh @Args
    if ([string]::IsNullOrWhiteSpace($output)) {
        return $null
    }
    return $output | ConvertFrom-Json
}

function Get-OpenDependabotAlerts {
    $apiPath = "/repos/$Repo/dependabot/alerts?state=open&per_page=100"
    Invoke-GhJson -Args @("api", "-H", "Accept: application/vnd.github+json", $apiPath)
}

function Get-DependabotOpenPrs {
    Invoke-GhJson -Args @(
        "pr", "list",
        "--repo", $Repo,
        "--state", "open",
        "--search", "author:app/dependabot",
        "--json", "number,title,headRefName,url"
    )
}

function Get-FailedWorkflowRuns {
    Invoke-GhJson -Args @(
        "run", "list",
        "--repo", $Repo,
        "--status", "failure",
        "--limit", "$RunLimit",
        "--json", "databaseId,name,workflowName,createdAt,url"
    )
}

if ($Summary) {
    $openAlertItems = @(Get-OpenDependabotAlerts)
    $openDependabotPrItems = @(Get-DependabotOpenPrs)
    $failedRunItems = @(Get-FailedWorkflowRuns)

    $alertsState = if ($openAlertItems.Count -gt 0) { "RED" } else { "GREEN" }
    $prsState = if ($openDependabotPrItems.Count -gt 0) { "YELLOW" } else { "GREEN" }
    $runsState = if ($failedRunItems.Count -gt 0) { "YELLOW" } else { "GREEN" }

    Write-Output (
        "alerts={0} ({1}) | dependabot_prs={2} ({3}) | failed_runs_last{4}={5} ({6})" -f
        $openAlertItems.Count, $alertsState, $openDependabotPrItems.Count, $prsState, $RunLimit, $failedRunItems.Count, $runsState
    )
}

if ($OpenAlerts) {
    $alerts = @(Get-OpenDependabotAlerts)
    if ($alerts.Count -eq 0) {
        Write-Output "No open Dependabot security alerts."
    }
    else {
        $alerts | Select-Object `
            @{ Name = "number"; Expression = { $_.number } },
            @{ Name = "ecosystem"; Expression = { $_.dependency.package.ecosystem } },
            @{ Name = "package"; Expression = { $_.dependency.package.name } },
            @{ Name = "severity"; Expression = { $_.security_advisory.severity } },
            @{ Name = "summary"; Expression = { $_.security_advisory.summary } },
            @{ Name = "manifest"; Expression = { $_.dependency.manifest_path } },
            @{ Name = "fixed_in"; Expression = { $_.security_vulnerability.first_patched_version.identifier } },
            @{ Name = "url"; Expression = { $_.html_url } }
    }
}

if ($FailedRuns) {
    $failedRunItems = @(Get-FailedWorkflowRuns)
    if ($failedRunItems.Count -eq 0) {
        Write-Output "No failed workflow runs in the selected window."
    }
    else {
        $failedRunItems |
            Select-Object databaseId, workflowName, name, createdAt, url
    }
}
