mod common;
use common::load_testcase;

#[test]
fn load_path_dependency() {
    let workspace = load_testcase("workspace_load");
    let loaded = workspace.list();
    assert!(loaded.contains(&String::from("entry")));
}

#[test]
fn ignore_external_dependency() {
    let workspace = load_testcase("workspace_load");
    let loaded = workspace.list();
    assert!(!loaded.contains(&String::from("anyhow")));
    assert!(!loaded.contains(&String::from("syn")));
}

#[test]
fn ignore_outer_path_dependency() {
    let workspace = load_testcase("workspace_load");
    let loaded = workspace.list();
    assert!(!loaded.contains(&String::from("bundle")));
}

#[test]
#[should_panic(expected = "renaming dependencies `entry` to `rename` not supported")]
fn error_rename_dependency() {
    let _ = load_testcase("workspace_load_rename");
}
