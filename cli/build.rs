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

    let mut store = Store::new();
    store
        .load_spdx(Path::new("../license-list-data/json/details"), false)
        .unwrap();
    let mut cache = File::create(EMBEDDED_CACHE).unwrap();
    store.to_cache(&mut cache).unwrap();
}
