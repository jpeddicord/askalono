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

use std::path::Path;

use failure::Error;

use askalono::{Store, TextData};

#[cfg(feature = "embedded-cache")]
static CACHE_DATA: &'static [u8] = include_bytes!(env!("ASKALONO_EMBEDDED_CACHE"));

#[allow(unused_variables)]
pub fn load_store(cache_filename: &Path) -> Result<Store, Error> {
    #[cfg(feature = "embedded-cache")]
    let store = Store::from_cache(CACHE_DATA)?;

    #[cfg(not(feature = "embedded-cache"))]
    let store = Store::from_cache_file(cache_filename)?; //FIXME: whoops, this doesn't build.

    Ok(store)
}

pub fn diff_result(license: &TextData, other: &TextData) {
    use difference::Changeset;

    let license_texts = &license.lines().expect("license texts is Some").join("\n");
    let other_texts = &other.lines().expect("other texts is Some").join("\n");

    let processed = Changeset::new(license_texts, other_texts, " ");
    println!(
        "{}\n\n---\n\n{}\n\n---\n\n{}",
        &license_texts, &other_texts, processed
    );
}
