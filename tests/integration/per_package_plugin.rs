use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;
use std::fs;
use fs_extra::dir::{copy, CopyOptions};

#[test]
fn per_package_plugin_execution_and_output() {
    // Copy fixture to a temp directory for isolation
    let temp = tempdir().unwrap();
    let fixture = std::env::current_dir().unwrap().join("tests/fixtures/multi_workspace");
    let temp_ws = temp.path().join("multi_workspace");
    copy(&fixture, &temp, &CopyOptions::new().copy_inside(true)).unwrap();

    // Run boltpm install
    Command::cargo_bin("boltpm")
        .unwrap()
        .current_dir(&temp_ws)
        .arg("install")
        .assert()
        .success()
        .stdout(predicate::str::contains("[PLUGIN LOADER DEBUG] Using per-package plugin dir:"))
        .stdout(predicate::str::contains("PLUGIN_TEST"));

    // Assert output files exist and are correct
    let out_a = fs::read_to_string(temp_ws.join(".boltpm/plugins_output/a/PLUGIN_TEST")).unwrap();
    assert_eq!(out_a, "Plugin A executed!\n");

    let out_b = fs::read_to_string(temp_ws.join(".boltpm/plugins_output/b/PLUGIN_TEST")).unwrap();
    assert_eq!(out_b, "Plugin B executed!\n");
} 