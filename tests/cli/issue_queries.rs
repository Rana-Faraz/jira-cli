use super::common::*;
use super::*;

#[test]
fn issue_create_uses_active_context_project_and_posts_markdown_as_adf() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("ENG"));

    let create_issue = server.mock(|when, then| {
        when.method(POST)
            .path("/rest/api/3/issue")
            .header_exists("authorization")
            .header("accept", "application/json")
            .body_contains(r#""project":{"key":"ENG"}"#)
            .body_contains(r#""issuetype":{"name":"Task"}"#)
            .body_contains(r#""summary":"CLI smoke""#)
            .body_contains(r#""type":"doc""#)
            .body_contains(r#""type":"heading""#)
            .body_contains(r#""text":"Title""#)
            .body_contains(r#""type":"strong""#)
            .body_contains(r#""text":"bold""#);

        then.status(201).json_body_obj(&serde_json::json!({
            "id": "10001",
            "key": "ENG-123",
            "self": format!("{}/rest/api/3/issue/10001", server.base_url()),
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("create")
        .arg("--summary")
        .arg("CLI smoke")
        .arg("--description")
        .arg("# Title\n\nBody with **bold** text.")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created issue ENG-123"))
        .stdout(predicate::str::contains("/browse/ENG-123"));

    create_issue.assert();
}

#[test]
fn issue_list_uses_active_context_project_and_prints_issue_rows() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("ENG"));

    let list_issues = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/search/jql")
            .header_exists("authorization")
            .query_param("jql", "project = ENG ORDER BY updated DESC")
            .query_param("maxResults", "20")
            .query_param("fields", "summary,status,issuetype,assignee");

        then.status(200).json_body_obj(&serde_json::json!({
            "issues": [
                {
                    "key": "ENG-123",
                    "fields": {
                        "summary": "CLI list smoke",
                        "status": { "name": "In Progress" },
                        "issuetype": { "name": "Task" },
                        "assignee": { "displayName": "Faraz" }
                    }
                },
                {
                    "key": "ENG-124",
                    "fields": {
                        "summary": "Unassigned follow-up",
                        "status": { "name": "To Do" },
                        "issuetype": { "name": "Bug" },
                        "assignee": null
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
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ENG-123\t[In Progress]\tTask\tFaraz\tCLI list smoke",
        ))
        .stdout(predicate::str::contains(
            "ENG-124\t[To Do]\tBug\tunassigned\tUnassigned follow-up",
        ));

    list_issues.assert();
}

#[test]
fn issue_view_prints_metadata_and_renders_adf_description() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("ENG"));

    let view_issue = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/ENG-123")
            .query_param(
                "fields",
                "summary,status,issuetype,project,assignee,reporter,created,updated,description",
            )
            .header_exists("authorization");

        then.status(200).json_body_obj(&serde_json::json!({
            "key": "ENG-123",
            "fields": {
                "summary": "CLI view smoke",
                "status": { "name": "Done" },
                "issuetype": { "name": "Story" },
                "project": { "key": "ENG" },
                "assignee": { "displayName": "Faraz" },
                "reporter": { "displayName": "Admin" },
                "created": "2026-04-05T10:00:00.000+0000",
                "updated": "2026-04-05T11:00:00.000+0000",
                "description": {
                    "version": 1,
                    "type": "doc",
                    "content": [
                        {
                            "type": "heading",
                            "attrs": { "level": 1 },
                            "content": [{ "type": "text", "text": "Title" }]
                        },
                        {
                            "type": "paragraph",
                            "content": [{ "type": "text", "text": "Body text" }]
                        },
                        {
                            "type": "bulletList",
                            "content": [
                                {
                                    "type": "listItem",
                                    "content": [
                                        {
                                            "type": "paragraph",
                                            "content": [{ "type": "text", "text": "First item" }]
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            }
        }));
    });

    let comments = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/api/3/issue/ENG-123/comment")
            .query_param("maxResults", "2")
            .query_param("orderBy", "-created")
            .header_exists("authorization");

        then.status(200).json_body_obj(&serde_json::json!({
            "comments": [
                {
                    "author": { "displayName": "Reviewer" },
                    "created": "2026-04-05T11:30:00.000+0000",
                    "body": {
                        "version": 1,
                        "type": "doc",
                        "content": [
                            {
                                "type": "paragraph",
                                "content": [{ "type": "text", "text": "Looks good" }]
                            }
                        ]
                    }
                }
            ]
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("issue")
        .arg("view")
        .arg("ENG-123")
        .arg("--comments")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("ENG-123"))
        .stdout(predicate::str::contains("Summary: CLI view smoke"))
        .stdout(predicate::str::contains("Status: Done"))
        .stdout(predicate::str::contains("Type: Story"))
        .stdout(predicate::str::contains("Project: ENG"))
        .stdout(predicate::str::contains("Assignee: Faraz"))
        .stdout(predicate::str::contains("Reporter: Admin"))
        .stdout(predicate::str::contains("Description:"))
        .stdout(predicate::str::contains("Title"))
        .stdout(predicate::str::contains("Body text"))
        .stdout(predicate::str::contains("First item"))
        .stdout(predicate::str::contains("Comments:"))
        .stdout(predicate::str::contains("Reviewer"))
        .stdout(predicate::str::contains("Looks good"));

    view_issue.assert();
    comments.assert();
}

#[test]
fn me_prints_the_current_user_profile() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("ENG"));

    let me = server.mock(|when, then| {
        when.method(GET).path("/rest/api/3/myself");

        then.status(200).json_body_obj(&serde_json::json!({
            "accountId": "abc-123",
            "displayName": "Faraz",
            "emailAddress": "faraz@example.com",
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("me")
        .assert()
        .success()
        .stdout(predicate::str::contains("Faraz"))
        .stdout(predicate::str::contains("abc-123"))
        .stdout(predicate::str::contains("faraz@example.com"));

    me.assert();
}

#[test]
fn project_list_prints_accessible_projects() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("ENG"));

    let projects = server.mock(|when, then| {
        when.method(GET).path("/rest/api/3/project/search");

        then.status(200).json_body_obj(&serde_json::json!({
            "values": [
                { "id": "10000", "key": "SCRUM", "name": "Front-End Dev", "projectTypeKey": "software" },
                { "id": "10001", "key": "OPS", "name": "Operations", "projectTypeKey": "service_desk" }
            ]
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("project")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("SCRUM\tFront-End Dev\tsoftware"))
        .stdout(predicate::str::contains("OPS\tOperations\tservice_desk"));

    projects.assert();
}

#[test]
fn board_list_filters_by_project_and_prints_boards() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let boards = server.mock(|when, then| {
        when.method(GET)
            .path("/rest/agile/1.0/board")
            .query_param("projectKeyOrId", "SCRUM");

        then.status(200).json_body_obj(&serde_json::json!({
            "values": [
                {
                    "id": 1,
                    "name": "SCRUM board",
                    "type": "simple",
                    "location": { "projectKey": "SCRUM", "projectName": "Front-End Dev" }
                }
            ]
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("board")
        .arg("list")
        .arg("--project")
        .arg("SCRUM")
        .assert()
        .success()
        .stdout(predicate::str::contains("1\tSCRUM board\tSCRUM\tsimple"));

    boards.assert();
}

#[test]
fn release_list_prints_project_versions() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let releases = server.mock(|when, then| {
        when.method(GET).path("/rest/api/3/project/SCRUM/versions");

        then.status(200).json_body_obj(&serde_json::json!([
            { "id": "10000", "name": "1.0.0", "released": false, "archived": false },
            { "id": "10001", "name": "0.9.0", "released": true, "archived": false }
        ]));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("release")
        .arg("list")
        .arg("--project")
        .arg("SCRUM")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "10000\t1.0.0\treleased=false\tarchived=false",
        ))
        .stdout(predicate::str::contains(
            "10001\t0.9.0\treleased=true\tarchived=false",
        ));

    releases.assert();
}

#[test]
fn serverinfo_prints_instance_metadata() {
    let temp = TempDir::new().expect("create temp dir");
    let server = MockServer::start();
    write_mock_config(&temp, &server.base_url(), Some("SCRUM"));

    let serverinfo = server.mock(|when, then| {
        when.method(GET).path("/rest/api/3/serverInfo");

        then.status(200).json_body_obj(&serde_json::json!({
            "deploymentType": "Cloud",
            "version": "1001.0.0-SNAPSHOT",
            "buildNumber": 100234,
            "serverTitle": "Faraz Jira"
        }));
    });

    let mut cmd = Command::cargo_bin("jira").expect("build jira binary");
    cmd.env("JIRA_CONFIG_DIR", temp.path())
        .env("JIRA_TOKEN", "dummy-token")
        .arg("serverinfo")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deployment: Cloud"))
        .stdout(predicate::str::contains("Version: 1001.0.0-SNAPSHOT"))
        .stdout(predicate::str::contains("Build: 100234"))
        .stdout(predicate::str::contains("Title: Faraz Jira"));

    serverinfo.assert();
}
