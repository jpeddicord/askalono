// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::Parser;

use clap::ValueEnum;

#[derive(Clone, ValueEnum)]
#[clap(rename_all = "lower")]
#[allow(clippy::upper_case_acronyms)]
pub enum OutputFormat {
    Text,
    JSON,
}

#[derive(Parser)]
#[clap(name = "askalono", version)]
pub struct Opt {
    /// Path to a cache file containing compiled license information
    #[clap(long = "cache", short = 'c')]
    pub cache: Option<PathBuf>,

    /// Output type: text (default), json
    #[clap(long = "format")]
    #[arg(value_enum)]
    pub format: Option<OutputFormat>,

    #[clap(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Parser)]
pub enum Subcommand {
    /// Identify a single file
    #[clap(name = "identify", alias = "id")]
    Identify {
        /// File to identify
        #[clap(name = "FILE", required_unless_present("batch"))]
        filename: Option<PathBuf>,

        /// Try to find the location of a license within the given file
        #[clap(long = "optimize", short = 'o')]
        optimize: bool,

        #[clap(long = "diff", hide = true)]
        diff: bool,

        /// Read in filenames on stdin for batch identification
        #[clap(long = "batch", short = 'b')]
        batch: bool,

        /// Detect multiple licenses in the same file
        #[structopt(long = "multiple", short = "m")]
        topdown: bool,
    },

    /// Crawl a directory identifying license files
    #[clap(name = "crawl")]
    Crawl {
        /// Directory to crawl
        #[clap(name = "DIR")]
        directory: PathBuf,

        /// Follow symlinks
        #[clap(long = "follow")]
        follow_links: bool,

        /// Glob of files to check (defaults to license-like files)
        #[clap(long = "glob")]
        glob: Option<String>,
    },

    /// Cache management actions
    #[clap(name = "cache")]
    Cache {
        #[clap(subcommand)]
        subcommand: CacheSubcommand,
    },
}

#[derive(Parser)]
pub enum CacheSubcommand {
    /// Load an SPDX license directory (see https://github.com/spdx/license-list-data/tree/master/json/details for format)
    #[clap(name = "load-spdx")]
    LoadSpdx {
        /// JSON "details" directory
        #[clap(name = "DIR")]
        dir: PathBuf,

        /// Store texts in cache along with match data
        #[clap(long = "store")]
        store_texts: bool,
    },
}
