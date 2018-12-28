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
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod license;
mod ngram;
mod preproc;
mod store;
mod strategy;

pub use crate::license::{LicenseType, TextData};
pub use crate::store::{Match, Store};
pub use crate::strategy::{ScanMode, ScanResult, ScanStrategy};
