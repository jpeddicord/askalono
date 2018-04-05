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

use std::cmp::Ordering;
use std::fmt;

use failure::Error;
use rayon::prelude::*;

use license::LicenseType;
use license::TextData;
use store::base::Store;

/// Information about text that was compared against licenses in the store.
///
/// This only contains information about the overall match; to uncover more
/// data you can run methods like `optimize_bounds` on `TextData`.
///
/// Its lifetime is tied to the lifetime of the `Store` it was generated from.
#[derive(Clone)]
pub struct Match<'a> {
    /// Confidence score of the match, ranging from 0 to 1.
    pub score: f32,
    /// The name of the closest matching license in the `Store`. This will
    /// always be something that exists in the store, regardless of the score.
    pub name: String,
    /// Alternate names for the matched license.
    pub aliases: Vec<String>,
    /// The type of the license that matched. Useful to know if the match was
    /// the complete text, a header, or something else.
    pub license_type: LicenseType,
    /// A reference to the license data that matched inside the `Store`. May be
    /// useful for diagnostic purposes or to further optimize the result.
    pub data: &'a TextData,
}

/// A lighter version of Match to be used during analysis.
/// Reduces the need for cloning a bunch of fields.
struct PartialMatch<'a> {
    pub name: &'a str,
    pub score: f32,
    pub license_type: LicenseType,
    pub data: &'a TextData,
}

impl<'a> PartialOrd for PartialMatch<'a> {
    fn partial_cmp(&self, other: &PartialMatch) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl<'a> PartialEq for PartialMatch<'a> {
    fn eq(&self, other: &PartialMatch) -> bool {
        self.score.eq(&other.score) && self.name == other.name
            && self.license_type == other.license_type
    }
}

impl<'a> fmt::Debug for Match<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Match {{ score: {}, name: {}, license_type: {:?} }}",
            self.score, self.name, self.license_type
        )
    }
}

impl Store {
    pub fn analyze(&self, text: &TextData) -> Result<Match, Error> {
        let mut res: Vec<PartialMatch> = self.licenses
            .par_iter()
            .fold(Vec::new, |mut a: Vec<PartialMatch>, (name, data)| {
                a.push(PartialMatch {
                    score: data.original.match_score(text),
                    name,
                    license_type: LicenseType::Original,
                    data: &data.original,
                });
                data.alternates.iter().for_each(|alt| {
                    a.push(PartialMatch {
                        score: alt.match_score(text),
                        name,
                        license_type: LicenseType::Alternate,
                        data: alt,
                    })
                });
                data.headers.iter().for_each(|head| {
                    a.push(PartialMatch {
                        score: head.match_score(text),
                        name,
                        license_type: LicenseType::Header,
                        data: head,
                    })
                });
                a
            })
            .reduce(
                Vec::new,
                |mut a: Vec<PartialMatch>, b: Vec<PartialMatch>| {
                    a.extend(b);
                    a
                },
            );
        res.par_sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());

        let m = &res[0];
        let license = &self.licenses[m.name];
        Ok(Match {
            score: m.score,
            name: m.name.to_string(),
            license_type: m.license_type.clone(),
            aliases: license.aliases.clone(),
            data: m.data,
        })
    }
}
