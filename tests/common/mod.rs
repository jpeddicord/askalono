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

use std::fs::File;
use std::path::Path;

use askalono::Store;

#[allow(dead_code)]
pub const SPDX_TEXT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/spdx-text");
pub const SPDX_JSON: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/datasets/spdx-json");
pub const TEST_CACHE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test-cache.bin.gz");

pub fn load_store() -> Store {
    if Path::new(TEST_CACHE).exists() {
        return Store::from_cache(&File::open(TEST_CACHE).unwrap()).unwrap();
    }

    let mut store = Store::new();
    store
        .load_spdx(Path::new(SPDX_JSON), true)
        .expect("Couldn't create a store from SPDX data (needed for tests). Have submodules been initialized?");
    let mut cache = File::create(TEST_CACHE).unwrap();
    store.to_cache(&mut cache).unwrap();

    store
}
