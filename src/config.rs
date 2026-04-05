use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use url::Url;

pub const CONFIG_DIR_ENV_VAR: &str = "JIRA_CONFIG_DIR";
const CONFIG_DIR_NAME: &str = "jira";
const CONFIG_FILE_NAME: &str = "config.toml";
const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub active_site: Option<String>,
    #[serde(default)]
    pub active_context: Option<String>,
    #[serde(default)]
    pub sites: BTreeMap<String, SiteProfile>,
    #[serde(default)]
    pub contexts: BTreeMap<String, ContextProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteProfile {
    pub key: String,
    pub site_url: String,
    pub api_base_url: String,
    pub email: String,
    #[serde(default)]
    pub cloud_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextProfile {
    pub site: String,
    #[serde(default)]
    pub project: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;

        match fs::read_to_string(&path) {
            Ok(contents) => {
                let mut config: Self = toml::from_str(&contents)
                    .with_context(|| format!("decode config at {}", path.display()))?;
                if config.version == 0 {
                    config.version = CURRENT_VERSION;
                }
                Ok(config)
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                version: CURRENT_VERSION,
                ..Self::default()
            }),
            Err(err) => Err(err).with_context(|| format!("read config at {}", path.display())),
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        let dir = path
            .parent()
            .context("config path should always have a parent directory")?;
        fs::create_dir_all(dir)
            .with_context(|| format!("create config directory {}", dir.display()))?;

        let payload = toml::to_string_pretty(self).context("encode config")?;
        let tmp_path = path.with_extension("toml.tmp");
        fs::write(&tmp_path, payload)
            .with_context(|| format!("write temp config {}", tmp_path.display()))?;
        set_restrictive_permissions(&tmp_path)?;
        fs::rename(&tmp_path, &path)
            .with_context(|| format!("write config to {}", path.display()))?;
        Ok(())
    }

    pub fn upsert_site(&mut self, site: SiteProfile) {
        self.sites.insert(site.key.clone(), site);
    }

    pub fn set_active_site(&mut self, site_key: String) {
        self.active_site = Some(site_key);
    }

    pub fn upsert_context(&mut self, name: String, context: ContextProfile) {
        self.contexts.insert(name, context);
    }

    pub fn set_active_context(&mut self, name: String) -> Result<()> {
        let context = self
            .contexts
            .get(&name)
            .with_context(|| format!("context {name:?} not found"))?;
        self.active_context = Some(name);
        self.active_site = Some(context.site.clone());
        Ok(())
    }

    pub fn remove_context(&mut self, name: &str) -> bool {
        let removed = self.contexts.remove(name).is_some();
        if removed && self.active_context.as_deref() == Some(name) {
            self.active_context = None;
        }
        removed
    }

    pub fn remove_site(&mut self, site_key: &str) -> bool {
        let removed = self.sites.remove(site_key).is_some();
        if removed {
            self.contexts.retain(|_, context| context.site != site_key);
            if let Some(active_context) = self.active_context.clone() {
                if !self.contexts.contains_key(&active_context) {
                    self.active_context = None;
                }
            }
        }
        if removed && self.active_site.as_deref() == Some(site_key) {
            self.active_site = self
                .active_context
                .as_ref()
                .and_then(|name| self.contexts.get(name))
                .map(|context| context.site.clone())
                .or_else(|| self.sites.keys().next().cloned());
        }
        removed
    }
}

pub fn normalize_site_url(input: &str) -> Result<String> {
    let raw = input.trim();
    if raw.is_empty() {
        bail!("site URL cannot be empty");
    }

    let with_scheme = if raw.contains("://") {
        raw.to_owned()
    } else {
        format!("https://{raw}")
    };

    let mut url =
        Url::parse(&with_scheme).with_context(|| format!("parse site URL {with_scheme:?}"))?;

    if url.scheme() != "https" {
        bail!("Jira Cloud site URLs must use https://");
    }

    if url.host_str().is_none() {
        bail!("site URL must include a host");
    }

    url.set_username("")
        .map_err(|_| anyhow::anyhow!("site URL must not include a username"))?;
    url.set_password(None)
        .map_err(|_| anyhow::anyhow!("site URL must not include a password"))?;
    url.set_path("");
    url.set_query(None);
    url.set_fragment(None);

    Ok(url.to_string().trim_end_matches('/').to_owned())
}

