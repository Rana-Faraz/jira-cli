use super::common::*;
use super::*;

#[test]
fn issue_create_includes_priority_labels_parent_and_custom_fields() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let create_issue = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue")
            .body_contains(r#""priority":{"name":"High"}"#)
            .body_contains(r#""labels":["backend","urgent"]"#)
            .body_contains(r#""parent":{"key":"SCRUM-10"}"#)
            .body_contains(r#""customfield_10000":{"value":"blue"}"#);

        then.status(201).json_body_obj(&serde_json::json!({
            "id": "10050",
            "key": "SCRUM-50",
            "self": format!("{}/rest/api/3/issue/10050", server.base_url()),
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("create")
        .arg("--summary")
        .arg("Full create")
        .arg("--priority")
        .arg("High")
        .arg("--label")
        .arg("backend")
        .arg("--label")
        .arg("urgent")
        .arg("--parent")
        .arg("SCRUM-10")
        .arg("--field-json")
        .arg("customfield_10000={\"value\":\"blue\"}")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created issue SCRUM-50"));

    create_issue.assert();
}

#[test]
fn issue_list_builds_filtered_jql_and_can_render_csv() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let search = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/search/jql")
            .query_param(
                "jql",
                "project = SCRUM AND assignee = currentUser() AND watcher = currentUser() AND status IN (\"In Progress\") AND priority IN (\"High\") AND labels IN (\"backend\") ORDER BY created ASC",
            )
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
                        "assignee": { "displayName": "Faraz" }
                    }
                }
            ]
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("list")
        .arg("--project")
        .arg("SCRUM")
        .arg("--assignee")
        .arg("me")
        .arg("--watching")
        .arg("--status")
        .arg("In Progress")
        .arg("--priority")
        .arg("High")
        .arg("--label")
        .arg("backend")
        .arg("--order-by")
        .arg("created")
        .arg("--reverse")
        .arg("--csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("key,status,type,assignee,summary"))
        .stdout(predicate::str::contains(
            "SCRUM-2,In Progress,Story,Faraz,Task 2",
        ));

    search.assert();
}

#[test]
fn issue_clone_fetches_source_issue_and_creates_a_new_issue_with_replacements() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let source = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/SCRUM-2")
            .query_param("fields", "summary,description,labels,components,fixVersions,priority,project,issuetype,assignee,parent");

        then.status(200).json_body_obj(&serde_json::json!({
            "key": "SCRUM-2",
            "fields": {
                "summary": "Task 2",
                "description": {
                    "version": 1,
                    "type": "doc",
                    "content": [
                        { "type": "paragraph", "content": [{ "type": "text", "text": "Task 2 description" }] }
                    ]
                },
                "labels": ["backend"],
                "components": [{ "name": "API" }],
                "fixVersions": [{ "name": "1.0.0" }],
                "priority": { "name": "High" },
                "project": { "key": "SCRUM" },
                "issuetype": { "name": "Story" },
                "assignee": null,
                "parent": null
            }
        }));
    });

    let create = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue")
            .body_contains(r#""summary":"Ticket 2""#)
            .body_contains(r#""labels":["backend"]"#)
            .body_contains(r#""fixVersions":[{"name":"1.0.0"}]"#);

        then.status(201).json_body_obj(&serde_json::json!({
            "id": "10060",
            "key": "SCRUM-60",
            "self": format!("{}/rest/api/3/issue/10060", server.base_url()),
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("clone")
        .arg("SCRUM-2")
        .arg("--replace")
        .arg("Task:Ticket")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cloned SCRUM-2 to SCRUM-60"));

    source.assert();
    create.assert();
}
