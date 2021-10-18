// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{
    ffi::OsStr,
    fs::{read_dir, File},
    io::prelude::*,
    path::Path,
};

use anyhow::{format_err, Error};
use log::{debug, info};

use crate::{
    license::TextData,
    store::base::{LicenseEntry, Store},
};

impl Store {
    /// Fill the store with SPDX JSON data.
    ///
    /// This function is very specific to the format of SPDX's
    /// `license-list-data` repository. It reads all JSON files in the
    /// `json/details` directory and creates entries inside the store for
    /// matching.
    ///
    /// This is intended to be used during build of askalono, so it's not
    /// available unless the `spdx` feature is enabled.
    ///
    /// `include_texts`, if true, will keep normalized license text data inside
    /// the store. This yields a larger store when serialized, but has the
    /// benefit of allowing you to diff your result against what askalono has
    /// stored.
    pub fn load_spdx(&mut self, dir: &Path, include_texts: bool) -> Result<(), Error> {
        use serde_json::{from_str, Value};

        // locate all json files in the directory
        let mut paths: Vec<_> = read_dir(dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().unwrap_or_else(|| OsStr::new("")) == "json")
            .collect();

        // sort without extensions; otherwise dashes and dots muck it up
        paths.sort_by(|a, b| a.file_stem().unwrap().cmp(b.file_stem().unwrap()));

        for path in paths {
            let mut f = File::open(path)?;
            let mut data = String::new();
            f.read_to_string(&mut data)?;
            let val: Value = from_str(&data)?;

            let name = val["licenseId"]
                .as_str()
                .ok_or_else(|| format_err!("missing licenseId"))?;

            let deprecated = val["isDeprecatedLicenseId"]
                .as_bool()
                .ok_or_else(|| format_err!("missing isDeprecatedLicenseId"))?;
            if deprecated {
                debug!("Skipping {} (deprecated)", name);
                continue;
            }

            let text = val["licenseText"]
                .as_str()
                .ok_or_else(|| format_err!("missing licenseText"))?;
            let header = val["standardLicenseHeader"].as_str();

            info!("Processing {}", name);

            let content = match include_texts {
                true => TextData::new(text),
                false => TextData::new(text).without_text(),
            };

            // check if an identical license is already present
            let mut already_existed = false;
            self.licenses.iter_mut().for_each(|(key, ref mut value)| {
                if value.original.eq_data(&content) {
                    value.aliases.push(name.to_string());
                    info!("{} already stored; added as an alias for {}", name, key);
                    already_existed = true;
                }
            });

            if already_existed {
                continue;
            }

            let license = self
                .licenses
                .entry(name.to_owned())
                .or_insert_with(|| LicenseEntry::new(content));

            if let Some(header_text) = header {
                let header_data = match include_texts {
                    false => TextData::new(header_text),
                    true => TextData::new(header_text).without_text(),
                };
                license.headers = vec![header_data];
            }
        }

        Ok(())
    }
}