pub fn site_key_from_input(input: &str) -> Result<String> {
    let normalized = normalize_site_url(input)?;
    site_key_from_url(&normalized)
}

pub fn site_key_from_url(site_url: &str) -> Result<String> {
    let url = Url::parse(site_url).with_context(|| format!("parse site URL {site_url:?}"))?;
    let host = url.host_str().context("site URL missing host")?;

    if let Some(port) = url.port() {
        Ok(format!("{}:{port}", host.to_ascii_lowercase()))
    } else {
        Ok(host.to_ascii_lowercase())
    }
}

pub fn config_path() -> Result<PathBuf> {
    let base = match std::env::var(CONFIG_DIR_ENV_VAR) {
        Ok(path) if !path.trim().is_empty() => PathBuf::from(path),
        _ => dirs::config_dir()
            .context("resolve platform config directory")?
            .join(CONFIG_DIR_NAME),
    };

    Ok(base.join(CONFIG_FILE_NAME))
}

fn default_version() -> u32 {
    CURRENT_VERSION
}

fn set_restrictive_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, permissions)
            .with_context(|| format!("set permissions on {}", path.display()))?;
    }

    #[cfg(not(unix))]
    {
        let _ = path;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Config, ContextProfile, SiteProfile, normalize_site_url, site_key_from_input};

    #[test]
    fn normalize_site_url_adds_https_and_strips_path() {
        let value = normalize_site_url("Example.atlassian.net/jira/software/projects/ABC").unwrap();
        assert_eq!(value, "https://example.atlassian.net");
    }

    #[test]
    fn site_key_accepts_raw_host() {
        let value = site_key_from_input("example.atlassian.net").unwrap();
        assert_eq!(value, "example.atlassian.net");
    }

    #[test]
    fn remove_site_reassigns_active_site() {
        let mut config = Config::default();
        config.upsert_site(SiteProfile {
            key: "a.example".into(),
            site_url: "https://a.example".into(),
            api_base_url: "https://a.example".into(),
            email: "a@example.com".into(),
            cloud_id: None,
        });
        config.upsert_site(SiteProfile {
            key: "b.example".into(),
            site_url: "https://b.example".into(),
            api_base_url: "https://b.example".into(),
            email: "b@example.com".into(),
            cloud_id: None,
        });
        config.set_active_site("a.example".into());

        assert!(config.remove_site("a.example"));
        assert_eq!(config.active_site.as_deref(), Some("b.example"));
    }

    #[test]
    fn set_active_context_updates_active_site() {
        let mut config = Config::default();
        config.upsert_site(SiteProfile {
            key: "a.example".into(),
            site_url: "https://a.example".into(),
            api_base_url: "https://a.example".into(),
            email: "a@example.com".into(),
            cloud_id: None,
        });
        config.upsert_context(
            "work".into(),
            ContextProfile {
                site: "a.example".into(),
                project: Some("PROJ".into()),
            },
        );

        config.set_active_context("work".into()).unwrap();

        assert_eq!(config.active_context.as_deref(), Some("work"));
        assert_eq!(config.active_site.as_deref(), Some("a.example"));
    }

    #[test]
    fn remove_site_prunes_contexts_for_that_site() {
        let mut config = Config::default();
        config.upsert_site(SiteProfile {
            key: "a.example".into(),
            site_url: "https://a.example".into(),
            api_base_url: "https://a.example".into(),
            email: "a@example.com".into(),
            cloud_id: None,
        });
        config.upsert_context(
            "work".into(),
            ContextProfile {
                site: "a.example".into(),
                project: Some("PROJ".into()),
            },
        );
        config.set_active_context("work".into()).unwrap();

        assert!(config.remove_site("a.example"));
        assert!(config.contexts.is_empty());
        assert!(config.active_context.is_none());
        assert!(config.active_site.is_none());
    }
}
