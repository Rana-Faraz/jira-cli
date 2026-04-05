use std::fs;

use tempfile::TempDir;

pub(super) fn write_config(temp: &TempDir, body: &str) {
    fs::write(temp.path().join("config.toml"), body.trim_start()).expect("write config");
}

pub(super) fn read_config(temp: &TempDir) -> String {
    fs::read_to_string(temp.path().join("config.toml")).expect("read config")
}

pub(super) fn write_mock_config(temp: &TempDir, api_base_url: &str, project: Option<&str>) {
    let mut body = format!(
        r#"
version = 1
active_site = "mock.local"
active_context = "work"

[sites."mock.local"]
key = "mock.local"
site_url = "{api_base_url}"
api_base_url = "{api_base_url}"
email = "user@example.com"

[contexts.work]
site = "mock.local"
"#,
    );

    if let Some(project) = project {
        body.push_str(&format!("project = \"{project}\"\n"));
    }

    write_config(temp, &body);
}
