use super::common::*;
use super::*;

#[test]
fn issue_assign_me_resolves_current_user_and_updates_assignee() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let me = server.mock(|when, then| {
        when.method(GET).path("/rest/api/3/myself");
        then.status(200).json_body_obj(&serde_json::json!({
            "accountId": "abc-123",
            "displayName": "Faraz",
            "emailAddress": "faraz@example.com"
        }));
    });

    let assign = server.mock(|when, then| {
        when.method(httpmock::Method::PUT)
            .path("/rest/api/3/issue/SCRUM-2/assignee")
            .body_contains(r#""accountId":"abc-123""#);
        then.status(204);
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("assign")
        .arg("SCRUM-2")
        .arg("me")
        .assert()
        .success()
        .stdout(predicate::str::contains("Assigned SCRUM-2"));

    me.assert();
    assign.assert();
}

#[test]
fn issue_comment_add_posts_markdown_body_as_adf() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let comment = server.mock(|when, then| {
        when.method(POST).path("/rest/api/3/issue/SCRUM-2/comment");

        then.status(201)
            .json_body_obj(&serde_json::json!({ "id": "20001" }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("comment")
        .arg("add")
        .arg("SCRUM-2")
        .arg("Hello **team**")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added comment"));

    comment.assert();
}

#[test]
fn issue_delete_cascade_passes_delete_subtasks_flag() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let delete_issue = server.mock(|when, then| {
        when.method(httpmock::Method::DELETE)
            .path("/rest/api/3/issue/SCRUM-4")
            .query_param("deleteSubtasks", "true");

        then.status(204);
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("delete")
        .arg("SCRUM-4")
        .arg("--cascade")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted SCRUM-4"));

    delete_issue.assert();
}

#[test]
fn issue_link_posts_the_link_payload() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let link = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issueLink")
            .body_contains(r#""name":"Blocks""#)
            .body_contains(r#""outwardIssue":{"key":"SCRUM-1"}"#)
            .body_contains(r#""inwardIssue":{"key":"SCRUM-2"}"#);

        then.status(201).json_body_obj(&serde_json::json!({}));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("link")
        .arg("SCRUM-1")
        .arg("SCRUM-2")
        .arg("Blocks")
        .assert()
        .success()
        .stdout(predicate::str::contains("Linked SCRUM-1"));

    link.assert();
}

#[test]
fn issue_unlink_resolves_link_id_and_deletes_it() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let fetch_issue = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/SCRUM-1")
            .query_param("fields", "issuelinks");

        then.status(200).json_body_obj(&serde_json::json!({
            "key": "SCRUM-1",
            "fields": {
                "issuelinks": [
                    {
                        "id": "9001",
                        "type": { "name": "Blocks" },
                        "outwardIssue": { "key": "SCRUM-2" }
                    }
                ]
            }
        }));
    });

    let unlink = server.mock(|when, then| {
        when.method(httpmock::Method::DELETE)
            .path("/rest/api/3/issueLink/9001");
        then.status(204);
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("unlink")
        .arg("SCRUM-1")
        .arg("SCRUM-2")
        .assert()
        .success()
        .stdout(predicate::str::contains("Unlinked SCRUM-1"));

    fetch_issue.assert();
    unlink.assert();
}

#[test]
fn issue_watch_remove_uses_current_user_account_id() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let me = server.mock(|when, then| {
        when.method(GET).path("/rest/api/3/myself");
        then.status(200).json_body_obj(&serde_json::json!({
            "accountId": "abc-123",
            "displayName": "Faraz"
        }));
    });

    let unwatch = server.mock(|when, then| {
        when.method(httpmock::Method::DELETE)
            .path("/rest/api/3/issue/SCRUM-2/watchers")
            .query_param("accountId", "abc-123");

        then.status(204);
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("watch")
        .arg("SCRUM-2")
        .arg("--remove")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated watch state for SCRUM-2"));

    me.assert();
    unwatch.assert();
}

#[test]
fn issue_edit_updates_summary_and_priority() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let edit = server.mock(|when, then| {
        when.method(httpmock::Method::PUT)
            .path("/rest/api/3/issue/SCRUM-2")
            .body_contains(r#""summary":"Updated summary""#)
            .body_contains(r#""priority":{"name":"High"}"#);
        then.status(204);
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("edit")
        .arg("SCRUM-2")
        .arg("--summary")
        .arg("Updated summary")
        .arg("--priority")
        .arg("High")
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated SCRUM-2"));

    edit.assert();
}

#[test]
fn issue_move_resolves_transition_name_and_posts_comment_and_resolution() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let transitions = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/SCRUM-2/transitions");
        then.status(200).json_body_obj(&serde_json::json!({
            "transitions": [
                { "id": "31", "name": "Done" }
            ]
        }));
    });

    let move_issue = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue/SCRUM-2/transitions")
            .body_contains(r#""transition":{"id":"31"}"#)
            .body_contains(r#""resolution":{"name":"Fixed"}"#)
            .body_contains("Shipped");
        then.status(204);
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("move")
        .arg("SCRUM-2")
        .arg("Done")
        .arg("--comment")
        .arg("Shipped")
        .arg("--resolution")
        .arg("Fixed")
        .assert()
        .success()
        .stdout(predicate::str::contains("Moved SCRUM-2"));

    transitions.assert();
    move_issue.assert();
}

#[test]
fn issue_move_can_unassign_during_transition() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let transitions = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/SCRUM-2/transitions");
        then.status(200).json_body_obj(&serde_json::json!({
            "transitions": [
                { "id": "31", "name": "Done" }
            ]
        }));
    });

    let move_issue = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue/SCRUM-2/transitions")
            .body_contains(r#""transition":{"id":"31"}"#)
            .body_contains(r#""assignee":null"#);
        then.status(204);
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("move")
        .arg("SCRUM-2")
        .arg("Done")
        .arg("--assignee")
        .arg("x")
        .assert()
        .success()
        .stdout(predicate::str::contains("Moved SCRUM-2"));

    transitions.assert();
    move_issue.assert();
}

#[test]
fn issue_worklog_add_posts_time_spent_and_comment() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let worklog = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue/SCRUM-2/worklog")
            .body_contains(r#""timeSpent":"1h 30m""#)
            .body_contains("Pairing session");
        then.status(201)
            .json_body_obj(&serde_json::json!({ "id": "30001" }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("worklog")
        .arg("add")
        .arg("SCRUM-2")
        .arg("1h 30m")
        .arg("--comment")
        .arg("Pairing session")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added worklog"));

    worklog.assert();
}
