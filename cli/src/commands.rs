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

use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(name = "askalono")]
pub struct Opt {
    #[structopt(long = "cache", short = "c", parse(from_os_str))]
    pub cache: Option<PathBuf>,
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(StructOpt)]
pub enum Subcommand {
    #[structopt(name = "identify", alias = "id")]
    Identify {
        #[structopt(
            name = "FILE", help = "file to identify", required_unless = "batch", parse(from_os_str)
        )]
        filename: Option<PathBuf>,
        #[structopt(
            long = "optimize",
            short = "o",
            help = "try to find the location of a license within the file"
        )]
        optimize: bool,
        #[structopt(long = "diff", help = "print a colored diff of match (debugging feature)")]
        diff: bool,
        // #[structopt(long = "output", short = "o", help = "output type")]
        // output: Option<OutputType>, // "json"
        #[structopt(long = "batch", short = "b", help = "read in filenames on stdin")]
        batch: bool,
    },
    #[structopt(name = "crawl")]
    Crawl {
        #[structopt(name = "DIR", help = "directory to crawl", parse(from_os_str))]
        directory: PathBuf,
        #[structopt(long = "follow", help = "follow symlinks")]
        follow_links: bool,
        #[structopt(
            long = "glob", help = "glob of files to check (defaults to license-like files)"
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
        #[structopt(name = "DIR", help = "JSON details directory", parse(from_os_str))]
        dir: PathBuf,
        #[structopt(long = "store", help = "store texts in cache along with match data")]
        store_texts: bool,
    },
}
