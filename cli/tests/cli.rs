// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::str::from_utf8;

use serde_json::Value;

fn find_exe() -> PathBuf {
    let me = std::env::current_exe().unwrap();
    let dir = me.parent().unwrap();
    let actual_dir = if dir.ends_with("deps") {
        dir.parent().unwrap()
    } else {
        dir
    };

    actual_dir.join("askalono")
}

fn run(args: &[&str]) -> Output {
    let exe = find_exe();
    let out = Command::new(exe)
        .args(args)
        .output()
        .expect("launch failed");
    out
}

fn run_json(args: &[&str]) -> Value {
    let cat = [&["--format=json"], args].concat();
    let out = run(&cat);
    serde_json::from_str(from_utf8(&out.stdout).expect("output was not utf8"))
        .expect("output was not valid json")
}

#[test]
fn cli_sanity() {
    let out = run(&["id", "../LICENSE"]);
    assert!(out.status.success());
}

#[test]
fn output_json() {
    let json = run_json(&["id", "../LICENSE"]);
    assert_eq!("../LICENSE", json["path"]);
    assert!(
        json["result"]["score"]
            .as_f64()
            .expect("score must be a number")
            > 0.90f64
    );
    assert_eq!("Apache-2.0", json["result"]["license"]["name"]);
    assert_eq!("original", json["result"]["license"]["kind"]);
    assert_eq!(
        0,
        json["result"]["license"]["aliases"]
            .as_array()
            .expect("aliases must be an array")
            .len()
    );
    assert_eq!(
        0,
        json["result"]["containing"]
            .as_array()
            .expect("containing must be an array")
            .len()
    );
}

#[test]
fn multiple_licenses() {
    let out = run(&["id", "./tests/data/python-zeep.LICENSE"]);
    assert!(!out.status.success());

    let json = run_json(&["id", "-m", "./tests/data/python-zeep.LICENSE"]);

    assert_eq!("./tests/data/python-zeep.LICENSE", json["path"]);

    // The score is currently zero for any file with multiple licenses in it
    assert!(
         json["result"]["score"]
             .as_f64()
             .expect("score must be a number")
             == 0.0
    );

    assert_eq!("MIT", json["result"]["containing"][0]["license"]["name"]);
    assert_eq!("original", json["result"]["containing"][0]["license"]["kind"]);

    assert_eq!("BSD-3-Clause", json["result"]["containing"][1]["license"]["name"]);
    assert_eq!("original", json["result"]["containing"][1]["license"]["kind"]);

    assert_eq!("BSD-3-Clause", json["result"]["containing"][2]["license"]["name"]);
    assert_eq!("original", json["result"]["containing"][2]["license"]["kind"]);
}
