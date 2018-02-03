extern crate askalono;
use std::io::Read;

macro_rules! license_path {
    ($name:expr) => {
        format!("{}/license-list-data/text/{}.txt", env!("CARGO_MANIFEST_DIR"), $name)
    }
}

macro_rules! assert_license {
    ($store:expr, $name:expr, $min_score:expr) => {
        let file_path = license_path!($name);
        let mut f = std::fs::File::open(file_path).unwrap();
        let mut text = String::new();
        f.read_to_string(&mut text).unwrap();
        let content = askalono::LicenseContent::from_text(&text, false);
        let matched = $store.analyze_content(&content);

        assert_eq!($name, matched.name);
        assert!(matched.score >= $min_score);
    };
}

#[test]
fn everything_in_one_function() {
    let cache_file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/cli/embedded-cache.bin.gz");
    let store = askalono::Store::from_cache_file(cache_file_path).unwrap();
    assert_license!(store, "MIT", 0.99);
    assert_license!(store, "MIT-advertising", 0.99);
    assert_license!(store, "Apache-1.0", 0.99);
    assert_license!(store, "Apache-1.1", 0.99);
    assert_license!(store, "Apache-2.0", 0.99);
    assert_license!(store, "MPL-1.0", 0.99);
    assert_license!(store, "MPL-1.1", 0.99);
    assert_license!(store, "MPL-2.0", 0.99);
    assert_license!(store, "AFL-1.1", 0.99);
    assert_license!(store, "AFL-1.2", 0.99);
    assert_license!(store, "AFL-2.0", 0.99);
    assert_license!(store, "AFL-2.1", 0.99);
    assert_license!(store, "AFL-3.0", 0.99);
}