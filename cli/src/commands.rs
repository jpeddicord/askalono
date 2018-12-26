// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

arg_enum! {
    #[allow(non_camel_case_types)]
    pub enum OutputFormat {
        text,
        json
    }
}

#[derive(StructOpt)]
#[structopt(name = "askalono")]
pub struct Opt {

    /// Path to a cache file containing compiled license information
    #[structopt(long = "cache", short = "c", parse(from_os_str))]
    pub cache: Option<PathBuf>,

    /// Output type: text (default), json
    #[structopt(long = "format")]
    #[structopt(raw(possible_values = "&OutputFormat::variants()"))]
    pub format: Option<OutputFormat>,

    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(StructOpt)]
pub enum Subcommand {
    
    /// Identify a single file
    #[structopt(name = "identify", alias = "id")]
    Identify {
        /// File to identify
        #[structopt(name = "FILE", required_unless = "batch", parse(from_os_str))]
        filename: Option<PathBuf>,

        /// Try to find the location of a license within the given file
        #[structopt(long = "optimize", short = "o")]
        optimize: bool,

        #[structopt(raw(hidden = "true"))]
        #[structopt(long = "diff")]
        diff: bool,

        /// Read in filenames on stdin for batch identification
        #[structopt(long = "batch", short = "b")]
        batch: bool,
    },

    /// Crawl a directory identifying license files
    #[structopt(name = "crawl")]
    Crawl {
        /// Directory to crawl
        #[structopt(name = "DIR", parse(from_os_str))]
        directory: PathBuf,

        /// Follow symlinks
        #[structopt(long = "follow")]
        follow_links: bool,

        /// Glob of files to check (defaults to license-like files)
        #[structopt(long = "glob")]
        glob: Option<String>,
    },

    /// Cache management actions
    #[structopt(name = "cache")]
    Cache {
        #[structopt(subcommand)]
        subcommand: CacheSubcommand,
    },
}

#[derive(StructOpt)]
pub enum CacheSubcommand {

    /// Load an SPDX license directory (see https://github.com/spdx/license-list-data/tree/master/json/details for format)
    #[structopt(name = "load-spdx")]
    LoadSpdx {
        /// JSON "details" directory
        #[structopt(name = "DIR", parse(from_os_str))]
        dir: PathBuf,

        /// Store texts in cache along with match data
        #[structopt(long = "store")]
        store_texts: bool,
    },
}
