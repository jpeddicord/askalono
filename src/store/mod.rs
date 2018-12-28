// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

mod analyze;
mod base;
mod cache;

#[cfg(feature = "spdx")]
mod spdx;

pub use self::{analyze::Match, base::Store};
