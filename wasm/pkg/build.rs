// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

extern crate askalono;

use std::fs::File;
use std::path::Path;

use askalono::Store;

const EMBEDDED_CACHE: &str = "wasm-embedded-cache.bin.gz";

// copied over from the CLI

fn main() {
    println!(
        "cargo:rustc-env=ASKALONO_WASM_EMBEDDED_CACHE=../{}",
        EMBEDDED_CACHE
    );

    if Path::new(EMBEDDED_CACHE).exists() {
        println!("cargo:warning=askalono wasm cache file already exists; not re-building");
        return;
    }

    let mut store = Store::new();
    store
        .load_spdx(Path::new("../datasets/spdx-json"), true)
        .expect("Couldn't create a store from SPDX data. Have submodules been initialized?");
    let mut cache = File::create(EMBEDDED_CACHE).unwrap();
    store.to_cache(&mut cache).unwrap();
}
