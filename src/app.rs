mod agile;
mod auth;
mod context;
mod issue;
mod meta;
mod support;

use anyhow::Result;

use crate::cli::{
    AuthCommand, BoardCommand, Cli, Command, CommentCommand, ContextCommand, EpicCommand,
    IssueCommand, ProjectCommand, ReleaseCommand, RemoteLinkCommand, SprintCommand, WorklogCommand,
};

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Auth(args) => match args.command {
            AuthCommand::Login(args) => auth::login(args),
            AuthCommand::Status => auth::status(),
            AuthCommand::Logout(args) => auth::logout(args),
        },
        Command::Context(args) => match args.command {
            ContextCommand::Create(args) => context::create(args),
            ContextCommand::Use(args) => context::use_context(args),
            ContextCommand::List => context::list(),
            ContextCommand::Delete(args) => context::delete(args),
        },
        Command::Issue(args) => match args.command {
            IssueCommand::Create(args) => issue::create(args),
            IssueCommand::View(args) => issue::view(args),
            IssueCommand::List(args) => issue::list(args),
            IssueCommand::Edit(args) => issue::edit(args),
            IssueCommand::Assign(args) => issue::assign(args),
            IssueCommand::Move(args) => issue::move_issue(args),
            IssueCommand::Link(args) => issue::link(args),
            IssueCommand::RemoteLink(args) => match args {
                RemoteLinkCommand::Add(args) => issue::add_remote_link(args),
            },
            IssueCommand::Unlink(args) => issue::unlink(args),
            IssueCommand::Clone(args) => issue::clone_issue(args),
            IssueCommand::Delete(args) => issue::delete(args),
            IssueCommand::Watch(args) => issue::watch(args),
            IssueCommand::Comment(args) => match args.command {
                CommentCommand::Add(args) => issue::add_comment(args),
            },
            IssueCommand::Worklog(args) => match args.command {
                WorklogCommand::Add(args) => issue::add_worklog(args),
            },
        },
        Command::Epic(args) => match args.command {
            EpicCommand::List(args) => agile::list_epics(args),
            EpicCommand::Create(args) => agile::create_epic(args),
            EpicCommand::Add(args) => agile::add_epic_issues(args),
            EpicCommand::Remove(args) => agile::remove_epic_issues(args),
        },
        Command::Board(args) => match args.command {
            BoardCommand::List(args) => agile::list_boards(args),
        },
        Command::Project(args) => match args.command {
            ProjectCommand::List(args) => agile::list_projects(args),
        },
        Command::Release(args) => match args.command {
            ReleaseCommand::List(args) => agile::list_releases(args),
        },
        Command::Sprint(args) => match args.command {
            SprintCommand::List(args) => agile::list_sprints(args),
            SprintCommand::Add(args) => agile::sprint_add(args),
            SprintCommand::Close(args) => agile::sprint_close(args),
        },
        Command::Me => meta::me(),
        Command::Open(args) => meta::open_target(args),
        Command::ServerInfo => meta::server_info(),
        Command::Init(args) => meta::init(args),
        Command::Completion(args) => meta::completion(args),
        Command::Man(args) => meta::man(args),
        Command::Version => meta::version(),
    }
}
