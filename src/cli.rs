use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "jira",
    version,
    about = "Jira CLI for authentication-first workflows",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Auth(AuthArgs),
    Context(ContextArgs),
    Issue(IssueArgs),
    Epic(EpicArgs),
    Board(BoardArgs),
    Project(ProjectArgs),
    Release(ReleaseArgs),
    Sprint(SprintArgs),
    Me,
    Open(OpenArgs),
    #[command(name = "serverinfo")]
    ServerInfo,
    Init(InitArgs),
    Completion(CompletionArgs),
    Man(ManArgs),
    Version,
}

#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    Login(LoginArgs),
    Status,
    Logout(LogoutArgs),
}

#[derive(Debug, Args)]
pub struct ContextArgs {
    #[command(subcommand)]
    pub command: ContextCommand,
}

#[derive(Debug, Subcommand)]
pub enum ContextCommand {
    Create(CreateContextArgs),
    Use(UseContextArgs),
    List,
    Delete(DeleteContextArgs),
}

#[derive(Debug, Args)]
pub struct IssueArgs {
    #[command(subcommand)]
    pub command: IssueCommand,
}

#[derive(Debug, Subcommand)]
pub enum IssueCommand {
    Create(CreateIssueArgs),
    View(ViewIssueArgs),
    List(ListIssuesArgs),
    Edit(EditIssueArgs),
    Assign(AssignIssueArgs),
    Move(MoveIssueArgs),
    Link(LinkIssueArgs),
    #[command(subcommand)]
    RemoteLink(RemoteLinkCommand),
    Unlink(UnlinkIssueArgs),
    Clone(CloneIssueArgs),
    Delete(DeleteIssueArgs),
    Watch(WatchIssueArgs),
    Comment(CommentArgs),
    Worklog(WorklogArgs),
}

#[derive(Debug, Args)]
pub struct BoardArgs {
    #[command(subcommand)]
    pub command: BoardCommand,
}

#[derive(Debug, Subcommand)]
pub enum BoardCommand {
    List(ListBoardsArgs),
}

#[derive(Debug, Args)]
pub struct ProjectArgs {
    #[command(subcommand)]
    pub command: ProjectCommand,
}

#[derive(Debug, Subcommand)]
pub enum ProjectCommand {
    List(ListProjectsArgs),
}

#[derive(Debug, Args)]
pub struct ReleaseArgs {
    #[command(subcommand)]
    pub command: ReleaseCommand,
}

#[derive(Debug, Subcommand)]
pub enum ReleaseCommand {
    List(ListReleasesArgs),
}

#[derive(Debug, Args)]
pub struct SprintArgs {
    #[command(subcommand)]
    pub command: SprintCommand,
}

#[derive(Debug, Args)]
pub struct EpicArgs {
    #[command(subcommand)]
    pub command: EpicCommand,
}

#[derive(Debug, Subcommand)]
pub enum EpicCommand {
    List(ListEpicsArgs),
    Create(CreateEpicArgs),
    Add(AddEpicIssuesArgs),
    Remove(RemoveEpicIssuesArgs),
}

#[derive(Debug, Subcommand)]
pub enum SprintCommand {
    List(ListSprintsArgs),
    Add(AddSprintIssuesArgs),
    Close(CloseSprintArgs),
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Jira Cloud site URL, for example https://your-team.atlassian.net
    pub site: Option<String>,

    /// Atlassian account email
    #[arg(long)]
    pub email: Option<String>,

    /// Jira API token. Warning: command-line tokens may be visible to other local users.
    #[arg(long)]
    pub token: Option<String>,

    /// Open the Atlassian API token page, then prompt for credentials.
    #[arg(long, short = 'w')]
    pub web: bool,
}

#[derive(Debug, Args)]
pub struct LogoutArgs {
    /// Site URL or host key to remove. Defaults to the active site.
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct CreateContextArgs {
    /// Context name
    pub name: String,

    /// Site URL or host key. Defaults to the active authenticated site.
    #[arg(long)]
    pub site: Option<String>,

    /// Default Jira project key for this context.
    #[arg(long)]
    pub project: Option<String>,

    /// Set the new context as active.
    #[arg(long)]
    pub set_active: bool,
}

#[derive(Debug, Args)]
pub struct UseContextArgs {
    /// Context name to activate
    pub name: String,
}

#[derive(Debug, Args)]
pub struct DeleteContextArgs {
    /// Context name to delete
    pub name: String,
}

#[derive(Debug, Args)]
pub struct CreateIssueArgs {
    /// Issue summary
    #[arg(long)]
    pub summary: String,

