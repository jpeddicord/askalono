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

use std::collections::HashMap;

use failure::Error;

use license::TextData;
use license::LicenseType;

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
/// let store = Store::from_cache(File::open("askalono-cache.bin.gz")?)?;
/// let result = store.analyze(&TextData::from("what's this"))?;
/// # Ok(())
/// # }
/// ```
#[derive(Default, Serialize, Deserialize)]
pub struct Store {
    pub(super) licenses: HashMap<String, LicenseEntry>,
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

    pub fn add_license(&mut self, name: String, data: TextData) {
        let entry = LicenseEntry::new(data);
        self.licenses.insert(name, entry);
    }

    pub fn add_variant(&mut self, name: &str, variant: LicenseType, data: TextData) -> Result<(), Error> {
        let entry = self.licenses.get_mut(name).ok_or(format_err!("license {} not present in store", name))?;
        match variant {
            LicenseType::Alternate => {
                entry.alternates.push(data);
            },
            LicenseType::Header => {
                entry.headers.push(data);
            },
            _ => { return Err(format_err!("variant type not applicable for add_variant")); }
        };
        Ok(())
    }
}
