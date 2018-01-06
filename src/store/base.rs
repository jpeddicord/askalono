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
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;

use walkdir::WalkDir;

use license::LicenseContent;

#[derive(Serialize, Deserialize)]
pub struct LicenseData {
    pub original: LicenseContent,
    pub alternates: Vec<LicenseContent>,
    pub headers: Vec<LicenseContent>,
    pub reference_score: f32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Store {
    pub licenses: HashMap<String, LicenseData>,
}

impl LicenseData {
    pub fn new(original: LicenseContent) -> LicenseData {
        LicenseData {
            original,
            alternates: Vec::new(),
            headers: Vec::new(),
            reference_score: 0.0,
        }
    }
}

impl Store {
    pub fn new() -> Store {
        Store {
            licenses: HashMap::new(),
        }
    }

    pub fn load_spdx(&mut self, dir: &str, include_texts: bool) -> Result<(), Box<Error>> {
        use json::{from_str, Value};

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() || path.extension().unwrap_or_else(|| OsStr::new("")) != "json" {
                continue;
            }

            let mut f = File::open(path)?;
            let mut data = String::new();
            f.read_to_string(&mut data)?;
            let val: Value = from_str(&data)?;

            let name = val["licenseId"].as_str().ok_or("missing licenseId")?;
            let text = val["licenseText"].as_str().ok_or("missing licenseText")?;
            let template = val["standardLicenseTemplate"].as_str();
            let header = val["standardLicenseHeader"].as_str();

            info!("Processing {}", name);

            let content = LicenseContent::from_text(text, include_texts);
            let license = self.licenses
                .entry(name.to_owned())
                .or_insert_with(|| LicenseData::new(content));

            if template.is_some() {
                let template_content = LicenseContent::from_text(template.unwrap(), include_texts);
                license.reference_score = license
                    .original
                    .grams
                    .combined_dice(&template_content.grams);
            }
            if header.is_some() {
                license.headers = vec![LicenseContent::from_text(header.unwrap(), include_texts)];
            }
        }

        Ok(())
    }
}
