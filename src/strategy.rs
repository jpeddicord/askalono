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

use failure::Error;

use store::Match;
use store::Store;
use license::TextData;


// TODO: Consider builder for scanning strategy?
// ScanStrategy::new().with_optimize().repeat_until(0.8f)
// - optimize(bool)
// - repeat_until_below(f32)
// - header_hint(bool)

// TODO: Into<TextData>

fn scan_basic<'a>(store: &'a Store, text: &TextData) -> Result<Match<'a>, Error> {
    // single pass, output result
    store.analyze(text)
}

// fn scan_optimize_single<'a>(store: &'a Store, text: &TextData) -> Result<Match<'a>, Error> {
//     // one overall pass
//     let matched = store.analyze(text);
//     // optimize result
//     let optimized = text.optimize_bounds(matched.data);
// }

// fn scan_optimize_all(store: &Store) -> Result<Vec<Match>, Error> {
//     // repeat until optimize score below threshold
//     //   overall pass
//     //   optimize -> result
//     //   white out
// }
