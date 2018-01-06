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

use rayon::prelude::*;

use store::base::Store;
use license::LicenseContent;

#[derive(Clone)]
pub struct Match<'a> {
    pub score: f32,
    pub name: String,
    pub content: &'a LicenseContent,
}

impl<'a> PartialOrd for Match<'a> {
    fn partial_cmp(&self, other: &Match) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl<'a> PartialEq for Match<'a> {
    fn eq(&self, other: &Match) -> bool {
        self.score.eq(&other.score) && self.name.eq(&other.name)
    }
}

impl<'a> fmt::Debug for Match<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Match {{ name: {}, score: {} }}", self.name, self.score)
    }
}

impl Store {
    pub fn analyze_content(&self, content: &LicenseContent) -> Match {
        let mut res: Vec<Match> = self.licenses
            .par_iter()
            .fold(Vec::new, |mut a: Vec<Match>, (name, data)| {
                a.push(Match {
                    score: data.original.grams.combined_dice(&content.grams),
                    name: name.clone(),
                    content: &data.original,
                });
                data.alternates.iter().for_each(|alt| {
                    a.push(Match {
                        score: alt.grams.combined_dice(&content.grams),
                        name: name.clone(),
                        content: alt,
                    })
                });
                data.headers.iter().for_each(|head| {
                    a.push(Match {
                        score: head.grams.combined_dice(&content.grams),
                        name: name.clone(),
                        content: head,
                    })
                });
                a
            })
            .reduce(Vec::new, |mut a: Vec<Match>, b: Vec<Match>| {
                a.extend(b);
                a
            });
        res.par_sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());

        res[0].clone()
    }
}
