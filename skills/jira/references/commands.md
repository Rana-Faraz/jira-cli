# Jira CLI Command Reference

Complete command reference for the `jira` CLI in this repository. Run `jira <command> --help` for flag details.

## Authentication

```bash
jira auth login https://your-site.atlassian.net --email you@example.com --web
jira auth status
jira auth logout
jira auth logout example.atlassian.net
```

## Contexts

```bash
jira context create work --project SCRUM --set-active
jira context create ops --site other.atlassian.net --project OPS
jira context list
jira context use work
jira context delete ops
```

## Issues

### List and view

```bash
jira issue list --project SCRUM
jira issue list --jql 'project = SCRUM AND assignee = currentUser()' --raw
jira issue list --project SCRUM --status "In Progress" --priority High --label backend --csv
jira issue view SCRUM-1
jira issue view SCRUM-1 --comments 5
```

### Create

```bash
jira issue create --summary "New task" --project SCRUM
jira issue create --summary "Rich task" --description "Created from **CLI**" --priority High --label backend
jira issue create --summary "Ticket from file" --description-file ./body.md --project SCRUM
jira issue create --summary "From stdin" --description-file - --project SCRUM
jira issue create --summary "Custom fields" --field team=platform --field-json 'customfield_10000={"value":"blue"}'
```

### Edit and assign

```bash
jira issue edit SCRUM-2 --summary "Updated summary" --priority High
jira issue edit SCRUM-2 --description-file ./updated.md
jira issue edit SCRUM-2 --assignee me
jira issue assign SCRUM-2 me
jira issue assign SCRUM-2 default
jira issue assign SCRUM-2 x
```

### Move, comment, and worklog

```bash
jira issue move SCRUM-2 Done
jira issue move SCRUM-2 Done --comment "Shipped" --resolution Fixed
jira issue move SCRUM-2 "In Review" --assignee me
jira issue comment add SCRUM-2 "Hello **team**"
jira issue comment add SCRUM-2 --template ./comment.md
jira issue worklog add SCRUM-2 "1h 30m" --comment "Pairing session"
```

### Links, clone, watch, delete

```bash
jira issue link SCRUM-1 SCRUM-2 Blocks
jira issue remote-link add SCRUM-1 https://example.com "External reference"
jira issue unlink SCRUM-1 SCRUM-2
jira issue clone SCRUM-2 --replace "Task:Ticket"
jira issue clone SCRUM-2 --project OPS --type Story --summary "Follow-up ticket"
jira issue watch SCRUM-2
jira issue watch SCRUM-2 --remove
jira issue delete SCRUM-4 --cascade
```

## Agile and Planning

```bash
jira board list --project SCRUM
jira sprint list --board 1
jira sprint list --board 1 --current
jira sprint list --board 1 --state active,future
jira sprint add 12 SCRUM-1 SCRUM-2
jira sprint close 12

jira epic list --project SCRUM
jira epic list --epic-key SCRUM-10
jira epic create --summary "Epic Alpha" --project SCRUM
jira epic create --summary "Epic with file" --project SCRUM --description-file ./epic.md
jira epic add SCRUM-10 SCRUM-1 SCRUM-2
jira epic remove SCRUM-1 SCRUM-2
```

## Project Metadata and Browser Links

```bash
jira me
jira project list
jira release list --project SCRUM
jira serverinfo
jira open
jira open SCRUM-1
jira open https://your-site.atlassian.net/browse/SCRUM-1 --launch
```

## Shell Integration

```bash
jira completion powershell
jira completion bash
jira man --output-dir ./man
jira version
```

## Environment Variables

- `JIRA_TOKEN`: Override the stored keyring token for the current process.
- `JIRA_CONFIG_DIR`: Override the config directory.
