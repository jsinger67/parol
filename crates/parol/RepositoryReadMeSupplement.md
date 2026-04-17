## Security alert operations

The repository contains an automated Dependabot security caretaker workflow:

- [dependabot-security-agent.yml](./.github/workflows/dependabot-security-agent.yml)

What it does:

- runs daily and on manual dispatch
- maintains an issue called "Dependabot Security Alerts Dashboard" when alerts exist
- summarizes open Dependabot alerts by severity and ecosystem
- updates Dependabot PRs (main/release branches) with a targeted triage comment
- marks the workflow as failed when critical alerts are open
- auto-closes the dashboard issue when alert count returns to zero

Suggested maintainer setup:

- subscribe to GitHub Action failure notifications for this workflow
- use the manual `workflow_dispatch` trigger after merging dependency updates
- treat the generated dashboard issue as the single triage board while alerts are open
