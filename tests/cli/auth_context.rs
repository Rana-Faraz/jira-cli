use super::common::*;
use super::*;

#[test]
fn auth_status_guides_user_when_no_sites_are_configured() {
    let temp = TempDir::new().expect("create temp dir");

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env_remove("JIRA_TOKEN")
        .arg("auth")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No Jira sites configured. Run `jira auth login` to add one.",
        ));
}

#[test]
fn context_create_persists_and_activates_the_new_context() {
    let temp = TempDir::new().expect("create temp dir");
    write_config(
        &temp,
        r#"
version = 1
active_site = "example.atlassian.net"

[sites."example.atlassian.net"]
key = "example.atlassian.net"
site_url = "https://example.atlassian.net"
api_base_url = "https://example.atlassian.net"
email = "user@example.com"
"#,
    );

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .arg("context")
        .arg("create")
        .arg("work")
        .arg("--project")
        .arg("eng")
        .arg("--set-active")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created context"))
        .stdout(predicate::str::contains("Active context: work"));

    let written = read_config(&temp);
    assert!(written.contains("active_context = \"work\""));
    assert!(written.contains("active_site = \"example.atlassian.net\""));
    assert!(written.contains("[contexts.work]"));
    assert!(written.contains("project = \"ENG\""));
}

#[test]
fn context_use_switches_active_site_with_active_context() {
    let temp = TempDir::new().expect("create temp dir");
    write_config(
        &temp,
        r#"
version = 1
active_site = "one.atlassian.net"
active_context = "work"

[sites."one.atlassian.net"]
key = "one.atlassian.net"
site_url = "https://one.atlassian.net"
api_base_url = "https://one.atlassian.net"
email = "one@example.com"

[sites."two.atlassian.net"]
key = "two.atlassian.net"
site_url = "https://two.atlassian.net"
api_base_url = "https://two.atlassian.net"
email = "two@example.com"

[contexts.work]
site = "one.atlassian.net"
project = "ONE"

[contexts.ops]
site = "two.atlassian.net"
project = "OPS"
"#,
    );

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .arg("context")
        .arg("use")
        .arg("ops")
        .assert()
        .success()
        .stdout(predicate::str::contains("Activated context \"ops\""));

    let written = read_config(&temp);
    assert!(written.contains("active_context = \"ops\""));
    assert!(written.contains("active_site = \"two.atlassian.net\""));
}

#[test]
fn context_delete_falls_back_to_another_context_when_active_one_is_removed() {
    let temp = TempDir::new().expect("create temp dir");
    write_config(
        &temp,
        r#"
version = 1
active_site = "one.atlassian.net"
active_context = "work"

[sites."one.atlassian.net"]
key = "one.atlassian.net"
site_url = "https://one.atlassian.net"
api_base_url = "https://one.atlassian.net"
email = "one@example.com"

[sites."two.atlassian.net"]
key = "two.atlassian.net"
site_url = "https://two.atlassian.net"
api_base_url = "https://two.atlassian.net"
email = "two@example.com"

[contexts.work]
site = "one.atlassian.net"
project = "ONE"

[contexts.ops]
site = "two.atlassian.net"
project = "OPS"
"#,
    );

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .arg("context")
        .arg("delete")
        .arg("work")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted context \"work\""));

    let written = read_config(&temp);
    assert!(!written.contains("[contexts.work]"));
    assert!(written.contains("active_context = \"ops\""));
    assert!(written.contains("active_site = \"two.atlassian.net\""));
}

#[test]
fn issue_create_fails_with_clear_error_when_no_project_can_be_resolved() {
    let temp = TempDir::new().expect("create temp dir");
    write_config(
        &temp,
        r#"
version = 1
active_site = "example.atlassian.net"

[sites."example.atlassian.net"]
key = "example.atlassian.net"
site_url = "https://example.atlassian.net"
api_base_url = "https://example.atlassian.net"
email = "user@example.com"
"#,
    );

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("create")
        .arg("--summary")
        .arg("Smoke")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "project is required when the active context does not define one",
        ));
}

#[test]
fn issue_list_requires_either_jql_or_a_context_project() {
    let temp = TempDir::new().expect("create temp dir");
    write_config(
        &temp,
        r#"
version = 1
active_site = "example.atlassian.net"

[sites."example.atlassian.net"]
key = "example.atlassian.net"
site_url = "https://example.atlassian.net"
api_base_url = "https://example.atlassian.net"
email = "user@example.com"
"#,
    );

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "project is required when the active context does not define one",
        ));
}
