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
}
