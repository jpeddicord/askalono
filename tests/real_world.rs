// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

extern crate askalono;

mod common;

use std::fs::read_dir;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;

use askalono::{Store, TextData};

const TEST_DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/real-licenses");

#[test]
fn real_world_licenses() {
    let store = common::load_store();

    for dir in read_dir(TEST_DATA).unwrap() {
        let dir = dir.unwrap();
        let dir_path = dir.path();
        let license_id = dir_path.file_name().unwrap().to_string_lossy();
        if dir_path.is_dir() {
            for file in read_dir(&dir_path).unwrap() {
                let file = file.unwrap();
                let file_path = file.path();
                let name = file_path.file_name().unwrap().to_string_lossy();
                let parts: Vec<_> = name.split("__").collect();
                let test_name = &parts[0];
                let confidence = f32::from_str(parts[1]).unwrap();

                assert_license(&store, &file_path, &test_name, &license_id, confidence);
            }
        }
    }
}

fn assert_license(
    store: &Store,
    path: &Path,
    test_name: &str,
    license_id: &str,
    min_confidence: f32,
) {
    let mut f = File::open(path).unwrap();
    let mut text = String::new();
    f.read_to_string(&mut text).unwrap();
    let text_data: TextData = text.into();

    let matched = store.analyze(&text_data).unwrap();
    assert_eq!(
        license_id, matched.name,
        "{} was identified as {} but should have been {}",
        test_name, matched.name, license_id
    );
    assert!(
        matched.score >= min_confidence,
        "{} scored {} but needed at least {}",
        test_name,
        matched.score,
        min_confidence
    );
}
