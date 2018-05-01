extern crate askalono;

mod common;

use std::fs::File;
use std::io::prelude::*;

use askalono::TextData;

#[test]
fn test_store_loads() {
    let store = common::load_store();
    assert!(store.len() > 0, "store should have licenses");
}

#[test]
fn test_self_licenses() {
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
