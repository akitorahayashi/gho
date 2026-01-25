//! Storage and account tests for gho.

mod common;

use common::TestContext;
use serial_test::serial;

#[test]
#[serial]
fn account_list_with_preexisting_accounts() {
    let ctx = TestContext::new();

    ctx.write_accounts(
        r#"{
        "personal": [
            {
                "id": "personal1",
                "kind": "personal",
                "username": "testuser",
                "protocol": "ssh"
            }
        ],
        "work": [],
        "active_account_id": "personal1"
    }"#,
    );

    ctx.cli()
        .args(["account", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("personal1"))
        .stdout(predicates::str::contains("testuser"))
        .stdout(predicates::str::contains("(active)"));
}

#[test]
#[serial]
fn account_use_switches_active() {
    let ctx = TestContext::new();

    ctx.write_accounts(
        r#"{
        "personal": [
            {
                "id": "first",
                "kind": "personal",
                "username": "user1",
                "protocol": "ssh"
            },
            {
                "id": "second",
                "kind": "personal",
                "username": "user2",
                "protocol": "https"
            }
        ],
        "work": [],
        "active_account_id": "first"
    }"#,
    );

    ctx.cli().args(["account", "use", "second"]).assert().success();

    let content = ctx.read_accounts();
    assert!(content.contains(r#""active_account_id": "second""#));
}

#[test]
#[serial]
fn account_remove_deletes_account() {
    let ctx = TestContext::new();

    ctx.write_accounts(
        r#"{
        "personal": [
            {
                "id": "todelete",
                "kind": "personal",
                "username": "testuser",
                "protocol": "ssh"
            }
        ],
        "work": [],
        "active_account_id": "todelete"
    }"#,
    );

    ctx.cli().args(["account", "remove", "todelete"]).assert().success();

    let content = ctx.read_accounts();
    assert!(!content.contains("todelete"));
}
