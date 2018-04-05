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

use std::fs::File;
use std::path::Path;

use failure::Error;

use super::util::*;
use super::identify::identify_file;

pub fn crawl(
    cache_filename: &Path,
    directory: &Path,
    follow_links: bool,
    glob: Option<&str>,
) -> Result<(), Error> {
    use ignore::types::TypesBuilder;
    use ignore::WalkBuilder;

    let store = load_store(cache_filename)?;

    let mut types_builder = TypesBuilder::new();
    if let Some(globstr) = glob {
        types_builder.add("custom", globstr)?;
        types_builder.select("custom");
    } else {
        types_builder.add_defaults();
        types_builder.select("license");
    }
    let matcher = types_builder.build().unwrap();

    WalkBuilder::new(directory)
        .types(matcher)
        .follow_links(follow_links)
        .build() // TODO: build_parallel? see if it's faster overall, or if it just chokes the ID threads
        .filter_map(|e| match e.is_ok() {
            true => Some(e),
            false => {
                eprintln!("{}", e.unwrap_err());
                None
            }
        })
        .filter(|e| match e {
            &Ok(ref entry) => !entry.metadata().unwrap().is_dir(),
            &Err(_) => false,
        })
        .for_each(|e| {
            let entry = e.unwrap();
            let path = entry.path();
            println!("{}", path.display());

            if let Ok(mut reader) = File::open(path) {
                identify_file(&store, &mut reader, false, false).unwrap_or_else(|err| {
                    eprintln!("Error: {}", err);
                });
            }
        });

    Ok(())
}
