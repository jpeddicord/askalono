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

use std::fs::read_to_string;
use std::path::Path;

use failure::Error;
use ignore::Error as IgnoreError;

use askalono::TextData;

use super::commands::*;
use super::formats::*;
use super::identify::identify_data;
use super::util::*;

pub fn crawl(
    cache_filename: &Path,
    output_format: OutputFormat,
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
        .build()
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(error) => {
                if let IgnoreError::WithPath { path, err } = error {
                    FileResult::from_error(&path.to_string_lossy(), err)
                        .print_as(&output_format, true);
                } else {
                    FileResult::from_error("", error).print_as(&output_format, false);
                }
                None
            }
        })
        .filter(|entry| !entry.metadata().unwrap().is_dir())
        .for_each(|entry| {
            let path = entry.path();
            let path_lossy = path.to_string_lossy();

            match read_to_string(path) {
                Ok(content) => {
                    let data = TextData::new(&content);
                    let idres = identify_data(&store, &data, false, false);
                    let fileres = FileResult::from_identification_result(&path_lossy, &idres);
                    fileres.print_as(&output_format, true);
                }
                Err(err) => {
                    FileResult::from_error(&path_lossy, err).print_as(&output_format, true);
                }
            };
        });

    Ok(())
}
