use std::fs::File;
use std::path::Path;

use askalono::Store;

pub const SPDX_TEXT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/license-list-data/text");
pub const SPDX_JSON: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/license-list-data/json/details"
);
pub const TEST_CACHE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test-cache.bin.gz");

pub fn load_store() -> Store {
    if Path::new(TEST_CACHE).exists() {
        return Store::from_cache(&File::open(TEST_CACHE).unwrap()).unwrap();
    }

    let mut store = Store::new();
    store
        .load_spdx(Path::new(SPDX_JSON), false)
        .expect("Couldn't create a store from SPDX data (needed for tests). Have submodules been initialized?");
    let mut cache = File::create(TEST_CACHE).unwrap();
    store.to_cache(&mut cache).unwrap();

    store
}
