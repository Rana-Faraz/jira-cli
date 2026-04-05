use super::common::*;
use super::*;

#[test]
fn sprint_list_prints_sprints_for_a_board() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let sprints = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/board/1/sprint")
            .query_param("maxResults", "50");

        then.status(200).json_body_obj(&serde_json::json!({
            "values": [
                { "id": 11, "name": "Sprint 1", "state": "closed" },
                { "id": 12, "name": "Sprint 2", "state": "active" }
            ]
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("sprint")
        .arg("list")
        .arg("--board")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("11\tSprint 1\tclosed"))
        .stdout(predicate::str::contains("12\tSprint 2\tactive"));

    sprints.assert();
}

#[test]
fn sprint_add_posts_issues_to_the_sprint() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let add = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/agile/1.0/sprint/12/issue")
            .body_contains(r#""issues":["SCRUM-1","SCRUM-2"]"#);
        then.status(204);
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("sprint")
        .arg("add")
        .arg("12")
        .arg("SCRUM-1")
        .arg("SCRUM-2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added 2 issues to sprint 12"));

    add.assert();
}

#[test]
fn sprint_close_updates_the_sprint_state() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let close = server.mock(|when, then| {
        when.method(httpmock::Method::PUT)
            .path("/rest/agile/1.0/sprint/12")
            .body_contains(r#""state":"closed""#);
        then.status(200)
            .json_body_obj(&serde_json::json!({ "id": 12, "state": "closed" }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("sprint")
        .arg("close")
        .arg("12")
        .assert()
        .success()
        .stdout(predicate::str::contains("Closed sprint 12"));

    close.assert();
}

#[test]
fn epic_list_prints_epics_from_project_scope() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let epics = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/search/jql")
            .query_param(
                "jql",
                "project = SCRUM AND issuetype = Epic ORDER BY updated DESC",
            )
            .query_param("maxResults", "20")
            .query_param("fields", "summary,status,issuetype,assignee");

        then.status(200).json_body_obj(&serde_json::json!({
            "issues": [
                {
                    "key": "SCRUM-10",
                    "fields": {
                        "summary": "Epic Alpha",
                        "status": { "name": "In Progress" },
                        "issuetype": { "name": "Epic" },
                        "assignee": null
                    }
                }
            ]
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("epic")
        .arg("list")
        .arg("--project")
        .arg("SCRUM")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "SCRUM-10\t[In Progress]\tEpic\tunassigned\tEpic Alpha",
        ));

    epics.assert();
}

#[test]
fn epic_add_posts_issues_to_the_epic() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let add = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/agile/1.0/epic/SCRUM-10/issue")
            .body_contains(r#""issues":["SCRUM-1","SCRUM-2"]"#);
        then.status(204);
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("epic")
        .arg("add")
        .arg("SCRUM-10")
        .arg("SCRUM-1")
        .arg("SCRUM-2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added 2 issues to epic SCRUM-10"));

    add.assert();
}

#[test]
fn epic_remove_posts_issues_to_none_bucket() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let remove = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/agile/1.0/epic/none/issue")
            .body_contains(r#""issues":["SCRUM-1","SCRUM-2"]"#);
        then.status(204);
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("epic")
        .arg("remove")
        .arg("SCRUM-1")
        .arg("SCRUM-2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed 2 issues from their epic"));

    remove.assert();
}

#[test]
fn sprint_list_current_filters_to_the_active_sprint() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let sprints = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/board/1/sprint")
            .query_param("maxResults", "50");

        then.status(200).json_body_obj(&serde_json::json!({
            "values": [
                { "id": 11, "name": "Sprint 1", "state": "closed" },
                { "id": 12, "name": "Sprint 2", "state": "active" },
                { "id": 13, "name": "Sprint 3", "state": "future" }
            ]
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("sprint")
        .arg("list")
        .arg("--board")
        .arg("1")
        .arg("--current")
        .assert()
        .success()
        .stdout(predicate::str::contains("12\tSprint 2\tactive"))
        .stdout(predicate::str::contains("Sprint 1").not());

    sprints.assert();
}

#[test]
fn epic_list_key_uses_the_epic_issue_endpoint() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let issues = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/epic/SCRUM-10/issue")
            .query_param("maxResults", "20")
            .query_param("fields", "summary,status,issuetype,assignee");

        then.status(200).json_body_obj(&serde_json::json!({
            "issues": [
                {
                    "key": "SCRUM-2",
                    "fields": {
                        "summary": "Task 2",
                        "status": { "name": "In Progress" },
                        "issuetype": { "name": "Story" },
                        "assignee": null
                    }
                }
            ]
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("epic")
        .arg("list")
        .arg("SCRUM-10")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "SCRUM-2\t[In Progress]\tStory\tunassigned\tTask 2",
        ));

    issues.assert();
}
