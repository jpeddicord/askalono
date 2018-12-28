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

use askalono;

mod common;

use std::fs::File;
use std::io::prelude::*;

use askalono::TextData;

// Ok, this test is a bit silly. But it's neat! I think.
// This test tests that it can identify this file (this one you're reading)
// as having an Apache 2.0 header, and then tries to see where it is.

#[test]
fn self_apache_header() {
    let store = common::load_store();
    let mut f = File::open(file!()).unwrap();
    let mut text = String::new();
    f.read_to_string(&mut text).unwrap();
    let text_data: TextData = text.into();
    let matched = store.analyze(&text_data).unwrap();

    // check that it looked apache-2.0-ish
    assert_eq!("Apache-2.0", &matched.name);
    assert_eq!(askalono::LicenseType::Header, matched.license_type);

    // now try to find the bounds of the license header
    let (optimized, _) = text_data.optimize_bounds(&matched.data).unwrap();

    // the license is from (0-indexed) lines 3 thru 12 of this file, excluding
    // that copyright statement on line 1.
    assert_eq!((2, 13), optimized.lines_view());
}
