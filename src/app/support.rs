use std::{fs, io::Read, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};

use crate::{
    config::{Config, SiteProfile, site_key_from_input},
    jira_cloud::{JiraCloudClient, VerifiedSite},
    secret,
};

pub(super) const TOKEN_MANAGEMENT_URL: &str =
    "https://id.atlassian.com/manage-profile/security/api-tokens";
const SCOPED_TOKEN_SCOPES: &[&str] = &[
    "read:jira-user",
    "read:jira-work",
    "write:jira-work",
    "read:project:jira",
    "read:board-scope:jira-software",
    "read:sprint:jira-software",
    "write:sprint:jira-software",
    "read:epic:jira-software",
    "write:epic:jira-software",
    "read:issue-details:jira",
    "read:jql:jira",
];

pub(super) struct AppContext {
    pub(super) config: Config,
}

impl AppContext {
    pub(super) fn load() -> Result<Self> {
        Ok(Self {
            config: Config::load()?,
        })
    }

    pub(super) fn issue_site(&self, site: Option<&str>) -> Result<SiteProfile> {
        resolve_issue_site(&self.config, site)
    }

    pub(super) fn jira(&self, site: Option<&str>) -> Result<JiraSession> {
        JiraSession::from_site(self.issue_site(site)?)
    }

    pub(super) fn project(&self, site_key: &str, project: Option<String>) -> Result<String> {
        resolve_issue_project(&self.config, site_key, project)
    }
}

pub(super) struct JiraSession {
    pub(super) site: SiteProfile,
    pub(super) token: String,
    pub(super) client: JiraCloudClient,
}

impl JiraSession {
    pub(super) fn from_site(site: SiteProfile) -> Result<Self> {
        Ok(Self {
            token: secret::load_token(&site.key).context("load Jira token")?,
            client: JiraCloudClient::new()?,
            site,
        })
    }
}

pub(super) fn resolve_logout_site_key(config: &Config, site: Option<String>) -> Result<String> {
    match site {
        Some(value) => {
            if config.sites.contains_key(value.as_str()) {
                return Ok(value);
            }
            site_key_from_input(&value)
        }
        None => config
            .active_site
            .clone()
            .context("site is required when no active site is configured"),
    }
}

pub(super) fn resolve_context_site_key(config: &Config, site: Option<&str>) -> Result<String> {
    match site {
        Some(value) => {
            if config.sites.contains_key(value) {
                return Ok(value.to_owned());
            }
            let site_key = site_key_from_input(value)?;
            if config.sites.contains_key(&site_key) {
                Ok(site_key)
            } else {
                bail!("site {:?} not found; run `jira auth login` first", value);
            }
        }
        None => config
            .active_site
            .clone()
            .context("site is required when no active site is configured"),
    }
}

pub(super) fn normalize_optional_project(project: Option<String>) -> Option<String> {
    project
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| !value.is_empty())
}

pub(super) fn resolve_issue_site(config: &Config, site: Option<&str>) -> Result<SiteProfile> {
    let site_key = match site {
        Some(value) => resolve_context_site_key(config, Some(value))?,
        None => config
            .active_context
            .as_ref()
            .and_then(|name| config.contexts.get(name))
            .map(|context| context.site.clone())
            .or_else(|| config.active_site.clone())
            .context(
                "no active site configured; run `jira auth login` or `jira context use` first",
            )?,
    };

    config
        .sites
        .get(&site_key)
        .cloned()
        .with_context(|| format!("site {site_key:?} not found in configuration"))
}

pub(super) fn resolve_issue_project(
    config: &Config,
    site_key: &str,
    project: Option<String>,
) -> Result<String> {
    let project = normalize_optional_project(project).or_else(|| {
        config
            .active_context
            .as_ref()
            .and_then(|name| config.contexts.get(name))
            .filter(|context| context.site == site_key)
            .and_then(|context| context.project.clone())
    });

    project.context(
        "project is required when the active context does not define one; pass --project or create a context with --project",
    )
}

pub(super) fn read_optional_text(
    inline: Option<&String>,
    path: Option<&PathBuf>,
    stdin_context: &str,
    file_label: &str,
) -> Result<Option<String>> {
    match (inline, path) {
        (Some(text), None) => Ok(Some(text.clone())),
        (None, Some(path)) if path.as_os_str() == "-" => Ok(Some(read_stdin(stdin_context)?)),
        (None, Some(path)) => {
            Ok(Some(fs::read_to_string(path).with_context(|| {
                format!("read {file_label} {}", path.display())
            })?))
        }
        (None, None) => Ok(None),
        _ => Ok(None),
    }
}

