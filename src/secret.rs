use anyhow::{Context, Result};
use keyring::{Entry, Error as KeyringError};

pub const SERVICE_NAME: &str = "jira-cli";
pub const TOKEN_ENV_VAR: &str = "JIRA_TOKEN";

pub fn token_from_env() -> Option<String> {
    std::env::var(TOKEN_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

pub fn resolved_token_source() -> &'static str {
    if token_from_env().is_some() {
        TOKEN_ENV_VAR
    } else {
        "keyring"
    }
}

pub fn store_token(site_key: &str, token: &str) -> Result<()> {
    let entry = entry(site_key)?;
    entry
        .set_password(token)
        .with_context(|| format!("write token for {site_key}"))?;
    Ok(())
}

pub fn delete_token(site_key: &str) -> Result<()> {
    let entry = entry(site_key)?;
    match entry.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(err) => Err(err).with_context(|| format!("delete token for {site_key}")),
    }
}

pub fn load_token(site_key: &str) -> Result<String> {
    if let Some(token) = token_from_env() {
        return Ok(token);
    }

    let entry = entry(site_key)?;
    entry
        .get_password()
        .with_context(|| format!("read token for {site_key}"))
}

fn entry(site_key: &str) -> Result<Entry> {
    Entry::new(SERVICE_NAME, &format!("site/{site_key}/token")).context("create keyring entry")
}
