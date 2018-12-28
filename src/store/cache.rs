// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use failure::Error;
use flate2::read::GzDecoder;
use flate2::{Compression, GzBuilder};
use std::io::copy;
use std::io::prelude::*;

// dear reader:
// you may think "gee, msgpack is great, but what if this used bincode instead?"
// and you would be disappointed at the result -- for some reason, msgpack
// compresses better overall, even though bincode has little-to-no overhead.
// *shrug* feel free to experiment.

use rmp_serde::Serializer;
use serde::Serialize;

use crate::store::base::Store;

const CACHE_VERSION: &[u8] = b"askalono-03";

impl Store {
    /// Create a store from a cache file.
    ///
    /// This method is highly useful for quickly loading a cache, as creating
    /// one from text data is rather slow. This method can typically load
    /// the full SPDX set from disk in 200-300 ms. The cache will be
    /// sanity-checked to ensure it was generated with a similar version of
    /// askalono.
    pub fn from_cache<R>(readable: R) -> Result<Store, Error>
    where
        R: Read + Sized,
    {
        use rmp_serde::decode::from_read;

        let dec = GzDecoder::new(readable);
        {
            let extra = dec
                .header()
                .ok_or_else(|| format_err!("cache gzip header invalid"))?
                .extra()
                .ok_or_else(|| format_err!("cache gzip extra header missing"))?;
            if extra != CACHE_VERSION {
                bail!("cache version mismatch");
            }
        }

        let store = from_read(dec)?;
        Ok(store)
    }

    /// Serialize the current store.
    ///
    /// The output will be a MessagePack'd gzip'd binary stream that should be
    /// written to disk.
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
