// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{fs::File, path::Path};

use failure::Error;
use log::info;

use super::commands::*;
use askalono::Store;

pub fn cache(cache_filename: &Path, subcommand: CacheSubcommand) -> Result<(), Error> {
    match subcommand {
        CacheSubcommand::LoadSpdx { dir, store_texts } => {
            cache_load_spdx(cache_filename, &dir, store_texts)
        }
    }
}

fn cache_load_spdx(
    cache_filename: &Path,
    directory: &Path,
    store_texts: bool,
) -> Result<(), Error> {
    info!("Processing licenses...");
    let mut store = Store::new();
    store.load_spdx(directory, store_texts)?;
    let cache_file = File::create(cache_filename)?;
    store.to_cache(&cache_file)?;
    Ok(())
}
