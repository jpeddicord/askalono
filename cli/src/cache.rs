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

use failure::Error;

use super::commands::*;
use askalono::Store;

pub fn cache(cache_filename: &Path, subcommand: CacheSubcommand) -> Result<(), Error> {
    // TODO
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