    /// Markdown description text that will be converted to Atlassian Document Format
    #[arg(long, conflicts_with = "description_file")]
    pub description: Option<String>,

    /// Path to a Markdown description file, or '-' to read from stdin
    #[arg(long, conflicts_with = "description")]
    pub description_file: Option<PathBuf>,

    /// Jira project key. Defaults to the active context project.
    #[arg(long)]
    pub project: Option<String>,

    /// Jira issue type name. Defaults to Task.
    #[arg(long = "type", default_value = "Task")]
    pub issue_type: String,

    /// Site URL or host key. Defaults to the active context/site.
    #[arg(long)]
    pub site: Option<String>,

    /// Jira priority name.
    #[arg(long)]
    pub priority: Option<String>,

    /// Issue labels. Repeat to add more than one label.
    #[arg(long = "label")]
    pub labels: Vec<String>,

    /// Jira components. Repeat to add more than one component.
    #[arg(long = "component")]
    pub components: Vec<String>,

    /// Fix versions. Repeat to add more than one version.
    #[arg(long = "fix-version")]
    pub fix_versions: Vec<String>,

    /// Assignee identifier. Use `me`, `default`, or `x` to unassign.
    #[arg(long)]
    pub assignee: Option<String>,

    /// Parent issue key, including epic parent links where supported.
    #[arg(long)]
    pub parent: Option<String>,

    /// Set a raw field value using key=value. Repeat for multiple fields.
    #[arg(long = "field")]
    pub fields: Vec<String>,

    /// Set a raw field value using key=JSON. Repeat for multiple fields.
    #[arg(long = "field-json")]
    pub field_json: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ViewIssueArgs {
    /// Jira issue key, for example ENG-123
    pub key: String,

    /// Site URL or host key. Defaults to the active context/site.
    #[arg(long)]
    pub site: Option<String>,

    /// Number of recent comments to include.
    #[arg(long, default_value_t = 0)]
    pub comments: u32,
}

#[derive(Debug, Args)]
pub struct ListIssuesArgs {
    /// Raw JQL query. If omitted, the active context project is required.
    #[arg(long, conflicts_with = "project")]
    pub jql: Option<String>,

    /// Jira project key. Defaults to the active context project when --jql is omitted.
    #[arg(long)]
    pub project: Option<String>,

    /// Maximum number of issues to return.
    #[arg(long, default_value_t = 20)]
    pub limit: u32,

    /// Site URL or host key. Defaults to the active context/site.
    #[arg(long)]
    pub site: Option<String>,

    /// Output raw JSON instead of formatted rows.
    #[arg(long, conflicts_with = "csv")]
    pub raw: bool,

    /// Output CSV instead of formatted rows.
    #[arg(long, conflicts_with = "raw")]
    pub csv: bool,

    /// Assignee filter.
    #[arg(long)]
    pub assignee: Option<String>,

    /// Reporter filter.
    #[arg(long)]
    pub reporter: Option<String>,

    /// Status filters. Repeat to add more than one status.
    #[arg(long = "status")]
    pub statuses: Vec<String>,

    /// Issue type filters. Repeat to add more than one type.
    #[arg(long = "type")]
    pub issue_types: Vec<String>,

    /// Priority filters. Repeat to add more than one priority.
    #[arg(long = "priority")]
    pub priorities: Vec<String>,

    /// Label filters. Repeat to add more than one label.
    #[arg(long = "label")]
    pub labels: Vec<String>,

    /// Sort field. Defaults to updated.
    #[arg(long, default_value = "updated")]
    pub order_by: String,

    /// Reverse the sort order.
    #[arg(long)]
    pub reverse: bool,

    /// Restrict to issues watched by the current user.
    #[arg(long)]
    pub watching: bool,
}

#[derive(Debug, Args)]
pub struct EditIssueArgs {
    pub key: String,

    #[arg(long)]
    pub summary: Option<String>,

    #[arg(long, conflicts_with = "description_file")]
    pub description: Option<String>,

    #[arg(long, conflicts_with = "description")]
    pub description_file: Option<PathBuf>,

    #[arg(long = "type")]
    pub issue_type: Option<String>,

    #[arg(long)]
    pub priority: Option<String>,

    #[arg(long = "label")]
    pub labels: Vec<String>,

    #[arg(long = "component")]
    pub components: Vec<String>,

    #[arg(long = "fix-version")]
    pub fix_versions: Vec<String>,

    #[arg(long)]
    pub assignee: Option<String>,

    #[arg(long)]
    pub parent: Option<String>,

    #[arg(long = "field")]
    pub fields: Vec<String>,

