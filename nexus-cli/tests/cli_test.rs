use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("nexus-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("fetch-chapter"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("nexus-cli").unwrap();
    cmd.arg("--version").assert().success();
}

#[test]
fn test_cli_get_story_data_from_url_requires_url() {
    let mut cmd = Command::cargo_bin("nexus-cli").unwrap();
    cmd.arg("get-story-data-from-url").assert().failure();
}

#[test]
fn test_cli_fetch_chapters_requires_story_id() {
    let mut cmd = Command::cargo_bin("nexus-cli").unwrap();
    cmd.arg("fetch-chapters")
        .arg("--site")
        .arg("fanfiction")
        .assert()
        .failure();
}

#[test]
fn test_cli_unknown_site() {
    let mut cmd = Command::cargo_bin("nexus-cli").unwrap();
    cmd.arg("fetch-chapter")
        .arg("--site")
        .arg("unknown-site")
        .arg("--story-id")
        .arg("123")
        .arg("--chapter-number")
        .arg("1")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown-site"));
}

#[test]
fn test_cli_invalid_story_id() {
    let mut cmd = Command::cargo_bin("nexus-cli").unwrap();
    cmd.arg("fetch-chapter")
        .arg("--site")
        .arg("fanfiction")
        .arg("--story-id")
        .arg("not-a-number")
        .arg("--chapter-number")
        .arg("1")
        .assert()
        .failure();
}

#[test]
fn test_cli_all_commands_listed() {
    let mut cmd = Command::cargo_bin("nexus-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("fetch-chapter"))
        .stdout(predicate::str::contains("fetch-chapters"))
        .stdout(predicate::str::contains("get-story-data-from-url"));
}
