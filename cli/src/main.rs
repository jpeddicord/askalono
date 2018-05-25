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

#![cfg_attr(feature = "cargo-clippy", allow(match_bool))]

extern crate askalono;
#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate failure;
extern crate ignore;
#[macro_use]
extern crate log;
extern crate rayon;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate structopt;

#[cfg(feature = "diagnostics")]
extern crate difference;

mod cache;
mod commands;
mod crawl;
mod formats;
mod identify;
mod util;

use std::path::PathBuf;
use std::process::exit;

use structopt::StructOpt;

use self::commands::*;

fn main() {
    let options = Opt::from_args();

    env_logger::init();
    rayon::ThreadPoolBuilder::new().build_global().unwrap();

    let cache_file: PathBuf = options
        .cache
        .unwrap_or_else(|| "./askalono-cache.bin.gz".into());

    let output_format = options.format.unwrap_or(OutputFormat::text);

    if let Err(_) = match options.subcommand {
        Subcommand::Identify {
            filename,
            optimize,
            diff,
            batch,
        } => identify::identify(&cache_file, output_format, filename, optimize, diff, batch),
        Subcommand::Crawl {
            directory,
            follow_links,
            glob,
        } => crawl::crawl(
            &cache_file,
            output_format,
            &directory,
            follow_links,
            glob.as_ref().map(String::as_str),
        ),
        Subcommand::Cache { subcommand } => cache::cache(&cache_file, subcommand),
    } {
        exit(1);
    }
}
