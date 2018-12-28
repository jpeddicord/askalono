// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! askalono is a crate that is Quite Good at detecting licenses from text.
//!
//! To get started, have a look at the `Store` struct, or one of the examples
//! in the `examples` directory.

#![warn(missing_docs)]
#![allow(clippy::match_bool, clippy::useless_format)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

mod license;
mod ngram;
mod preproc;
mod store;
mod strategy;

pub use crate::license::{LicenseType, TextData};
pub use crate::store::{Match, Store};
pub use crate::strategy::{ScanMode, ScanResult, ScanStrategy};
