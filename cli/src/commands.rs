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
    #[structopt(long = "cache", short = "c", parse(from_os_str))]
    pub cache: Option<PathBuf>,
    #[structopt(long = "format", help = "output type: text (default), json")]
    #[structopt(raw(possible_values = "&OutputFormat::variants()"))]
    pub format: Option<OutputFormat>,
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(StructOpt)]
pub enum Subcommand {
    #[structopt(name = "identify", alias = "id")]
    Identify {
        #[structopt(
            name = "FILE",
            help = "file to identify",
            required_unless = "batch",
            parse(from_os_str)
        )]
        filename: Option<PathBuf>,
        #[structopt(
            long = "optimize",
            short = "o",
            help = "try to find the location of a license within the file"
        )]
        optimize: bool,
        #[structopt(raw(hidden = "true"))]
        #[structopt(long = "diff")]
        diff: bool,
        #[structopt(
            long = "batch",
            short = "b",
            help = "read in filenames on stdin"
        )]
        batch: bool,
    },
    #[structopt(name = "crawl")]
    Crawl {
        #[structopt(
            name = "DIR",
            help = "directory to crawl",
            parse(from_os_str)
        )]
        directory: PathBuf,
        #[structopt(long = "follow", help = "follow symlinks")]
        follow_links: bool,
        #[structopt(
            long = "glob",
            help = "glob of files to check (defaults to license-like files)"
        )]
        glob: Option<String>,
    },
    #[structopt(name = "cache")]
    Cache {
        #[structopt(subcommand)]
        subcommand: CacheSubcommand,
    },
}

#[derive(StructOpt)]
pub enum CacheSubcommand {
    #[structopt(name = "load-spdx")]
    LoadSpdx {
        #[structopt(
            name = "DIR",
            help = "JSON details directory",
            parse(from_os_str)
        )]
        dir: PathBuf,
        #[structopt(
            long = "store",
            help = "store texts in cache along with match data"
        )]
        store_texts: bool,
    },
}
