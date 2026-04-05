use std::fs;

use anyhow::{Context, Result, bail};
use clap::CommandFactory;
use clap_complete::{Generator, Shell, generate};

use crate::{
    cli::{Cli, CompletionArgs, InitArgs, ManArgs, OpenArgs},
    config::{Config, ContextProfile, normalize_site_url},
    jira_cloud::JiraCloudClient,
    secret,
};

use super::support::{AppContext, normalize_optional_project};

pub(super) fn me() -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(None)?;
    let user = session.client.me(&session.site, &session.token)?;

    println!("{}", user.display_name);
    println!("Account ID: {}", user.account_id);
    if let Some(email) = user.email_address {
        if !email.is_empty() {
            println!("Email: {email}");
        }
    }
    println!("Site: {}", session.site.site_url);
    Ok(())
}

pub(super) fn open_target(args: OpenArgs) -> Result<()> {
    let app = AppContext::load()?;
    let site = app.issue_site(args.site.as_deref())?;
    let url = match args.target {
        Some(target) => {
            if target.starts_with("http://") || target.starts_with("https://") {
                target
            } else {
                format!("{}/browse/{}", site.site_url, target.trim())
            }
        }
        None => {
            if let Some(context_name) = app.config.active_context.as_ref() {
                if let Some(context) = app.config.contexts.get(context_name) {
                    if let Some(project) = context.project.as_ref() {
                        format!("{}/jira/software/projects/{project}", site.site_url)
                    } else {
                        site.site_url.clone()
                    }
                } else {
                    site.site_url.clone()
                }
            } else {
                site.site_url.clone()
            }
        }
    };

    if args.launch {
        webbrowser::open(&url).with_context(|| format!("open {url}"))?;
    }
    println!("{url}");
    Ok(())
}

pub(super) fn server_info() -> Result<()> {
    let app = AppContext::load()?;
    let session = app.jira(None)?;
    let info = session.client.server_info(&session.site, &session.token)?;

    println!("Title: {}", info.server_title);
    println!("Deployment: {}", info.deployment_type);
    println!("Version: {}", info.version);
    println!("Build: {}", info.build_number);
    Ok(())
}

pub(super) fn init(args: InitArgs) -> Result<()> {
    let site = match args.site {
        Some(site) => normalize_site_url(&site)?,
        None => bail!("--site is required for `jira init`"),
    };
    let email = args
        .email
        .context("`jira init` requires --email so it can authenticate the site")?;
    let token = args
        .token
        .context("`jira init` requires --token so it can authenticate the site")?;

    let client = JiraCloudClient::new()?;
    let verified = client
        .verify_credentials(&site, &email, &token)
        .context("verify Jira Cloud credentials")?;

    secret::store_token(&verified.profile.key, &token).context("store API token in keyring")?;

    let mut config = Config::load()?;
    config.upsert_site(verified.profile.clone());
    config.set_active_site(verified.profile.key.clone());
    config.upsert_context(
        args.context.clone(),
        ContextProfile {
            site: verified.profile.key.clone(),
            project: normalize_optional_project(args.project),
        },
    );
    config.set_active_context(args.context.clone())?;
    config.save()?;

    println!("Initialized {}", verified.profile.site_url);
    println!("Active context: {}", args.context);
    Ok(())
}

pub(super) fn completion(args: CompletionArgs) -> Result<()> {
    let shell = match args.shell.to_ascii_lowercase().as_str() {
        "bash" => Shell::Bash,
        "elvish" => Shell::Elvish,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        "zsh" => Shell::Zsh,
        other => bail!("unsupported shell {other:?}"),
    };

    let mut command = Cli::command();
    let mut stdout = std::io::stdout();
    generate_completion(shell, &mut command, &mut stdout);
    Ok(())
}

pub(super) fn man(args: ManArgs) -> Result<()> {
    fs::create_dir_all(&args.output_dir)
        .with_context(|| format!("create man output directory {}", args.output_dir.display()))?;

    let command = Cli::command();
    let mut file = fs::File::create(args.output_dir.join("jira.1"))
        .with_context(|| format!("create manpage in {}", args.output_dir.display()))?;
    clap_mangen::Man::new(command).render(&mut file)?;
    println!("{}", args.output_dir.join("jira.1").display());
    Ok(())
}

pub(super) fn version() -> Result<()> {
    println!("{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn generate_completion<G: Generator>(
    generator: G,
    command: &mut clap::Command,
    out: &mut dyn std::io::Write,
) {
    generate(generator, command, command.get_name().to_owned(), out);
}
