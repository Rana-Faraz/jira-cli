use anyhow::{Result, bail};

use crate::{
    cli::{CreateContextArgs, DeleteContextArgs, UseContextArgs},
    config::{Config, ContextProfile},
};

use super::support::{normalize_optional_project, resolve_context_site_key};

pub(super) fn create(args: CreateContextArgs) -> Result<()> {
    let mut config = Config::load()?;
    let site_key = resolve_context_site_key(&config, args.site.as_deref())?;

    let context = ContextProfile {
        site: site_key.clone(),
        project: normalize_optional_project(args.project),
    };

    let was_present = config.contexts.contains_key(&args.name);
    config.upsert_context(args.name.clone(), context);

    if args.set_active || config.active_context.is_none() {
        config.set_active_context(args.name.clone())?;
    }

    config.save()?;

    let action = if was_present { "Updated" } else { "Created" };
    println!("{action} context {:?} (site: {site_key})", args.name);
    if config.active_context.as_deref() == Some(args.name.as_str()) {
        println!("Active context: {}", args.name);
    }
    Ok(())
}

pub(super) fn use_context(args: UseContextArgs) -> Result<()> {
    let mut config = Config::load()?;
    config.set_active_context(args.name.clone())?;
    config.save()?;

    println!("Activated context {:?}", args.name);
    Ok(())
}

pub(super) fn list() -> Result<()> {
    let config = Config::load()?;

    if config.contexts.is_empty() {
        println!("No contexts configured. Run `jira context create` to add one.");
        return Ok(());
    }

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

pub(super) fn delete(args: DeleteContextArgs) -> Result<()> {
    let mut config = Config::load()?;
    if !config.remove_context(&args.name) {
        bail!("context {:?} not found", args.name);
    }

    if config.active_context.is_none() {
        if let Some(first_name) = config.contexts.keys().next().cloned() {
            config.set_active_context(first_name)?;
        }
    }

    config.save()?;
    println!("Deleted context {:?}", args.name);
    Ok(())
}
