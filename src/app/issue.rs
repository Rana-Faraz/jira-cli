use anyhow::{Context, Result, bail};
use serde_json::{Map, Value, json};

use crate::{
    adf::{adf_to_plain_text, markdown_to_adf},
    cli::{
        AddCommentArgs, AddRemoteLinkArgs, AddWorklogArgs, AssignIssueArgs, CreateIssueArgs,
        DeleteIssueArgs, EditIssueArgs, LinkIssueArgs, ListIssuesArgs, MoveIssueArgs,
        UnlinkIssueArgs, ViewIssueArgs, WatchIssueArgs,
    },
    jira_cloud::{CreateIssueRequest, IssueSummary},
};

use super::support::{
    AppContext, JiraSession, merge_raw_fields, read_optional_text, read_required_text,
    resolve_assignee_id,
};

pub(super) fn create(args: CreateIssueArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let request = build_create_issue_request(&app, &session, args)?;
    let issue = session
        .client
        .create_issue(&session.site, &session.token, request)?;

    println!("Created issue {}", issue.key);
    println!("Browse: {}/browse/{}", session.site.site_url, issue.key);
    println!("API: {}", issue.self_url);
    println!("ID: {}", issue.id);
    Ok(())
}

pub(super) fn view(args: ViewIssueArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let issue = session
        .client
        .get_issue(&session.site, &session.token, args.key.trim())?;
    let comments = if args.comments > 0 {
        session.client.list_issue_comments(
            &session.site,
            &session.token,
            args.key.trim(),
            args.comments,
        )?
    } else {
        Vec::new()
    };

    println!("{}", issue.key);
    println!("Summary: {}", issue.summary);
    println!("Status: {}", issue.status);
    println!("Type: {}", issue.issue_type);
    println!("Project: {}", issue.project_key);
    println!(
        "Assignee: {}",
        issue.assignee.as_deref().unwrap_or("Unassigned")
    );
    println!(
        "Reporter: {}",
        issue.reporter.as_deref().unwrap_or("Unknown")
    );
    if !issue.created.is_empty() {
        println!("Created: {}", issue.created);
    }
    if !issue.updated.is_empty() {
        println!("Updated: {}", issue.updated);
    }
    println!("Browse: {}/browse/{}", session.site.site_url, issue.key);

    if let Some(description) = issue.description {
        let text = adf_to_plain_text(&description);
        if !text.is_empty() {
            println!();
            println!("Description:");
            println!("{text}");
        }
    }

    if !comments.is_empty() {
        println!();
        println!("Comments:");
        for comment in comments {
            if comment.created.is_empty() {
                println!("- {}", comment.author);
            } else {
                println!("- {} ({})", comment.author, comment.created);
            }

            if let Some(body) = comment.body {
                let text = adf_to_plain_text(&body);
                if !text.is_empty() {
                    println!("{text}");
                }
            }
        }
    }

    Ok(())
}

