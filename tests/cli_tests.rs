//! CLI command tests for gho.

mod common;

use common::TestContext;
use predicates::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn version_flag_works() {
    let ctx = TestContext::new();

    ctx.cli()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
#[serial]
fn help_shows_subcommands() {
    let ctx = TestContext::new();

    ctx.cli().arg("--help").assert().success().stdout(
        predicate::str::contains("account")
            .and(predicate::str::contains("repo"))
            .and(predicate::str::contains("pr")),
    );
}

#[test]
#[serial]
fn help_shows_aliases() {
    let ctx = TestContext::new();

    ctx.cli().arg("--help").assert().success().stdout(
        predicate::str::contains("[aliases: a]")
            .and(predicate::str::contains("[aliases: r]"))
            .and(predicate::str::contains("[aliases: p]")),
    );
}

#[test]
#[serial]
fn account_list_empty() {
    let ctx = TestContext::new();

    ctx.cli()
        .args(["account", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No accounts configured"));
}

#[test]
#[serial]
fn account_list_alias_works() {
    let ctx = TestContext::new();

    ctx.cli()
        .args(["a", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No accounts configured"));
}

#[test]
#[serial]
fn account_show_without_active_fails() {
    let ctx = TestContext::new();

    ctx.cli()
        .args(["account", "show"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active account"));
}

#[test]
#[serial]
fn account_remove_nonexistent_fails() {
    let ctx = TestContext::new();

    ctx.cli()
        .args(["account", "remove", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Account not found"));
}

#[test]
#[serial]
fn account_use_nonexistent_fails() {
    let ctx = TestContext::new();

    ctx.cli()
        .args(["account", "use", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Account not found"));
}

#[test]
#[serial]
fn repo_list_without_account_fails() {
    let ctx = TestContext::new();

    ctx.cli()
        .args(["repo", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active account"));
}

#[test]
#[serial]
fn repo_clone_requires_arg_or_org() {
    let ctx = TestContext::new();

    ctx.cli()
        .args(["repo", "clone"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("provide either a repo"));
}

#[test]
#[serial]
fn pr_list_without_account_fails() {
    let ctx = TestContext::new();

    ctx.cli()
        .args(["pr", "list", "owner/repo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active account"));
}
