use assert_cmd::Command;
use httpmock::{
    Method::{GET, POST},
    MockServer,
};
use predicates::prelude::*;
use tempfile::TempDir;

#[path = "cli/auth_context.rs"]
mod auth_context;
#[path = "cli/common.rs"]
mod common;
#[path = "cli/issue_advanced.rs"]
mod issue_advanced;
#[path = "cli/issue_mutations.rs"]
mod issue_mutations;
#[path = "cli/issue_queries.rs"]
mod issue_queries;
#[path = "cli/planning.rs"]
mod planning;
