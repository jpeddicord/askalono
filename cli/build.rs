// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::fs::{remove_file, File};
use std::path::Path;

use askalono::Store;

const EMBEDDED_CACHE: &str = "embedded-cache.bin.gz";

fn main() {
    if env::var("CARGO_FEATURE_EMBEDDED_CACHE").is_err() {
        println!("cargo:warning=askalono embedded cache feature disabled");
        remove_file(EMBEDDED_CACHE).ok(); // don't care if this succeeds
        return;
    }

    println!(
        "cargo:rustc-env=ASKALONO_EMBEDDED_CACHE=../{}",
        EMBEDDED_CACHE
    );

    if Path::new(EMBEDDED_CACHE).exists() {
        println!("cargo:warning=askalono cache file already exists; not re-building");
        return;
    }

    let store_texts = env::var("CARGO_FEATURE_DIAGNOSTICS").is_ok();

    let mut store = Store::new();
    store
        .load_spdx(Path::new("../datasets/spdx-json"), store_texts)
        .expect("Couldn't create a store from SPDX data. Have submodules been initialized?");
    let mut cache = File::create(EMBEDDED_CACHE).unwrap();
    store.to_cache(&mut cache).unwrap();
}
