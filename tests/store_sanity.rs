// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License").
// You may not use this file except in compliance with the License.
// A copy of the License is located at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// or in the "license" file accompanying this file. This file is distributed
// on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing
// permissions and limitations under the License.

extern crate askalono;

mod common;

use std::fs::File;
use std::io::prelude::*;

use askalono::TextData;

#[test]
fn store_loads() {
    let store = common::load_store();
    assert!(store.len() > 0, "store should have licenses");
}

#[test]
fn self_licenses() {
    let store = common::load_store();
    for license in &[
        "MIT",
        "BSD-2-Clause",
        "BSD-3-Clause",
        "GPL-2.0-only",
        "LGPL-2.0-only",
        "MPL-2.0",
    ] {
        let mut f = File::open(format!("{}/{}.txt", common::SPDX_TEXT, license))
            .expect(&format!("couldn't open license file '{}'", license));
        let mut text = String::new();
        f.read_to_string(&mut text).unwrap();
        let text_data: TextData = text.into();
        let matched = store.analyze(&text_data).unwrap();

        assert_eq!(license, &matched.name);
        assert_eq!(
            matched.score, 1.0f32,
            "license {} must have confidence 1 against itself, it was {}",
            license, matched.score
        );
    }
}

// this is primarily checking that we don't panic on empty text
#[test]
fn empty_match() {
    let store = common::load_store();
    let text = TextData::from("");
    let matched = store.analyze(&text).unwrap();

    assert_eq!(0.0f32, matched.score);
}
