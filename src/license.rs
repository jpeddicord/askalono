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

use ngram::NgramSet;
use preproc::{apply_aggressive, apply_normalizers};

#[derive(Serialize, Deserialize)]
pub struct LicenseContent {
    pub texts: Option<Texts>,
    pub grams: Grams,
}

#[derive(Serialize, Deserialize)]
pub struct Texts {
    pub normalized: String,
    pub processed: String,
}

// TODO: API cleanup: drop this
#[derive(Serialize, Deserialize)]
pub struct Grams {
    pub bi: NgramSet,
}

impl LicenseContent {
    pub fn from_text(text: &str, store_texts: bool) -> LicenseContent {
        let normalized = apply_normalizers(text);
        let processed = apply_aggressive(&normalized);

        let bi = NgramSet::from_str(&processed, 2);

        let grams = Grams {
            bi,
        };

        LicenseContent {
            texts: match store_texts {
                true => Some(Texts {
                    normalized,
                    processed,
                }),
                false => None,
            },
            grams,
        }
    }
}

impl Grams {
    // TODO: this can likely be dropped or mended into LicenseContent (API cleanup)
    pub fn combined_dice(&self, other: &Grams) -> f32 {
        self.bi.dice(&other.bi)
    }
}
