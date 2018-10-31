// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! askalono is a crate that is Quite Good at detecting licenses from text.
//!
//! To get started, have a look at the `Store` struct, or one of the examples
//! in the `examples` directory.

#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(match_bool, useless_format))]

#[macro_use]
extern crate failure;
extern crate flate2;
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
extern crate rayon;

mod license;
mod ngram;
mod preproc;
mod store;
mod strategy;

pub use license::{LicenseType, TextData};
pub use store::{Match, Store};
pub use strategy::{ScanMode, ScanResult, ScanStrategy};