pub(super) fn list(args: ListIssuesArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;

    let jql = if let Some(jql) = args.jql.as_ref() {
        let jql = jql.trim();
        if jql.is_empty() {
            bail!("--jql cannot be empty");
        }
        jql.to_owned()
    } else {
        build_issue_jql(&app, &session, &args)?
    };

    let issues = session
        .client
        .list_issues(&session.site, &session.token, &jql, args.limit)?;

    if issues.is_empty() {
        println!("No issues found.");
        return Ok(());
    }

    if args.raw {
        println!(
            "{}",
            serde_json::to_string_pretty(&issues_as_json(&issues))?
        );
        return Ok(());
    }

    if args.csv {
        println!("key,status,type,assignee,summary");
        for issue in issues {
            let assignee = issue.assignee.unwrap_or_else(|| "unassigned".to_owned());
            println!(
                "{},{},{},{},{}",
                csv_escape(&issue.key),
                csv_escape(&issue.status),
                csv_escape(&issue.issue_type),
                csv_escape(&assignee),
                csv_escape(&issue.summary)
            );
        }
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

pub(super) fn edit(args: EditIssueArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let fields = build_edit_issue_fields(&session, &args)?;

    session.client.update_issue(
        &session.site,
        &session.token,
        args.key.trim(),
        Value::Object(fields),
    )?;

    println!("Updated {}", args.key.trim());
    Ok(())
}

pub(super) fn assign(args: AssignIssueArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let assignee = resolve_assignee_id(&session, &args.user)?;

    session.client.assign_issue(
        &session.site,
        &session.token,
        args.key.trim(),
        assignee.as_deref(),
    )?;

    println!("Assigned {}", args.key.trim());
    Ok(())
}

pub(super) fn move_issue(args: MoveIssueArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;

    let transitions =
        session
            .client
            .list_transitions(&session.site, &session.token, args.key.trim())?;
    let transition = transitions
        .into_iter()
        .find(|transition| transition.name.eq_ignore_ascii_case(args.transition.trim()))
        .with_context(|| {
            format!(
                "transition {:?} not found for {}",
                args.transition.trim(),
                args.key.trim()
            )
        })?;

    let assignee = args
        .assignee
        .as_deref()
        .map(|value| resolve_assignee_id(&session, value))
        .transpose()?;
    let comment = args.comment.as_ref().map(|text| markdown_to_adf(text));

    session.client.transition_issue(
        &session.site,
        &session.token,
        args.key.trim(),
        &transition.id,
        comment,
        args.resolution.as_deref(),
        assignee.as_ref().map(|value| value.as_deref()),
    )?;

    println!("Moved {} to {}", args.key.trim(), transition.name);
    Ok(())
}

pub(super) fn link(args: LinkIssueArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;

    session.client.link_issues(
        &session.site,
        &session.token,
        args.key.trim(),
        args.other.trim(),
        args.relation.trim(),
    )?;

    println!(
        "Linked {} to {} as {}",
        args.key.trim(),
        args.other.trim(),
        args.relation.trim()
    );
    Ok(())
}

pub(super) fn add_remote_link(args: AddRemoteLinkArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;

    session.client.add_remote_link(
        &session.site,
        &session.token,
        args.key.trim(),
        args.url.trim(),
        args.title.trim(),
        args.summary.as_deref(),
    )?;

    println!("Added remote link to {}", args.key.trim());
    Ok(())
}

pub(super) fn unlink(args: UnlinkIssueArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;

    let links = session
        .client
        .list_issue_links(&session.site, &session.token, args.key.trim())?;
    let link = links
        .into_iter()
        .find(|link| link.other_key.eq_ignore_ascii_case(args.other.trim()))
        .with_context(|| {
            format!(
                "no link found between {} and {}",
                args.key.trim(),
                args.other.trim()
            )
        })?;
    session
        .client
        .delete_issue_link(&session.site, &session.token, &link.id)?;

    println!("Unlinked {} from {}", args.key.trim(), args.other.trim());
    Ok(())
}

pub(super) fn clone_issue(args: crate::cli::CloneIssueArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let source = session
        .client
        .get_clone_source(&session.site, &session.token, args.key.trim())?;

    let mut summary = args.summary.unwrap_or(source.summary);
    let replacements = parse_replacements(&args.replacements)?;
    for (from, to) in &replacements {
        summary = summary.replace(from, to);
    }

    let mut description = source.description;
    if let Some(value) = description.as_mut() {
        for (from, to) in &replacements {
            replace_text_nodes(value, from, to);
        }
    }

    let issue = session.client.create_issue(
        &session.site,
        &session.token,
        CreateIssueRequest {
            project_key: args.project.unwrap_or(source.project_key),
            issue_type: args.issue_type.unwrap_or(source.issue_type),
            summary,
            description,
            extra_fields: json!({
                "labels": if args.labels.is_empty() { source.labels } else { args.labels },
                "components": (if args.components.is_empty() { source.components } else { args.components })
                    .into_iter()
                    .map(|value| json!({ "name": value }))
                    .collect::<Vec<_>>(),
                "fixVersions": (if args.fix_versions.is_empty() { source.fix_versions } else { args.fix_versions })
                    .into_iter()
                    .map(|value| json!({ "name": value }))
                    .collect::<Vec<_>>(),
                "priority": args.priority.or(source.priority).map(|value| json!({ "name": value })),
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        },
    )?;

    println!("Cloned {} to {}", args.key.trim(), issue.key);
    Ok(())
}

pub(super) fn delete(args: DeleteIssueArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;

    session
        .client
        .delete_issue(&session.site, &session.token, args.key.trim(), args.cascade)?;
    println!("Deleted {}", args.key.trim());
    Ok(())
}

pub(super) fn watch(args: WatchIssueArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let me = session.client.me(&session.site, &session.token)?;

    session.client.set_watch(
        &session.site,
        &session.token,
        args.key.trim(),
        &me.account_id,
        args.remove,
    )?;
    println!("Updated watch state for {}", args.key.trim());
    Ok(())
}

pub(super) fn add_comment(args: AddCommentArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let body = read_required_text(
        args.body.as_ref(),
        args.template.as_ref(),
        "read comment body from stdin",
        "comment template",
        "comment body is required; pass it positionally or use --template",
    )?;
    let adf = markdown_to_adf(&body);
    let id = session.client.add_comment(
        &session.site,
        &session.token,
        args.key.trim(),
        adf,
        args.internal,
    )?;

    println!("Added comment {id} to {}", args.key.trim());
    Ok(())
}

pub(super) fn add_worklog(args: AddWorklogArgs) -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(args.site.as_deref())?;
    let comment = args.comment.as_ref().map(|text| markdown_to_adf(text));
    let id = session.client.add_worklog(
        &session.site,
        &session.token,
        args.key.trim(),
        args.time_spent.trim(),
        comment,
    )?;

    println!("Added worklog {id} to {}", args.key.trim());
    Ok(())
}

fn build_create_issue_request(
    app: &AppContext,
    session: &JiraSession,
    args: CreateIssueArgs,
) -> Result<CreateIssueRequest> {
    let project_key = app.project(&session.site.key, args.project.clone())?;
    let description = read_optional_text(
        args.description.as_ref(),
        args.description_file.as_ref(),
        "read description from stdin",
        "description file",
    )?
    .map(|markdown| markdown_to_adf(&markdown));

    let mut extra_fields = Map::new();
    if let Some(priority) = args.priority.as_ref() {
        extra_fields.insert("priority".into(), json!({ "name": priority.trim() }));
    }
    if !args.labels.is_empty() {
        extra_fields.insert("labels".into(), json!(args.labels));
    }
    if !args.components.is_empty() {
        extra_fields.insert(
            "components".into(),
            json!(
                args.components
                    .iter()
                    .map(|value| json!({ "name": value }))
                    .collect::<Vec<_>>()
            ),
        );
    }
    if !args.fix_versions.is_empty() {
        extra_fields.insert(
            "fixVersions".into(),
            json!(
                args.fix_versions
                    .iter()
                    .map(|value| json!({ "name": value }))
                    .collect::<Vec<_>>()
            ),
        );
    }
    if let Some(parent) = args.parent.as_ref() {
        extra_fields.insert("parent".into(), json!({ "key": parent.trim() }));
    }
    if let Some(assignee) = args.assignee.as_ref() {
        match resolve_assignee_id(session, assignee)? {
            Some(account_id) => {
                extra_fields.insert("assignee".into(), json!({ "accountId": account_id }));
            }
            None => {
                extra_fields.insert("assignee".into(), json!(null));
            }
        }
    }
    merge_raw_fields(&mut extra_fields, &args.fields, &args.field_json)?;

    Ok(CreateIssueRequest {
        project_key,
        issue_type: args.issue_type.trim().to_owned(),
        summary: args.summary.trim().to_owned(),
        description,
        extra_fields,
    })
}

fn build_issue_jql(
    app: &AppContext,
    session: &JiraSession,
    args: &ListIssuesArgs,
) -> Result<String> {
    let mut clauses = Vec::new();
    let project = app.project(&session.site.key, args.project.clone())?;
    clauses.push(format!("project = {project}"));

    if let Some(assignee) = args.assignee.as_ref() {
        let clause = if assignee.eq_ignore_ascii_case("me") {
            "assignee = currentUser()".to_owned()
        } else if assignee.eq_ignore_ascii_case("x")
            || assignee.eq_ignore_ascii_case("none")
            || assignee.eq_ignore_ascii_case("unassigned")
        {
            "assignee is EMPTY".to_owned()
        } else {
            format!("assignee = {}", jql_quote(assignee))
        };
        clauses.push(clause);
    }
    if let Some(reporter) = args.reporter.as_ref() {
        let clause = if reporter.eq_ignore_ascii_case("me") {
            "reporter = currentUser()".to_owned()
        } else {
            format!("reporter = {}", jql_quote(reporter))
        };
        clauses.push(clause);
    }
    if args.watching {
        clauses.push("watcher = currentUser()".to_owned());
    }
    if !args.statuses.is_empty() {
        clauses.push(format!("status IN ({})", join_jql_values(&args.statuses)));
    }
    if !args.issue_types.is_empty() {
        clauses.push(format!(
            "issuetype IN ({})",
            join_jql_values(&args.issue_types)
        ));
    }
    if !args.priorities.is_empty() {
        clauses.push(format!(
            "priority IN ({})",
            join_jql_values(&args.priorities)
        ));
    }
    if !args.labels.is_empty() {
        clauses.push(format!("labels IN ({})", join_jql_values(&args.labels)));
    }

    let direction = if args.reverse { "ASC" } else { "DESC" };
    Ok(format!(
        "{} ORDER BY {} {}",
        clauses.join(" AND "),
        args.order_by.trim(),
        direction
    ))
}

fn build_edit_issue_fields(
    session: &JiraSession,
    args: &EditIssueArgs,
) -> Result<Map<String, Value>> {
    let mut fields = Map::new();

    if let Some(summary) = &args.summary {
        fields.insert("summary".into(), Value::String(summary.trim().to_owned()));
    }
    if let Some(description) = read_optional_text(
        args.description.as_ref(),
        args.description_file.as_ref(),
        "read description from stdin",
        "description file",
    )? {
        fields.insert("description".into(), markdown_to_adf(&description));
    }
    if let Some(issue_type) = args.issue_type.as_ref() {
        fields.insert("issuetype".into(), json!({ "name": issue_type.trim() }));
    }
    if let Some(priority) = args.priority.as_ref() {
        fields.insert("priority".into(), json!({ "name": priority.trim() }));
    }
    if !args.labels.is_empty() {
        fields.insert("labels".into(), json!(args.labels));
    }
    if !args.components.is_empty() {
        fields.insert(
            "components".into(),
            json!(
                args.components
                    .iter()
                    .map(|value| json!({ "name": value }))
                    .collect::<Vec<_>>()
            ),
        );
    }
    if !args.fix_versions.is_empty() {
        fields.insert(
            "fixVersions".into(),
            json!(
                args.fix_versions
                    .iter()
                    .map(|value| json!({ "name": value }))
                    .collect::<Vec<_>>()
            ),
        );
    }
    if let Some(parent) = args.parent.as_ref() {
        fields.insert("parent".into(), json!({ "key": parent.trim() }));
    }
    if let Some(assignee) = args.assignee.as_ref() {
        match resolve_assignee_id(session, assignee)? {
            Some(account_id) => {
                fields.insert("assignee".into(), json!({ "accountId": account_id }));
            }
            None => {
                fields.insert("assignee".into(), json!(null));
            }
        }
    }

    merge_raw_fields(&mut fields, &args.fields, &args.field_json)?;

    if fields.is_empty() {
        bail!("no editable fields were provided");
    }

    Ok(fields)
}

fn join_jql_values(values: &[String]) -> String {
    values
        .iter()
        .map(|value| jql_quote(value))
        .collect::<Vec<_>>()
        .join(",")
}

fn jql_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn issues_as_json(issues: &[IssueSummary]) -> Value {
    Value::Array(
        issues
            .iter()
            .map(|issue| {
                json!({
                    "key": issue.key,
                    "status": issue.status,
                    "type": issue.issue_type,
                    "assignee": issue.assignee,
                    "summary": issue.summary,
                })
            })
            .collect(),
    )
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn parse_replacements(values: &[String]) -> Result<Vec<(String, String)>> {
    values
        .iter()
        .map(|value| {
            let (from, to) = value
                .split_once(':')
                .with_context(|| format!("expected from:to replacement, got {value:?}"))?;
            Ok((from.to_owned(), to.to_owned()))
        })
        .collect()
}

fn replace_text_nodes(value: &mut Value, from: &str, to: &str) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(text)) = map.get_mut("text") {
                *text = text.replace(from, to);
            }
            for nested in map.values_mut() {
                replace_text_nodes(nested, from, to);
            }
        }
        Value::Array(values) => {
            for nested in values {
                replace_text_nodes(nested, from, to);
            }
        }
        _ => {}
    }
}
