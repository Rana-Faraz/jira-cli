---
name: jira
description: Use the jira-cli command-line tool to manage Jira Cloud authentication, contexts, issues, epics, sprints, boards, projects, releases, comments, worklogs, and browser links from the terminal. Trigger when a user asks to use `jira`, `jira-cli`, Jira issue commands, JQL-backed issue listing, context switching, or Jira Cloud workflow automation.
---

# Jira CLI

Use `jira` for Jira Cloud work in the terminal.

## Dependency Check

Before running Jira commands, verify the CLI is installed:

```bash
jira version
```

If `jira` is missing, install it first:

| Platform | Command |
|----------|---------|
| macOS/Linux | `curl -fsSL https://raw.githubusercontent.com/Rana-Faraz/jira-cli/main/scripts/install.sh | sh` |
| Windows PowerShell | `irm https://raw.githubusercontent.com/Rana-Faraz/jira-cli/main/scripts/install.ps1 | iex` |
| Homebrew | `brew tap Rana-Faraz/tap && brew install jira-cli` |
| Binary | Download from [GitHub Releases](https://github.com/Rana-Faraz/jira-cli/releases) |

Only continue after `jira version` succeeds.

## Authentication and Context

Check whether Jira Cloud auth is already configured:

```bash
jira auth status
```

Authenticate a site:

```bash
jira auth login https://your-site.atlassian.net --email you@example.com --web
```

Create and activate a default context:

```bash
jira context create work --project SCRUM --set-active
jira context list
jira context use work
```

Use `jira init` when the user wants a one-shot first-run setup:

```bash
jira init --site https://your-site.atlassian.net --email you@example.com --token <token> --project SCRUM --context work
```

## Workflow Notes

- Target Jira Cloud, not Jira Server or Data Center.
- Prefer using an active context so `issue create`, `issue list`, `release list`, and epic commands resolve the default project automatically.
- `jira issue list` requires either `--jql` or a resolved project from `--project` or the active context.
- `jira issue list --raw` returns JSON. `--csv` returns CSV. Other commands mostly print human-readable text.
- `jira issue create`, `jira issue edit`, and `jira epic create` can read descriptions from files with `--description-file`. Use `-` to read from stdin.
- `jira issue comment add` can read the body from `--template -` via stdin.
- `JIRA_TOKEN` overrides the stored keyring token for the current process.
- `JIRA_CONFIG_DIR` overrides the config directory.

## Common Commands

```bash
# Read
jira issue list --project SCRUM
jira issue list --project SCRUM --status "In Progress" --priority High --label backend --raw
jira issue view SCRUM-1 --comments 3
jira me
jira project list
jira board list --project SCRUM
jira sprint list --board 1
jira release list --project SCRUM
jira epic list --project SCRUM

# Write
jira issue create --summary "New task" --description "Created from **CLI**" --project SCRUM
jira issue edit SCRUM-2 --summary "Updated summary" --priority High
jira issue assign SCRUM-2 me
jira issue move SCRUM-2 Done --comment "Shipped" --resolution Fixed
jira issue comment add SCRUM-2 "Hello **team**"
jira issue worklog add SCRUM-2 "1h 30m" --comment "Pairing session"
jira issue link SCRUM-1 SCRUM-2 Blocks
jira issue remote-link add SCRUM-1 https://example.com "External reference"
jira issue unlink SCRUM-1 SCRUM-2
jira issue clone SCRUM-2 --replace "Task:Ticket"
jira issue watch SCRUM-2
jira issue watch SCRUM-2 --remove
jira issue delete SCRUM-4 --cascade

# Agile
jira sprint add 12 SCRUM-1 SCRUM-2
jira sprint close 12
jira epic create --summary "Epic Alpha" --project SCRUM
jira epic add SCRUM-10 SCRUM-1 SCRUM-2
jira epic remove SCRUM-1 SCRUM-2
```

## References

- Full command reference: [references/commands.md](references/commands.md)
