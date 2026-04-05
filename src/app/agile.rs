use anyhow::{Context, Result, bail};
use serde_json::Map;

use crate::{
    adf::markdown_to_adf,
    cli::{
        AddEpicIssuesArgs, AddSprintIssuesArgs, CloseSprintArgs, CreateEpicArgs, ListBoardsArgs,
        ListEpicsArgs, ListProjectsArgs, ListReleasesArgs, ListSprintsArgs, RemoveEpicIssuesArgs,
    },
    jira_cloud::CreateIssueRequest,
};

use super::support::{AppContext, read_optional_text};

pub(super) fn list_projects(args: ListProjectsArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let projects = session
        .client
        .list_projects(&session.site, &session.token)?;
    if projects.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    for project in projects {
        println!(
            "{}\t{}\t{}",
            project.key, project.name, project.project_type
        );
    }

    Ok(())
}

pub(super) fn list_boards(args: ListBoardsArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let boards =
        session
            .client
            .list_boards(&session.site, &session.token, args.project.as_deref())?;
    if boards.is_empty() {
        println!("No boards found.");
        return Ok(());
    }

    for board in boards {
        let project_key = board.project_key.unwrap_or_else(|| "-".to_owned());
        println!(
            "{}\t{}\t{}\t{}",
            board.id, board.name, project_key, board.board_type
        );
    }

    Ok(())
}

pub(super) fn list_releases(args: ListReleasesArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let project = app.project(&session.site.key, args.project)?;
    let releases = session
        .client
        .list_releases(&session.site, &session.token, &project)?;
    if releases.is_empty() {
        println!("No releases found.");
        return Ok(());
    }

    for release in releases {
        println!(
            "{}\t{}\treleased={}\tarchived={}",
            release.id, release.name, release.released, release.archived
        );
    }

    Ok(())
}

pub(super) fn list_sprints(args: ListSprintsArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let board_id = args
        .board
        .context("`jira sprint list` currently requires --board")?;

    let mut sprints = session
        .client
        .list_sprints(&session.site, &session.token, board_id)?;
    if args.current {
        sprints.retain(|sprint| sprint.state.eq_ignore_ascii_case("active"));
    } else if args.next {
        sprints.retain(|sprint| sprint.state.eq_ignore_ascii_case("future"));
    } else if args.prev {
        sprints.retain(|sprint| sprint.state.eq_ignore_ascii_case("closed"));
        sprints.truncate(1);
    } else if let Some(state) = args.state.as_ref() {
        let allowed = state
            .split(',')
            .map(|value| value.trim().to_ascii_lowercase())
            .collect::<Vec<_>>();
        sprints.retain(|sprint| allowed.contains(&sprint.state.to_ascii_lowercase()));
    }
    if sprints.is_empty() {
        println!("No sprints found.");
        return Ok(());
    }
    for sprint in sprints {
        println!("{}\t{}\t{}", sprint.id, sprint.name, sprint.state);
    }
    Ok(())
}

pub(super) fn sprint_add(args: AddSprintIssuesArgs) -> Result<()> {
    if args.issues.is_empty() {
        bail!("at least one issue key is required");
    }

    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    session.client.add_issues_to_sprint(
        &session.site,
        &session.token,
        args.sprint_id,
        &args.issues,
    )?;

    println!(
        "Added {} issues to sprint {}",
        args.issues.len(),
        args.sprint_id
    );
    Ok(())
}

pub(super) fn sprint_close(args: CloseSprintArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    session
        .client
        .close_sprint(&session.site, &session.token, args.sprint_id)?;
    println!("Closed sprint {}", args.sprint_id);
    Ok(())
}

pub(super) fn list_epics(args: ListEpicsArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let limit = args.limit.unwrap_or(20);
    let issues = if let Some(epic_key) = args.epic_key.as_ref() {
        session
            .client
            .list_epic_issues(&session.site, &session.token, epic_key.trim(), limit)?
    } else {
        let project = app.project(&session.site.key, args.project)?;
        let jql = format!("project = {project} AND issuetype = Epic ORDER BY updated DESC");
        session
            .client
            .list_issues(&session.site, &session.token, &jql, limit)?
    };

    if issues.is_empty() {
        println!("No epics found.");
        return Ok(());
    }
    for issue in issues {
        let assignee = issue.assignee.unwrap_or_else(|| "unassigned".to_owned());
        println!(
            "{}\t[{}]\t{}\t{}\t{}",
            issue.key, issue.status, issue.issue_type, assignee, issue.summary
        );
    }
    Ok(())
}

pub(super) fn create_epic(args: CreateEpicArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let project_key = app.project(&session.site.key, args.project.clone())?;
    let description = read_optional_text(
        args.description.as_ref(),
        args.description_file.as_ref(),
        "read description from stdin",
        "description file",
    )?
    .map(|markdown| markdown_to_adf(&markdown));

    let issue = session.client.create_issue(
        &session.site,
        &session.token,
        CreateIssueRequest {
            project_key,
            issue_type: "Epic".to_owned(),
            summary: if args.summary.trim().is_empty() {
                args.name.clone().unwrap_or_default()
            } else {
                args.summary.trim().to_owned()
            },
            description,
            extra_fields: Map::new(),
        },
    )?;

    println!("Created epic {}", issue.key);
    println!("Browse: {}/browse/{}", session.site.site_url, issue.key);
    Ok(())
}

pub(super) fn add_epic_issues(args: AddEpicIssuesArgs) -> Result<()> {
    if args.issues.is_empty() {
        bail!("at least one issue key is required");
    }

    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    session.client.add_issues_to_epic(
        &session.site,
        &session.token,
        args.epic_key.trim(),
        &args.issues,
    )?;

    println!(
        "Added {} issues to epic {}",
        args.issues.len(),
        args.epic_key.trim()
    );
    Ok(())
}

pub(super) fn remove_epic_issues(args: RemoveEpicIssuesArgs) -> Result<()> {
    if args.issues.is_empty() {
        bail!("at least one issue key is required");
    }

    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    session
        .client
        .remove_issues_from_epic(&session.site, &session.token, &args.issues)?;
    println!("Removed {} issues from their epic", args.issues.len());
    Ok(())
}
