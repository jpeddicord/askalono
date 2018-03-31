extern crate askalono;

use std::fs::File;
use std::io::Read;
use askalono::{Store, TextData};

macro_rules! license_path {
    ($name:expr) => {
        format!("{}/../license-list-data/text/{}.txt", env!("CARGO_MANIFEST_DIR"), $name)
    }
}

macro_rules! assert_license {
    ($store:expr, $name:expr, $min_score:expr) => {
        let file_path = license_path!($name);
        let mut f = File::open(file_path).unwrap();
        let mut text = String::new();
        f.read_to_string(&mut text).unwrap();
        let text_data: TextData = text.into();
        let matched = $store.analyze(&text_data).unwrap();

        assert_eq!($name, matched.name);
        assert!(matched.score >= $min_score);
    };
}

// TODO: these tests are really for the library, not the CLI.
// they should be re-located once we've figured out a good way
// to set up the cache in the library for tests.

#[test]
fn everything_in_one_function() {
    let cache_file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/embedded-cache.bin.gz");
    let file = File::open(cache_file_path).unwrap();
    let store = Store::from_cache(&file).unwrap();
    assert_license!(store, "MIT", 0.99);
    assert_license!(store, "MIT-advertising", 0.99);
    assert_license!(store, "Apache-1.0", 0.99);
    assert_license!(store, "Apache-1.1", 0.99);
    assert_license!(store, "Apache-2.0", 0.99);
    assert_license!(store, "MPL-1.0", 0.99);
    assert_license!(store, "MPL-1.1", 0.99);
    // there's an oddity with this license in SPDX:
    // it and MPL-2.0-no-copyleft-exception are identical.
    // https://github.com/spdx/license-list-XML/issues/441
    // assert_license!(store, "MPL-2.0", 0.99);
    assert_license!(store, "AFL-1.1", 0.99);
    assert_license!(store, "AFL-1.2", 0.99);
    assert_license!(store, "AFL-2.0", 0.99);
    assert_license!(store, "AFL-2.1", 0.99);
    assert_license!(store, "AFL-3.0", 0.99);
}
