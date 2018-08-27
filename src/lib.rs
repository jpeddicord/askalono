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

//! askalono is a crate that is Quite Good at detecting licenses from text.
//!
//! To get started, have a look at the `Store` struct, or one of the examples
//! in the `examples` directory.

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(match_bool, useless_format))]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate rmp_serde as rmps;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate unicode_normalization;

#[cfg(feature = "spdx")]
extern crate serde_json as json;

#[cfg(not(target_arch = "wasm32"))]
extern crate flate2;
#[cfg(not(target_arch = "wasm32"))]
extern crate rayon;

mod license;
mod ngram;
mod preproc;
mod store;
mod strategy;

pub use license::{LicenseType, TextData};
pub use store::{Match, Store};
pub use strategy::{ScanStrategy, ScanResult};
