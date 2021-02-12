// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::match_bool)]

mod cache;
mod commands;
mod crawl;
mod formats;
mod identify;
mod util;

use std::{path::PathBuf, process::exit};

use structopt::StructOpt;

use self::commands::*;

fn main() {
    let options = Opt::from_args();

    env_logger::init();
    rayon::ThreadPoolBuilder::new().build_global().unwrap();

    let cache_file: PathBuf = options
        .cache
        .unwrap_or_else(|| "./askalono-cache.bin.zstd".into());

    let output_format = options.format.unwrap_or(OutputFormat::text);

    let res = match options.subcommand {
        Subcommand::Identify {
            filename,
            optimize,
            diff,
            batch,
        } => identify::identify(&cache_file, &output_format, filename, optimize, diff, batch),
        Subcommand::Crawl {
            directory,
            follow_links,
            glob,
        } => crawl::crawl(
            &cache_file,
            &output_format,
            &directory,
            follow_links,
            glob.as_deref(),
        ),
        Subcommand::Cache { subcommand } => cache::cache(&cache_file, subcommand),
    };
    if res.is_err() {
        exit(1);
    }
}
