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

#![doc(hidden)]
/// This module is experimental. Its API should not be considered stable
/// in any form.

use std::borrow::Cow;

use failure::Error;

use store::Match;
use store::Store;
use license::TextData;
use license::LicenseType;

#[derive(Serialize, Debug)]
pub struct IdentifiedLicense {
    pub name: String,
    pub kind: LicenseType,
}

#[derive(Serialize, Debug)]
pub struct ScanResult {
    pub score: f32,
    pub license: Option<IdentifiedLicense>,
    pub containing: Vec<ContainedResult>,
}

#[derive(Serialize, Debug)]
pub struct ContainedResult {
    pub score: f32,
    pub license: IdentifiedLicense,
    pub line_range: (usize, usize),
}

pub struct ScanStrategy<'a> {
    store: &'a Store,
    // fast_finish: f32
    confidence_threshold: f32,
    optimize: bool,
    find_all: bool,
    //find_all_threshold: f32,
}

impl<'a> ScanStrategy<'a> {

    pub fn new(store: &'a Store) -> ScanStrategy<'a> {
        Self {
            store,
            confidence_threshold: 0.8,
            optimize: false,
            find_all: false,
            //find_all_threshold: 0.1234, // ??? not yet determined a good value
        }
    }

    pub fn confidence_threshold(mut self, confidence_threshold: f32) -> Self {
        self.confidence_threshold = confidence_threshold;
        self
    }

    pub fn optimize(mut self, optimize: bool) -> Self {
        self.optimize = optimize;
        self
    }


    pub fn scan(&self, text: &TextData) -> Result<ScanResult, Error> {
        let mut analysis = self.store.analyze(text)?;
        let mut containing = Vec::new();

        // if we're only doing shallow analysis, bail out here
        if analysis.score > 0.98 { // TODO
            return Ok(ScanResult {
                score: analysis.score,
                license: Some(IdentifiedLicense {
                    name: analysis.name,
                    kind: analysis.license_type
                }),
                containing,
            });
        }

        // repeatedly try to dig deeper
        // this loop effectively iterates once for each license it finds
        let mut current_text: Cow<TextData> = Cow::Borrowed(text);
        loop {
            let (optimized, optimized_score) = current_text.optimize_bounds(analysis.data);

            // stop if we didn't find anything acceptable
            if optimized_score < 0.6 { // TODO
                break
            }

            // otherwise, save it
            containing.push(ContainedResult {
                score: optimized_score,
                license: IdentifiedLicense {
                    name: analysis.name,
                    kind: analysis.license_type
                },
                line_range: optimized.lines_view(),
            });

            // and white-out + reanalyze for next iteration
            current_text = Cow::Owned(optimized.white_out().expect("optimized must have text"));
            analysis = self.store.analyze(&current_text)?;

        }


        Ok(ScanResult {
            score: 0.0f32,
            license: None,
            containing: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FIXME: bad
    #[test]
    fn test_builder_works() {
        let store = Store::new();
        ScanStrategy::new(&store).confidence_threshold(0.5);
    }
}