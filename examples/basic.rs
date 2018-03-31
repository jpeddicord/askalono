extern crate askalono;
use std::path::Path;
use askalono::{Store, TextData};

// Note: this example is stupid slow because it loads and parses licenses
// each run instead of persisting to a cache file. Expect runs to take ~20s;
// 19.99s of that to be loading license data.

fn main() {
    // create a new license text store
    let mut store = Store::new();

    // load up data from SPDX JSON files, opting to not embed full text
    println!("Loading SPDX data, this may take a while...");
    store
        .load_spdx(
            Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/license-list-data/json/details"
            )),
            false,
        )
        .unwrap();

    // load input text
    println!("Parsing input text");
    let input: TextData = include_str!("../LICENSE").into();

    // do the heavy lifting
    println!("Scoring licenses");
    let matched = store.analyze(&input);

    println!("{:?}", matched);
}
