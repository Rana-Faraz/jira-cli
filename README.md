# jira-cli

`jira-cli` is a Rust command-line client for Jira Cloud with `gh`-style ergonomics: keyring-backed authentication, named contexts, Markdown-to-ADF conversion, and a broad command surface for day-to-day issue workflows.

## Features

- Keyring-backed Jira Cloud authentication with `JIRA_TOKEN` override support
- Named contexts so project and site defaults can follow the way you work
- Markdown descriptions/comments converted to Atlassian Document Format
- Issue querying, creation, editing, transitions, linking, cloning, commenting, worklogs, watch management, and deletion
- Project, board, sprint, release, epic, profile, and server-info commands
- Shell completion and manpage generation
- Integration-style CLI tests against mock Jira APIs

## Installation

### Homebrew

Once the tap is configured, install with:

```bash
brew tap Rana-Faraz/tap
brew install jira-cli
```

### macOS and Linux

Install the latest GitHub Release into `/usr/local/bin`:

```bash
curl -fsSL https://raw.githubusercontent.com/Rana-Faraz/jira-cli/main/scripts/install.sh | sh
```

Install into a custom directory:

```bash
curl -fsSL https://raw.githubusercontent.com/Rana-Faraz/jira-cli/main/scripts/install.sh | sh -s -- --bin-dir "$HOME/.local/bin"
```

### Windows PowerShell

Install the latest GitHub Release and add it to the user `PATH`:

```powershell
irm https://raw.githubusercontent.com/Rana-Faraz/jira-cli/main/scripts/install.ps1 | iex
```

### Manual downloads

Prebuilt binaries are published on [GitHub Releases](https://github.com/Rana-Faraz/jira-cli/releases) for:

- macOS Apple Silicon
- macOS Intel
- Linux x86_64
- Windows x86_64

## Quick Start

Authenticate a Jira Cloud site:

```powershell
jira auth login https://your-site.atlassian.net --email you@example.com --web
```

Create an active context with a default project:

```powershell
jira context create work --project SCRUM --set-active
```

Or do the first-run flow in one command:

```powershell
jira init --site https://your-site.atlassian.net --email you@example.com --token <token> --project SCRUM --context work
```

## Common Workflows

### Issue queries

```powershell
jira issue list --project SCRUM
jira issue list --project SCRUM --status "In Progress" --priority High --label backend --csv
jira issue view SCRUM-1 --comments 3
```

### Issue changes

```powershell
jira issue create --summary "New task" --description "Created from **CLI**" --project SCRUM
jira issue create --summary "Rich task" --priority High --label backend --parent SCRUM-10 --field-json 'customfield_10000={"value":"blue"}'
jira issue edit SCRUM-2 --summary "Updated summary" --priority High
jira issue assign SCRUM-2 me
jira issue move SCRUM-2 Done --comment "Shipped" --resolution Fixed
```

### Linking, collaboration, and cleanup

```powershell
jira issue link SCRUM-1 SCRUM-2 Blocks
jira issue remote-link add SCRUM-1 https://example.com "External reference"
jira issue unlink SCRUM-1 SCRUM-2
jira issue comment add SCRUM-2 "Hello **team**"
jira issue worklog add SCRUM-2 "1h 30m" --comment "Pairing session"
jira issue clone SCRUM-2 --replace "Task:Ticket"
jira issue watch SCRUM-2
jira issue watch SCRUM-2 --remove
jira issue delete SCRUM-4 --cascade
```

### Project and planning commands

```powershell
jira me
jira project list
jira board list --project SCRUM
jira sprint list --board 1
jira sprint add 12 SCRUM-1 SCRUM-2
jira sprint close 12
jira release list --project SCRUM
jira epic list --project SCRUM
jira epic create --summary "Epic Alpha" --project SCRUM
jira epic add SCRUM-10 SCRUM-1 SCRUM-2
jira epic remove SCRUM-1 SCRUM-2
jira serverinfo
```

### Shell integration

```powershell
jira completion powershell
jira man --output-dir .\man
jira version
```

## Supported Commands

- `auth login|status|logout`
- `init`
- `context create|use|list|delete`
- `me`
- `open`
- `project list`
- `board list`
- `sprint list|add|close`
- `release list`
- `serverinfo`
- `issue list|view|create|edit|assign|move|link|remote-link add|unlink|clone|delete|watch|comment add|worklog add`
- `epic list|create|add|remove`
- `completion`
- `man`
- `version`

## Authentication and Token Scopes

The CLI targets Jira Cloud and Atlassian Cloud REST APIs. Credentials can be stored in the OS keyring or overridden per-process with `JIRA_TOKEN`.

Core issue and project commands need at least:

- `read:jira-user`
- `read:jira-work`
- `write:jira-work`

Jira Software commands such as boards, sprints, and epic changes also need:

- `read:board-scope:jira-software`
- `read:sprint:jira-software`
- `read:epic:jira-software`
- `write:sprint:jira-software`
- `write:epic:jira-software`

If a token is missing required scopes, the CLI surfaces the underlying Jira scope mismatch rather than only a generic auth error.

## Configuration

- Config is stored in the platform config directory by default.
- Set `JIRA_CONFIG_DIR` to override the config location.
- Set `JIRA_TOKEN` to override the stored keyring token for the current process.
- `jira open` prints the resolved URL by default; add `--launch` to open it in a browser.

## Development

Run the full local verification suite with:

```powershell
cargo fmt
cargo test
```

Tagged releases are built by GitHub Actions and published as downloadable release assets.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

This project is licensed under the [MIT License](./LICENSE).
