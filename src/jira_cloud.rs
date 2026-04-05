use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use reqwest::{
    StatusCode,
    blocking::{Client, Response},
    header::{ACCEPT, USER_AGENT},
};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use url::form_urlencoded;

use crate::config::{SiteProfile, normalize_site_url, site_key_from_url};

const API_GATEWAY_ROOT: &str = "https://api.atlassian.com/ex/jira";

#[derive(Debug, Clone)]
pub struct JiraCloudClient {
    http: Client,
}

#[derive(Debug, Clone)]
pub struct VerifiedSite {
    pub profile: SiteProfile,
    pub user: JiraUser,
    pub token_mode: TokenMode,
}

#[derive(Debug, Clone)]
pub struct JiraUser {
    pub account_id: String,
    pub display_name: String,
    pub email_address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateIssueRequest {
    pub project_key: String,
    pub issue_type: String,
    pub summary: String,
    pub description: Option<Value>,
    pub extra_fields: Map<String, Value>,
}

#[derive(Debug, Clone)]
pub struct CreatedIssue {
    pub id: String,
    pub key: String,
    pub self_url: String,
}

#[derive(Debug, Clone)]
pub struct IssueSummary {
    pub key: String,
    pub summary: String,
    pub status: String,
    pub issue_type: String,
    pub assignee: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IssueDetails {
    pub key: String,
    pub summary: String,
    pub status: String,
    pub issue_type: String,
    pub project_key: String,
    pub assignee: Option<String>,
    pub reporter: Option<String>,
    pub created: String,
    pub updated: String,
    pub description: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct IssueComment {
    pub author: String,
    pub created: String,
    pub body: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct ProjectSummary {
    pub id: String,
    pub key: String,
    pub name: String,
    pub project_type: String,
}

#[derive(Debug, Clone)]
pub struct BoardSummary {
    pub id: u64,
    pub name: String,
    pub board_type: String,
    pub project_key: Option<String>,
    pub project_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReleaseSummary {
    pub id: String,
    pub name: String,
    pub released: bool,
    pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub deployment_type: String,
    pub version: String,
    pub build_number: i64,
    pub server_title: String,
}

#[derive(Debug, Clone)]
pub struct IssueLinkRef {
    pub id: String,
    pub other_key: String,
}

#[derive(Debug, Clone)]
pub struct TransitionSummary {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SprintSummary {
    pub id: u64,
    pub name: String,
    pub state: String,
}

#[derive(Debug, Clone)]
pub struct CloneSourceIssue {
    pub summary: String,
    pub description: Option<Value>,
    pub labels: Vec<String>,
    pub components: Vec<String>,
    pub fix_versions: Vec<String>,
    pub priority: Option<String>,
    pub project_key: String,
    pub issue_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenMode {
    SiteBasic,
    ScopedGateway,
}

impl TokenMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::SiteBasic => "site API token",
            Self::ScopedGateway => "scoped API token via api.atlassian.com",
        }
    }
}

#[derive(Debug, Deserialize)]
struct JiraMyselfResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "emailAddress")]
    email_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TenantInfoResponse {
    #[serde(rename = "cloudId")]
    cloud_id: String,
}

#[derive(Debug, Deserialize)]
struct CreateIssueResponse {
    id: String,
    key: String,
    #[serde(rename = "self")]
    self_url: String,
}

#[derive(Debug, Deserialize)]
struct SearchIssuesResponse {
    issues: Vec<ApiIssue>,
}

#[derive(Debug, Deserialize)]
struct ProjectSearchResponse {
    #[serde(default)]
    values: Vec<ProjectSearchItem>,
}

#[derive(Debug, Deserialize)]
struct ProjectSearchItem {
    id: String,
    key: String,
    name: String,
    #[serde(rename = "projectTypeKey", default)]
    project_type_key: String,
}

#[derive(Debug, Deserialize)]
struct BoardSearchResponse {
    #[serde(default)]
    values: Vec<ApiBoard>,
}

#[derive(Debug, Deserialize)]
struct ApiBoard {
    id: u64,
    name: String,
    #[serde(rename = "type")]
    board_type: String,
    location: Option<ApiBoardLocation>,
}

#[derive(Debug, Deserialize)]
struct ApiBoardLocation {
    #[serde(rename = "projectKey")]
    project_key: Option<String>,
    #[serde(rename = "projectName")]
    project_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiRelease {
    id: String,
    name: String,
    #[serde(default)]
    released: bool,
    #[serde(default)]
    archived: bool,
}

#[derive(Debug, Deserialize)]
struct ApiServerInfo {
    #[serde(rename = "deploymentType", default)]
    deployment_type: String,
    #[serde(default)]
    version: String,
    #[serde(rename = "buildNumber", default)]
    build_number: i64,
    #[serde(rename = "serverTitle", default)]
    server_title: String,
}

#[derive(Debug, Deserialize)]
struct ApiIssue {
    key: String,
    fields: ApiIssueFields,
}

#[derive(Debug, Deserialize)]
struct ApiIssueFields {
    summary: String,
    status: NamedField,
    #[serde(rename = "issuetype")]
    issue_type: NamedField,
    project: Option<ProjectField>,
    assignee: Option<UserField>,
    reporter: Option<UserField>,
    created: Option<String>,
    updated: Option<String>,
    description: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct NamedField {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ProjectField {
    key: String,
}

#[derive(Debug, Deserialize)]
struct UserField {
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct ApiIssueLink {
    id: String,
    #[serde(rename = "outwardIssue")]
    outward_issue: Option<ApiLinkedIssueRef>,
    #[serde(rename = "inwardIssue")]
    inward_issue: Option<ApiLinkedIssueRef>,
}

#[derive(Debug, Deserialize)]
struct ApiLinkedIssueRef {
    key: String,
}

#[derive(Debug, Deserialize)]
struct ApiCommentResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ApiCommentsResponse {
    #[serde(default)]
    comments: Vec<ApiComment>,
}

#[derive(Debug, Deserialize)]
struct ApiComment {
    #[serde(default)]
    author: Option<UserField>,
    #[serde(default)]
    created: Option<String>,
    #[serde(default)]
    body: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ApiTransitionsResponse {
    #[serde(default)]
    transitions: Vec<ApiTransition>,
}

#[derive(Debug, Deserialize)]
struct ApiSprintsResponse {
    #[serde(default)]
    values: Vec<ApiSprint>,
}

#[derive(Debug, Deserialize)]
struct ApiCloneIssue {
    fields: ApiCloneIssueFields,
}

#[derive(Debug, Deserialize)]
struct ApiCloneIssueFields {
    summary: String,
    description: Option<Value>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    components: Vec<ApiNamedValue>,
    #[serde(rename = "fixVersions", default)]
    fix_versions: Vec<ApiNamedValue>,
    priority: Option<NamedField>,
    project: ProjectField,
    #[serde(rename = "issuetype")]
    issue_type: NamedField,
}

#[derive(Debug, Deserialize)]
struct ApiNamedValue {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ApiSprint {
    id: u64,
    name: String,
    state: String,
}

#[derive(Debug, Deserialize)]
struct ApiTransition {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ApiIssueLinksResponse {
    fields: ApiIssueLinksFields,
}

#[derive(Debug, Deserialize)]
struct ApiIssueLinksFields {
    #[serde(default)]
    issuelinks: Vec<ApiIssueLink>,
}

impl JiraCloudClient {
    pub fn new() -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .context("build HTTP client")?;

        Ok(Self { http })
    }

    pub fn verify_credentials(
        &self,
        site_url: &str,
        email: &str,
        token: &str,
    ) -> Result<VerifiedSite> {
        let site_url = normalize_site_url(site_url)?;
        let site_key = site_key_from_url(&site_url)?;

        let direct_attempt = self.fetch_myself(&site_url, email, token);
        let cloud_id = self.fetch_cloud_id(&site_url).ok();
        let scoped_attempt = cloud_id.as_ref().map(|cloud_id| {
            let api_base_url = format!("{API_GATEWAY_ROOT}/{cloud_id}");
            self.fetch_myself(&api_base_url, email, token)
        });

        resolve_verified_site(
            site_key,
            site_url,
            email,
            cloud_id,
            direct_attempt,
            scoped_attempt,
        )
    }

    fn fetch_myself(&self, api_base_url: &str, email: &str, token: &str) -> Result<JiraUser> {
        let response = self
            .http
            .get(format!(
                "{}/rest/api/3/myself",
                api_base_url.trim_end_matches('/')
            ))
            .basic_auth(email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("request Jira profile from {api_base_url}"))?;

        let response = ensure_success(response, api_base_url)?;
        let payload: JiraMyselfResponse = response
            .json()
            .with_context(|| format!("decode Jira profile response from {api_base_url}"))?;

        Ok(JiraUser {
            account_id: payload.account_id,
            display_name: payload.display_name,
            email_address: payload.email_address,
        })
    }

    fn fetch_cloud_id(&self, site_url: &str) -> Result<String> {
        let response = self
            .http
            .get(format!(
                "{}/_edge/tenant_info",
                site_url.trim_end_matches('/')
            ))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("request tenant info from {site_url}"))?;

        let response = ensure_success(response, site_url)?;
        let payload: TenantInfoResponse = response
            .json()
            .with_context(|| format!("decode tenant info response from {site_url}"))?;

        if payload.cloud_id.trim().is_empty() {
            bail!("tenant info response did not include a cloud id");
        }

        Ok(payload.cloud_id)
    }

    pub fn create_issue(
        &self,
        site: &SiteProfile,
        token: &str,
        request: CreateIssueRequest,
    ) -> Result<CreatedIssue> {
        let mut fields = Map::new();
        fields.insert("project".into(), json!({ "key": request.project_key }));
        fields.insert("issuetype".into(), json!({ "name": request.issue_type }));
        fields.insert("summary".into(), Value::String(request.summary));
        if let Some(description) = request.description {
            fields.insert("description".into(), description);
        }
        fields.extend(request.extra_fields);

        let response = self
            .http
            .post(format!(
                "{}/rest/api/3/issue",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&json!({ "fields": fields }))
            .send()
            .with_context(|| format!("create issue in project {}", site.site_url))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: CreateIssueResponse = response
            .json()
            .with_context(|| format!("decode create issue response from {}", site.api_base_url))?;

        Ok(CreatedIssue {
            id: payload.id,
            key: payload.key,
            self_url: payload.self_url,
        })
    }

    pub fn get_issue(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
    ) -> Result<IssueDetails> {
        let response = self
            .http
            .get(format!(
                "{}/rest/api/3/issue/{}?fields=summary,status,issuetype,project,assignee,reporter,created,updated,description",
                site.api_base_url.trim_end_matches('/'),
                issue_key
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("fetch issue {issue_key} from {}", site.site_url))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: ApiIssue = response
            .json()
            .with_context(|| format!("decode issue response for {issue_key}"))?;

        Ok(IssueDetails {
            key: payload.key,
            summary: payload.fields.summary,
            status: payload.fields.status.name,
            issue_type: payload.fields.issue_type.name,
            project_key: payload
                .fields
                .project
                .map(|project| project.key)
                .ok_or_else(|| anyhow!("issue response for {issue_key} did not include project"))?,
            assignee: payload.fields.assignee.map(|user| user.display_name),
            reporter: payload.fields.reporter.map(|user| user.display_name),
            created: payload.fields.created.unwrap_or_default(),
            updated: payload.fields.updated.unwrap_or_default(),
            description: payload.fields.description,
        })
    }

    pub fn list_issue_comments(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
        limit: u32,
    ) -> Result<Vec<IssueComment>> {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("maxResults", &limit.to_string())
            .append_pair("orderBy", "-created")
            .finish();

        let response = self
            .http
            .get(format!(
                "{}/rest/api/3/issue/{issue_key}/comment?{}",
                site.api_base_url.trim_end_matches('/'),
                query
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("list comments for {issue_key}"))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: ApiCommentsResponse = response
            .json()
            .with_context(|| format!("decode comment list response for {issue_key}"))?;

        Ok(payload
            .comments
            .into_iter()
            .map(|comment| IssueComment {
                author: comment
                    .author
                    .map(|author| author.display_name)
                    .unwrap_or_else(|| "Unknown".to_owned()),
                created: comment.created.unwrap_or_default(),
                body: comment.body,
            })
            .collect())
    }

    pub fn list_issues(
        &self,
        site: &SiteProfile,
        token: &str,
        jql: &str,
        limit: u32,
    ) -> Result<Vec<IssueSummary>> {
        let fields = "summary,status,issuetype,assignee";
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("jql", jql)
            .append_pair("maxResults", &limit.to_string())
            .append_pair("fields", fields)
            .finish();

        let response = self
            .http
            .get(format!(
                "{}/rest/api/3/search/jql?{}",
                site.api_base_url.trim_end_matches('/'),
                query
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("search issues on {}", site.site_url))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: SearchIssuesResponse = response
            .json()
            .with_context(|| format!("decode search response from {}", site.site_url))?;

        Ok(payload
            .issues
            .into_iter()
            .map(|issue| IssueSummary {
                key: issue.key,
                summary: issue.fields.summary,
                status: issue.fields.status.name,
                issue_type: issue.fields.issue_type.name,
                assignee: issue.fields.assignee.map(|user| user.display_name),
            })
            .collect())
    }

    pub fn me(&self, site: &SiteProfile, token: &str) -> Result<JiraUser> {
        self.fetch_myself(&site.api_base_url, &site.email, token)
    }

    pub fn list_projects(&self, site: &SiteProfile, token: &str) -> Result<Vec<ProjectSummary>> {
        let response = self
            .http
            .get(format!(
                "{}/rest/api/3/project/search",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("list projects on {}", site.site_url))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: ProjectSearchResponse = response
            .json()
            .with_context(|| format!("decode project list response from {}", site.site_url))?;

        Ok(payload
            .values
            .into_iter()
            .map(|project| ProjectSummary {
                id: project.id,
                key: project.key,
                name: project.name,
                project_type: project.project_type_key,
            })
            .collect())
    }

    pub fn list_boards(
        &self,
        site: &SiteProfile,
        token: &str,
        project: Option<&str>,
    ) -> Result<Vec<BoardSummary>> {
        let mut query = form_urlencoded::Serializer::new(String::new());
        if let Some(project) = project {
            query.append_pair("projectKeyOrId", project);
        }
        let query = query.finish();
        let url = if query.is_empty() {
            format!(
                "{}/rest/agile/1.0/board",
                site.api_base_url.trim_end_matches('/')
            )
        } else {
            format!(
                "{}/rest/agile/1.0/board?{}",
                site.api_base_url.trim_end_matches('/'),
                query
            )
        };

        let response = self
            .http
            .get(url)
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("list boards on {}", site.site_url))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: BoardSearchResponse = response
            .json()
            .with_context(|| format!("decode board list response from {}", site.site_url))?;

        Ok(payload
            .values
            .into_iter()
            .map(|board| BoardSummary {
                id: board.id,
                name: board.name,
                board_type: board.board_type,
                project_key: board
                    .location
                    .as_ref()
                    .and_then(|value| value.project_key.clone()),
                project_name: board.location.and_then(|value| value.project_name),
            })
            .collect())
    }

    pub fn list_releases(
        &self,
        site: &SiteProfile,
        token: &str,
        project: &str,
    ) -> Result<Vec<ReleaseSummary>> {
        let response = self
            .http
            .get(format!(
                "{}/rest/api/3/project/{}/versions",
                site.api_base_url.trim_end_matches('/'),
                project
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("list releases for project {project}"))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: Vec<ApiRelease> = response
            .json()
            .with_context(|| format!("decode release list response from {}", site.site_url))?;

        Ok(payload
            .into_iter()
            .map(|release| ReleaseSummary {
                id: release.id,
                name: release.name,
                released: release.released,
                archived: release.archived,
            })
            .collect())
    }

    pub fn server_info(&self, site: &SiteProfile, token: &str) -> Result<ServerInfo> {
        let response = self
            .http
            .get(format!(
                "{}/rest/api/3/serverInfo",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("fetch server info from {}", site.site_url))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: ApiServerInfo = response
            .json()
            .with_context(|| format!("decode server info response from {}", site.site_url))?;

        Ok(ServerInfo {
            deployment_type: payload.deployment_type,
            version: payload.version,
            build_number: payload.build_number,
            server_title: payload.server_title,
        })
    }

    pub fn assign_issue(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
        account_id: Option<&str>,
    ) -> Result<()> {
        let payload = match account_id {
            Some(account_id) => json!({ "accountId": account_id }),
            None => json!({ "accountId": Value::Null }),
        };

        let response = self
            .http
            .put(format!(
                "{}/rest/api/3/issue/{issue_key}/assignee",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&payload)
            .send()
            .with_context(|| format!("assign issue {issue_key}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn update_issue(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
        fields: Value,
    ) -> Result<()> {
        let response = self
            .http
            .put(format!(
                "{}/rest/api/3/issue/{issue_key}",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&json!({ "fields": fields }))
            .send()
            .with_context(|| format!("update issue {issue_key}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn add_comment(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
        body: Value,
        internal: bool,
    ) -> Result<String> {
        let mut payload = json!({ "body": body });
        if internal {
            payload["properties"] = json!([
                {
                    "key": "sd.public.comment",
                    "value": { "internal": true }
                }
            ]);
        }

        let response = self
            .http
            .post(format!(
                "{}/rest/api/3/issue/{issue_key}/comment",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&payload)
            .send()
            .with_context(|| format!("add comment to {issue_key}"))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: ApiCommentResponse = response
            .json()
            .with_context(|| format!("decode comment response for {issue_key}"))?;
        Ok(payload.id)
    }

    pub fn list_transitions(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
    ) -> Result<Vec<TransitionSummary>> {
        let response = self
            .http
            .get(format!(
                "{}/rest/api/3/issue/{issue_key}/transitions",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("list transitions for {issue_key}"))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: ApiTransitionsResponse = response
            .json()
            .with_context(|| format!("decode transitions response for {issue_key}"))?;

        Ok(payload
            .transitions
            .into_iter()
            .map(|transition| TransitionSummary {
                id: transition.id,
                name: transition.name,
            })
            .collect())
    }

    pub fn transition_issue(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
        transition_id: &str,
        comment: Option<Value>,
        resolution: Option<&str>,
        assignee_account_id: Option<Option<&str>>,
    ) -> Result<()> {
        let mut payload = json!({
            "transition": { "id": transition_id }
        });

        if comment.is_some() || resolution.is_some() || assignee_account_id.is_some() {
            payload["fields"] = json!({});
        }
        if let Some(resolution) = resolution {
            payload["fields"]["resolution"] = json!({ "name": resolution });
        }
        if let Some(account_id) = assignee_account_id {
            payload["fields"]["assignee"] = match account_id {
                Some(account_id) => json!({ "accountId": account_id }),
                None => Value::Null,
            };
        }
        if let Some(comment) = comment {
            payload["update"] = json!({
                "comment": [
                    {
                        "add": { "body": comment }
                    }
                ]
            });
        }

        let response = self
            .http
            .post(format!(
                "{}/rest/api/3/issue/{issue_key}/transitions",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&payload)
            .send()
            .with_context(|| format!("transition issue {issue_key}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn delete_issue(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
        cascade: bool,
    ) -> Result<()> {
        let mut query = form_urlencoded::Serializer::new(String::new());
        if cascade {
            query.append_pair("deleteSubtasks", "true");
        }
        let query = query.finish();
        let url = if query.is_empty() {
            format!(
                "{}/rest/api/3/issue/{issue_key}",
                site.api_base_url.trim_end_matches('/')
            )
        } else {
            format!(
                "{}/rest/api/3/issue/{issue_key}?{}",
                site.api_base_url.trim_end_matches('/'),
                query
            )
        };

        let response = self
            .http
            .delete(url)
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("delete issue {issue_key}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn add_worklog(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
        time_spent: &str,
        comment: Option<Value>,
    ) -> Result<String> {
        let mut payload = json!({
            "timeSpent": time_spent
        });
        if let Some(comment) = comment {
            payload["comment"] = comment;
        }

        let response = self
            .http
            .post(format!(
                "{}/rest/api/3/issue/{issue_key}/worklog",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&payload)
            .send()
            .with_context(|| format!("add worklog to {issue_key}"))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: ApiCommentResponse = response
            .json()
            .with_context(|| format!("decode worklog response for {issue_key}"))?;
        Ok(payload.id)
    }

    pub fn list_sprints(
        &self,
        site: &SiteProfile,
        token: &str,
        board_id: u64,
    ) -> Result<Vec<SprintSummary>> {
        let response = self
            .http
            .get(format!(
                "{}/rest/agile/1.0/board/{board_id}/sprint?maxResults=50",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("list sprints for board {board_id}"))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: ApiSprintsResponse = response
            .json()
            .with_context(|| format!("decode sprint list response for board {board_id}"))?;

        Ok(payload
            .values
            .into_iter()
            .map(|sprint| SprintSummary {
                id: sprint.id,
                name: sprint.name,
                state: sprint.state,
            })
            .collect())
    }

    pub fn list_epic_issues(
        &self,
        site: &SiteProfile,
        token: &str,
        epic_key: &str,
        limit: u32,
    ) -> Result<Vec<IssueSummary>> {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("maxResults", &limit.to_string())
            .append_pair("fields", "summary,status,issuetype,assignee")
            .finish();

        let response = self
            .http
            .get(format!(
                "{}/rest/agile/1.0/epic/{epic_key}/issue?{}",
                site.api_base_url.trim_end_matches('/'),
                query
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("list issues in epic {epic_key}"))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: SearchIssuesResponse = response
            .json()
            .with_context(|| format!("decode epic issues response for {epic_key}"))?;

        Ok(payload
            .issues
            .into_iter()
            .map(|issue| IssueSummary {
                key: issue.key,
                summary: issue.fields.summary,
                status: issue.fields.status.name,
                issue_type: issue.fields.issue_type.name,
                assignee: issue.fields.assignee.map(|user| user.display_name),
            })
            .collect())
    }

    pub fn add_issues_to_sprint(
        &self,
        site: &SiteProfile,
        token: &str,
        sprint_id: u64,
        issues: &[String],
    ) -> Result<()> {
        let response = self
            .http
            .post(format!(
                "{}/rest/agile/1.0/sprint/{sprint_id}/issue",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&json!({ "issues": issues }))
            .send()
            .with_context(|| format!("add issues to sprint {sprint_id}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn close_sprint(&self, site: &SiteProfile, token: &str, sprint_id: u64) -> Result<()> {
        let response = self
            .http
            .put(format!(
                "{}/rest/agile/1.0/sprint/{sprint_id}",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&json!({ "state": "closed" }))
            .send()
            .with_context(|| format!("close sprint {sprint_id}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn add_issues_to_epic(
        &self,
        site: &SiteProfile,
        token: &str,
        epic_key: &str,
        issues: &[String],
    ) -> Result<()> {
        let response = self
            .http
            .post(format!(
                "{}/rest/agile/1.0/epic/{epic_key}/issue",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&json!({ "issues": issues }))
            .send()
            .with_context(|| format!("add issues to epic {epic_key}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn remove_issues_from_epic(
        &self,
        site: &SiteProfile,
        token: &str,
        issues: &[String],
    ) -> Result<()> {
        let response = self
            .http
            .post(format!(
                "{}/rest/agile/1.0/epic/none/issue",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&json!({ "issues": issues }))
            .send()
            .with_context(|| "remove issues from epic".to_owned())?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn link_issues(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
        other_key: &str,
        relation: &str,
    ) -> Result<()> {
        let response = self
            .http
            .post(format!(
                "{}/rest/api/3/issueLink",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&json!({
                "type": { "name": relation },
                "outwardIssue": { "key": issue_key },
                "inwardIssue": { "key": other_key }
            }))
            .send()
            .with_context(|| format!("link issue {issue_key} to {other_key}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn add_remote_link(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
        url: &str,
        title: &str,
        summary: Option<&str>,
    ) -> Result<()> {
        let mut object = json!({
            "url": url,
            "title": title,
        });
        if let Some(summary) = summary {
            object["summary"] = Value::String(summary.to_owned());
        }

        let response = self
            .http
            .post(format!(
                "{}/rest/api/3/issue/{issue_key}/remotelink",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .json(&json!({ "object": object }))
            .send()
            .with_context(|| format!("add remote link to {issue_key}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn list_issue_links(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
    ) -> Result<Vec<IssueLinkRef>> {
        let response = self
            .http
            .get(format!(
                "{}/rest/api/3/issue/{issue_key}?fields=issuelinks",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("fetch issue links for {issue_key}"))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: ApiIssueLinksResponse = response
            .json()
            .with_context(|| format!("decode issue links response for {issue_key}"))?;

        Ok(payload
            .fields
            .issuelinks
            .into_iter()
            .filter_map(|link| {
                let other = link
                    .outward_issue
                    .map(|issue| issue.key)
                    .or_else(|| link.inward_issue.map(|issue| issue.key));
                other.map(|other_key| IssueLinkRef {
                    id: link.id,
                    other_key,
                })
            })
            .collect())
    }

    pub fn delete_issue_link(&self, site: &SiteProfile, token: &str, link_id: &str) -> Result<()> {
        let response = self
            .http
            .delete(format!(
                "{}/rest/api/3/issueLink/{link_id}",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("delete issue link {link_id}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn set_watch(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
        account_id: &str,
        remove: bool,
    ) -> Result<()> {
        let url = if remove {
            format!(
                "{}/rest/api/3/issue/{issue_key}/watchers?accountId={}",
                site.api_base_url.trim_end_matches('/'),
                form_urlencoded::byte_serialize(account_id.as_bytes()).collect::<String>()
            )
        } else {
            format!(
                "{}/rest/api/3/issue/{issue_key}/watchers",
                site.api_base_url.trim_end_matches('/')
            )
        };

        let request = if remove {
            self.http.delete(url)
        } else {
            self.http
                .post(url)
                .json(&Value::String(account_id.to_owned()))
        };

        let response = request
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("update watch state for {issue_key}"))?;

        let _ = ensure_success(response, &site.api_base_url)?;
        Ok(())
    }

    pub fn get_clone_source(
        &self,
        site: &SiteProfile,
        token: &str,
        issue_key: &str,
    ) -> Result<CloneSourceIssue> {
        let response = self
            .http
            .get(format!(
                "{}/rest/api/3/issue/{issue_key}?fields=summary,description,labels,components,fixVersions,priority,project,issuetype,assignee,parent",
                site.api_base_url.trim_end_matches('/')
            ))
            .basic_auth(&site.email, Some(token))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, concat!("jira-cli/", env!("CARGO_PKG_VERSION")))
            .send()
            .with_context(|| format!("fetch clone source for {issue_key}"))?;

        let response = ensure_success(response, &site.api_base_url)?;
        let payload: ApiCloneIssue = response
            .json()
            .with_context(|| format!("decode clone source response for {issue_key}"))?;

        Ok(CloneSourceIssue {
            summary: payload.fields.summary,
            description: payload.fields.description,
            labels: payload.fields.labels,
            components: payload
                .fields
                .components
                .into_iter()
                .map(|value| value.name)
                .collect(),
            fix_versions: payload
                .fields
                .fix_versions
                .into_iter()
                .map(|value| value.name)
                .collect(),
            priority: payload.fields.priority.map(|value| value.name),
            project_key: payload.fields.project.key,
            issue_type: payload.fields.issue_type.name,
        })
    }
}

fn resolve_verified_site(
    site_key: String,
    site_url: String,
    email: &str,
    cloud_id: Option<String>,
    direct_attempt: Result<JiraUser>,
    scoped_attempt: Option<Result<JiraUser>>,
) -> Result<VerifiedSite> {
    if let Ok(user) = direct_attempt {
        return Ok(VerifiedSite {
            profile: SiteProfile {
                key: site_key,
                site_url: site_url.clone(),
                api_base_url: site_url,
                email: email.to_owned(),
                cloud_id,
            },
            user,
            token_mode: TokenMode::SiteBasic,
        });
    }

    if let Some(cloud_id) = cloud_id {
        return match scoped_attempt {
            Some(Ok(user)) => Ok(VerifiedSite {
                profile: SiteProfile {
                    key: site_key,
                    site_url,
                    api_base_url: format!("{API_GATEWAY_ROOT}/{cloud_id}"),
                    email: email.to_owned(),
                    cloud_id: Some(cloud_id),
                },
                user,
                token_mode: TokenMode::ScopedGateway,
            }),
            Some(Err(err)) => Err(err),
            None => unreachable!("scoped attempt should exist when a cloud id is present"),
        };
    }

    match direct_attempt {
        Ok(_) => unreachable!("successful direct auth should have returned early"),
        Err(err) => Err(err),
    }
}

fn ensure_success(response: Response, api_base_url: &str) -> Result<Response> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = response.text().unwrap_or_default();
    let detail = compact_body(&body);

    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        if detail.to_ascii_lowercase().contains("scope does not match") {
            bail!(
                "authentication failed against {api_base_url} ({status}): {detail}. This usually means the token is missing required scopes for this Jira API. Jira Software commands such as boards, sprints, and epics need additional Jira Software scopes."
            );
        }
        bail!(
            "authentication failed against {api_base_url} ({status}): {detail}. Check the site URL, email, and whether the API token is scoped or unscoped"
        );
    }

    Err(anyhow!(
        "Jira API request to {api_base_url} failed with {status}: {detail}"
    ))
}

fn compact_body(body: &str) -> String {
    let compact = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = compact.trim();

    if trimmed.is_empty() {
        "empty response body".to_owned()
    } else if trimmed.len() > 200 {
        format!("{}...", &trimmed[..200])
    } else {
        trimmed.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::{JiraUser, TokenMode, compact_body, resolve_verified_site};

    #[test]
    fn compact_body_trims_and_truncates() {
        let body = "  a  b  c  ".repeat(80);
        let compact = compact_body(&body);
        assert!(compact.starts_with("a b c"));
        assert!(compact.ends_with("..."));
    }

    #[test]
    fn token_mode_labels_are_stable() {
        assert_eq!(TokenMode::SiteBasic.label(), "site API token");
        assert_eq!(
            TokenMode::ScopedGateway.label(),
            "scoped API token via api.atlassian.com"
        );
    }

    #[test]
    fn scoped_auth_error_is_preferred_over_direct_auth_error() {
        let direct_error = anyhow!("direct auth failed");
        let scoped_error = anyhow!("scope does not match");

        let err = resolve_verified_site(
            "example.atlassian.net".into(),
            "https://example.atlassian.net".into(),
            "user@example.com",
            Some("cloud-123".into()),
            Err(direct_error),
            Some(Err(scoped_error)),
        )
        .expect_err("scoped auth should fail");

        assert!(err.to_string().contains("scope does not match"));
    }

    #[test]
    fn scoped_auth_success_returns_gateway_profile() {
        let user = JiraUser {
            account_id: "acct-123".into(),
            display_name: "Faraz".into(),
            email_address: Some("user@example.com".into()),
        };

        let verified = resolve_verified_site(
            "example.atlassian.net".into(),
            "https://example.atlassian.net".into(),
            "user@example.com",
            Some("cloud-123".into()),
            Err(anyhow!("direct auth failed")),
            Some(Ok(user)),
        )
        .expect("scoped auth should succeed");

        assert_eq!(verified.token_mode, TokenMode::ScopedGateway);
        assert_eq!(
            verified.profile.api_base_url,
            "https://api.atlassian.com/ex/jira/cloud-123"
        );
    }
}
