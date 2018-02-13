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

use std::io::copy;
use std::io::prelude::*;
use failure::Error;
use flate2::{Compression, GzBuilder};
use flate2::read::GzDecoder;

// dear reader:
// you may think "gee, msgpack is great, but what if this used bincode instead?"
// and you would be disappointed at the result -- for some reason, msgpack
// compresses better overall, even though bincode has little-to-no overhead.
// *shrug* feel free to experiment.

use serde::Serialize;
use rmps::Serializer;

use store::base::Store;

const CACHE_VERSION: &[u8] = b"askalono-02";

impl Store {
    pub fn from_cache<R>(readable: R) -> Result<Store, Error>
    where
        R: Read + Sized,
    {
        use rmps::decode::from_read;

        let dec = GzDecoder::new(readable);
        {
            let extra = dec.header()
                .ok_or(format_err!("cache gzip header invalid"))?
                .extra()
                .ok_or(format_err!("cache gzip extra header missing"))?;
            if extra != CACHE_VERSION {
                bail!("cache version mismatch");
            }
        }

        let store = from_read(dec)?;
        Ok(store)
    }

    pub fn to_cache<W>(&self, mut writable: W) -> Result<(), Error>
    where
        W: Write + Sized,
    {
        let mut buf = Vec::new();
        {
            let mut serializer = Serializer::new(&mut buf);
            self.serialize(&mut serializer)?;
        }

        info!("Pre-compressed output is {} bytes", buf.len());

        let mut gz = GzBuilder::new()
            .extra(CACHE_VERSION)
            .write(&mut writable, Compression::best());
        copy(&mut buf.as_slice(), &mut gz)?;

        Ok(())
    }
}