    #[arg(long = "field-json")]
    pub field_json: Vec<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct AssignIssueArgs {
    pub key: String,
    pub user: String,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct MoveIssueArgs {
    pub key: String,
    pub transition: String,

    #[arg(long)]
    pub comment: Option<String>,

    #[arg(long)]
    pub resolution: Option<String>,

    #[arg(long)]
    pub assignee: Option<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct LinkIssueArgs {
    pub key: String,
    pub other: String,
    pub relation: String,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum RemoteLinkCommand {
    Add(AddRemoteLinkArgs),
}

#[derive(Debug, Args)]
pub struct AddRemoteLinkArgs {
    pub key: String,
    pub url: String,
    pub title: String,

    #[arg(long)]
    pub summary: Option<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct UnlinkIssueArgs {
    pub key: String,
    pub other: String,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct CloneIssueArgs {
    pub key: String,

    #[arg(long)]
    pub summary: Option<String>,

    #[arg(long)]
    pub project: Option<String>,

    #[arg(long = "type")]
    pub issue_type: Option<String>,

    #[arg(long)]
    pub assignee: Option<String>,

    #[arg(long = "label")]
    pub labels: Vec<String>,

    #[arg(long = "component")]
    pub components: Vec<String>,

    #[arg(long = "fix-version")]
    pub fix_versions: Vec<String>,

    #[arg(long)]
    pub priority: Option<String>,

    #[arg(long = "replace")]
    pub replacements: Vec<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct DeleteIssueArgs {
    pub key: String,

    #[arg(long)]
    pub cascade: bool,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct WatchIssueArgs {
    pub key: String,

    #[arg(long)]
    pub remove: bool,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct CommentArgs {
    #[command(subcommand)]
    pub command: CommentCommand,
}

#[derive(Debug, Subcommand)]
pub enum CommentCommand {
    Add(AddCommentArgs),
}

#[derive(Debug, Args)]
pub struct AddCommentArgs {
    pub key: String,

    pub body: Option<String>,

    #[arg(long = "template")]
    pub template: Option<PathBuf>,

    #[arg(long)]
    pub internal: bool,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct WorklogArgs {
    #[command(subcommand)]
    pub command: WorklogCommand,
}

#[derive(Debug, Subcommand)]
pub enum WorklogCommand {
    Add(AddWorklogArgs),
}

#[derive(Debug, Args)]
pub struct AddWorklogArgs {
    pub key: String,
    pub time_spent: String,

    #[arg(long)]
    pub comment: Option<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct ListBoardsArgs {
    #[arg(long)]
    pub project: Option<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct ListProjectsArgs {
    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct ListReleasesArgs {
    #[arg(long)]
    pub project: Option<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct ListSprintsArgs {
    pub sprint_id: Option<u64>,

    #[arg(long)]
    pub board: Option<u64>,

    #[arg(long)]
    pub current: bool,

    #[arg(long)]
    pub next: bool,

    #[arg(long)]
    pub prev: bool,

    #[arg(long)]
    pub state: Option<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct ListEpicsArgs {
    pub epic_key: Option<String>,

    #[arg(long)]
    pub project: Option<String>,

    #[arg(long)]
    pub limit: Option<u32>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct CreateEpicArgs {
    #[arg(long)]
    pub summary: String,

    #[arg(long)]
    pub name: Option<String>,

    #[arg(long, conflicts_with = "description_file")]
    pub description: Option<String>,

    #[arg(long, conflicts_with = "description")]
    pub description_file: Option<PathBuf>,

    #[arg(long)]
    pub project: Option<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct AddEpicIssuesArgs {
    pub epic_key: String,
    pub issues: Vec<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct RemoveEpicIssuesArgs {
    pub issues: Vec<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct AddSprintIssuesArgs {
    pub sprint_id: u64,
    pub issues: Vec<String>,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct CloseSprintArgs {
    pub sprint_id: u64,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct OpenArgs {
    pub target: Option<String>,

    #[arg(long)]
    pub launch: bool,

    #[arg(long)]
    pub site: Option<String>,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long)]
    pub site: Option<String>,

    #[arg(long)]
    pub email: Option<String>,

    #[arg(long)]
    pub token: Option<String>,

    #[arg(long)]
    pub project: Option<String>,

    #[arg(long, default_value = "default")]
    pub context: String,
}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    pub shell: String,
}

#[derive(Debug, Args)]
pub struct ManArgs {
    #[arg(long, default_value = ".")]
    pub output_dir: PathBuf,
}
