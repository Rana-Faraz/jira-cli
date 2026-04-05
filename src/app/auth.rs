use anyhow::{Context, Result, bail};

use crate::{
    cli::{LoginArgs, LogoutArgs},
    config::{Config, normalize_site_url},
    jira_cloud::JiraCloudClient,
    secret, ui,
};

use super::support::{
    print_login_success, print_token_help, print_web_instructions, resolve_logout_site_key,
};

pub(super) fn login(args: LoginArgs) -> Result<()> {
    if secret::token_from_env().is_some() {
        bail!(
            "{} is set; token is externally managed. Unset it to use `jira auth login`.",
            secret::TOKEN_ENV_VAR
        );
    }

    if args.token.is_some() {
        eprintln!(
            "WARNING: --token is visible in process listings and shell history; prefer the interactive prompt"
        );
    }

    let site = match args.site {
        Some(site) => normalize_site_url(&site)?,
        None if ui::is_terminal() => normalize_site_url(&ui::prompt_string(
            "Jira Cloud site URL (for example https://your-team.atlassian.net)",
        )?)?,
        None => bail!("site is required when not running in a TTY"),
    };

    if args.web {
        print_web_instructions();
    }

    let email = match args.email {
        Some(email) => email.trim().to_owned(),
        None if ui::is_terminal() => ui::prompt_string("Atlassian account email")?,
        None => bail!("email is required when not running in a TTY"),
    };

    if email.is_empty() {
        bail!("email cannot be empty");
    }

    let token = match args.token {
        Some(token) => token.trim().to_owned(),
        None if ui::is_terminal() => {
            print_token_help();
            ui::prompt_secret("Jira API token")?
        }
        None => bail!("token is required when not running in a TTY"),
    };

    if token.is_empty() {
        bail!("token cannot be empty");
    }

    let client = JiraCloudClient::new()?;
    let verified = client
        .verify_credentials(&site, &email, &token)
        .context("verify Jira Cloud credentials")?;

    secret::store_token(&verified.profile.key, &token).context("store API token in keyring")?;

    let mut config = Config::load()?;
    let site_was_known = config.sites.contains_key(&verified.profile.key);
    config.upsert_site(verified.profile.clone());
    config.set_active_site(verified.profile.key.clone());
    config.save()?;

    print_login_success(&verified, site_was_known);
    Ok(())
}

pub(super) fn status() -> Result<()> {
    let config = Config::load()?;

    if config.sites.is_empty() {
        println!("No Jira sites configured. Run `jira auth login` to add one.");
        return Ok(());
    }

    println!("Sites:");
    for (key, site) in &config.sites {
        let active = if config.active_site.as_deref() == Some(key.as_str()) {
            "*"
        } else {
            " "
        };
        println!("{active} {} ({key})", site.site_url);
        println!("    email: {}", site.email);
        println!("    api base: {}", site.api_base_url);
        println!("    token source: {}", secret::resolved_token_source());
        if let Some(cloud_id) = &site.cloud_id {
            println!("    cloud id: {cloud_id}");
        }
    }

    if config.contexts.is_empty() {
        println!();
        println!("No contexts configured. Run `jira context create` to add one.");
        return Ok(());
    }

    println!();
    println!("Contexts:");
    for (name, context) in &config.contexts {
        let active = if config.active_context.as_deref() == Some(name.as_str()) {
            "*"
        } else {
            " "
        };
        println!("{active} {name} (site: {})", context.site);
        if let Some(project) = &context.project {
            println!("    project: {project}");
        }
    }

    Ok(())
}

pub(super) fn logout(args: LogoutArgs) -> Result<()> {
    if secret::token_from_env().is_some() {
        bail!(
            "{} is set; token is externally managed. Unset it to use `jira auth logout`.",
            secret::TOKEN_ENV_VAR
        );
    }

    let mut config = Config::load()?;
    let site_key = resolve_logout_site_key(&config, args.site)?;

    secret::delete_token(&site_key).context("delete API token from keyring")?;
    if !config.remove_site(&site_key) {
        bail!("site {site_key:?} not found in configuration");
    }
    config.save()?;

    println!("Removed credentials for {site_key}");
    Ok(())
}
