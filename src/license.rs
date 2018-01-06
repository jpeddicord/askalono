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

#[derive(Serialize, Deserialize)]
pub struct Grams {
    pub uni: NgramSet,
    pub bi: NgramSet,
    pub tri: NgramSet,
}

impl LicenseContent {
    pub fn from_text(text: &str, store_texts: bool) -> LicenseContent {
        let normalized = apply_normalizers(text);
        let processed = apply_aggressive(&normalized);

        let bi = NgramSet::from_str(&processed, 2);
        // let (uni, bi, tri) = NgramSet::three_from_str(&processed);

        let grams = Grams {
            uni: NgramSet::new(1),
            bi,
            tri: NgramSet::new(3),
        };

        if store_texts {
            LicenseContent {
                texts: Some(Texts {
                    normalized,
                    processed,
                }),
                grams,
            }
        } else {
            LicenseContent { texts: None, grams }
        }
    }
}

impl Grams {
    pub fn combined_dice(&self, other: &Grams) -> f32 {
        // let uni = self.uni.dice(&other.uni);
        // let bi = self.bi.dice(&other.bi);
        // let tri = self.tri.dice(&other.tri);
        // (tri * 12.0 + bi * 4.0 + uni) / 17.0 // XXX: this is totally arbitrary
        self.bi.dice(&other.bi)
    }
}
