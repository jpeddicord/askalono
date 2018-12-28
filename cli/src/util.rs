// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

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
    let store = {
        use std::fs::File;
        Store::from_cache(File::open(cache_filename)?)?
    };

    Ok(store)
}

#[allow(unused_variables)]
pub fn diff_result(license: &TextData, other: &TextData) {
    #[cfg(feature = "diagnostics")]
    {
        use difference::Changeset;

        let license_texts = &license.text_processed().expect("license texts is stored");
        let other_texts = &other.text_processed().expect("other texts is stored");

        let processed = Changeset::new(license_texts, other_texts, " ");
        println!(
            "{}\n\n---\n\n{}\n\n---\n\n{}",
            &license_texts, &other_texts, processed
        );
    }

    #[cfg(not(feature = "diagnostics"))]
    println!("askalono wasn't compiled with diagnostics enabled. diff not available.");
}
