// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use anyhow::{format_err, Error};
use serde::{Deserialize, Serialize};

use crate::{license::LicenseType, license::TextData};

#[derive(Serialize, Deserialize)]
pub(crate) struct LicenseEntry {
    pub original: TextData,
    pub aliases: Vec<String>,
    pub headers: Vec<TextData>,
    pub alternates: Vec<TextData>,
}

/// A representation of a collection of known licenses.
///
/// This struct is generally what you want to start with if you're looking to
/// match text against a database of licenses. Load a cache from disk using
/// `from_cache`, then use the `analyze` function to determine what a text most
/// closely matches.
///
/// # Examples
///
/// ```rust,should_panic
/// # use std::fs::File;
/// # use std::error::Error;
/// use askalono::{Store, TextData};
///
/// # fn main() -> Result<(), Box<Error>> {
/// let store = Store::from_cache(File::open("askalono-cache.bin.zstd")?)?;
/// let result = store.analyze(&TextData::from("what's this"));
/// # Ok(())
/// # }
/// ```
#[derive(Default, Serialize, Deserialize)]
pub struct Store {
    pub(crate) licenses: HashMap<String, LicenseEntry>,
}

impl LicenseEntry {
    pub fn new(original: TextData) -> LicenseEntry {
        LicenseEntry {
            original,
            aliases: Vec::new(),
            alternates: Vec::new(),
            headers: Vec::new(),
        }
    }
}

impl Store {
    /// Create a new `Store`.
    ///
    /// More often, you probably want to use `from_cache` instead of creating
    /// an empty store.
    pub fn new() -> Store {
        Store {
            licenses: HashMap::new(),
        }
    }

    /// Get the number of licenses in the store.
    ///
    /// This only counts licenses by name -- headers, aliases, and alternates
    /// aren't included in the count.
    pub fn len(&self) -> usize {
        self.licenses.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.licenses.is_empty()
    }

    /// Get all licenses by name via iterator.
    pub fn licenses<'a>(&'a self) -> impl Iterator<Item = &String> + 'a {
        self.licenses.keys()
    }

    /// Get a license's standard TextData by name.
    pub fn get_original(&self, name: &str) -> Option<&TextData> {
        Some(&self.licenses.get(name)?.original)
    }

    /// Add a single license to the store.
    ///
    /// If the license with the given name already existed, it and all of its
    /// variants will be replaced.
    pub fn add_license(&mut self, name: String, data: TextData) {
        let entry = LicenseEntry::new(data);
        self.licenses.insert(name, entry);
    }

    /// Add a variant (a header or alternate formatting) of a given license to
    /// the store.
    ///
    /// The license must already exist. This function cannot be used to replace
    /// the original/canonical text of the license.
    pub fn add_variant(
        &mut self,
        name: &str,
        variant: LicenseType,
        data: TextData,
    ) -> Result<(), Error> {
        let entry = self
            .licenses
            .get_mut(name)
            .ok_or_else(|| format_err!("license {} not present in store", name))?;
        match variant {
            LicenseType::Alternate => {
                entry.alternates.push(data);
            }
            LicenseType::Header => {
                entry.headers.push(data);
            }
            _ => {
                return Err(format_err!("variant type not applicable for add_variant"));
            }
        };
        Ok(())
    }

    /// Get the list of aliases for a given license.
    pub fn aliases(&self, name: &str) -> Result<&Vec<String>, Error> {
        let entry = self
            .licenses
            .get(name)
            .ok_or_else(|| format_err!("license {} not present in store", name))?;
        Ok(&entry.aliases)
    }

    /// Set the list of aliases for a given license.
    pub fn set_aliases(&mut self, name: &str, aliases: Vec<String>) -> Result<(), Error> {
        let entry = self
            .licenses
            .get_mut(name)
            .ok_or_else(|| format_err!("license {} not present in store", name))?;
        entry.aliases = aliases;
        Ok(())
    }
}