pub(super) fn read_required_text(
    inline: Option<&String>,
    path: Option<&PathBuf>,
    stdin_context: &str,
    file_label: &str,
    missing_message: &str,
) -> Result<String> {
    read_optional_text(inline, path, stdin_context, file_label)?
        .with_context(|| missing_message.to_owned())
}

pub(super) fn merge_raw_fields(
    fields: &mut Map<String, Value>,
    raw_fields: &[String],
    raw_json_fields: &[String],
) -> Result<()> {
    for entry in raw_fields {
        let (key, value) = parse_key_value(entry)?;
        fields.insert(key, Value::String(value));
    }
    for entry in raw_json_fields {
        let (key, value) = parse_key_value(entry)?;
        let json_value: Value =
            serde_json::from_str(&value).with_context(|| format!("parse JSON field {key}"))?;
        fields.insert(key, json_value);
    }
    Ok(())
}

pub(super) fn resolve_assignee_id(session: &JiraSession, input: &str) -> Result<Option<String>> {
    let value = input.trim();
    if value.is_empty() {
        bail!("assignee cannot be empty");
    }

    if value.eq_ignore_ascii_case("me") {
        return Ok(Some(
            session.client.me(&session.site, &session.token)?.account_id,
        ));
    }
    if value.eq_ignore_ascii_case("x")
        || value.eq_ignore_ascii_case("none")
        || value.eq_ignore_ascii_case("unassigned")
    {
        return Ok(None);
    }
    if value.eq_ignore_ascii_case("default") {
        return Ok(Some("-1".to_owned()));
    }

    Ok(Some(value.to_owned()))
}

pub(super) fn print_web_instructions() {
    println!("Opening Atlassian API token management...");
    print_token_help();

    if let Err(err) = webbrowser::open(TOKEN_MANAGEMENT_URL) {
        println!("Failed to open browser: {err}");
        println!("Open this URL manually: {TOKEN_MANAGEMENT_URL}");
    }
}

pub(super) fn print_token_help() {
    println!();
    println!("How to get a Jira token:");
    println!("  1. Open {TOKEN_MANAGEMENT_URL}");
    println!("  2. Preferred: select \"Create API token with scopes\"");
    println!("  3. Choose the Jira app when Atlassian asks which app the token should access");
    println!("  4. Set an expiry date and create the token");
    println!("  5. Copy it immediately; Atlassian will not show it again");
    println!();
    println!("Scoped token permissions for full CLI coverage:");
    for scope in SCOPED_TOKEN_SCOPES {
        println!("  - {scope}");
    }
    println!();
    println!("Notes:");
    println!("  - `jira epic add` and `jira epic remove` require `write:epic:jira-software`.");
    println!(
        "  - The scope list above is based on the Jira platform and Jira Software endpoints this CLI currently calls."
    );
    println!(
        "  - Scoped tokens use api.atlassian.com under the hood; jira-cli detects that automatically after login."
    );
    println!(
        "  - If your Atlassian account UI does not support scoped tokens for your workflow yet, create a regular API token without scopes and use that instead."
    );
    println!("  - Basic auth uses your Atlassian account email plus the API token.");
    println!();
}

pub(super) fn print_login_success(verified: &VerifiedSite, site_was_known: bool) {
    let action = if site_was_known {
        "Updated"
    } else {
        "Logged in to"
    };

    println!(
        "{action} {} as {} ({})",
        verified.profile.site_url, verified.user.display_name, verified.user.account_id
    );
    println!("API base: {}", verified.profile.api_base_url);
    println!("Token mode: {}", verified.token_mode.label());
}

fn parse_key_value(input: &str) -> Result<(String, String)> {
    let (key, value) = input
        .split_once('=')
        .with_context(|| format!("expected key=value, got {input:?}"))?;
    let key = key.trim();
    if key.is_empty() {
        bail!("field key cannot be empty");
    }
    Ok((key.to_owned(), value.to_owned()))
}

fn read_stdin(context: &str) -> Result<String> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .with_context(|| context.to_owned())?;
    Ok(input)
}
