// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{io::copy, io::prelude::*};

use anyhow::Error;
use log::info;
use rmp_serde::Serializer;
use serde::Serialize;

use crate::store::base::Store;

const CACHE_VERSION: &[u8] = b"askalono-04";

impl Store {
    /// Create a store from a cache file.
    ///
    /// This method is highly useful for quickly loading a cache, as creating
    /// one from text data is rather slow. This method can typically load
    /// the full SPDX set from disk in 200-300 ms. The cache will be
    /// sanity-checked to ensure it was generated with a similar version of
    /// askalono.
    pub fn from_cache<R>(mut readable: R) -> Result<Store, Error>
    where
        R: Read + Sized,
    {
        let mut header = [0u8; 11];
        readable.read_exact(&mut header)?;

        if header != CACHE_VERSION {
            anyhow::bail!("cache version mismatch");
        }

        #[cfg(not(feature = "gzip"))]
        let dec = zstd::Decoder::new(readable)?;
        #[cfg(feature = "gzip")]
        let dec = flate2::read::GzDecoder::new(readable);

        let store = rmp_serde::decode::from_read(dec)?;
        Ok(store)
    }

    /// Serialize the current store.
    ///
    /// The output will be a MessagePack'd gzip'd or zstd'd binary stream that should be
    /// written to disk.
    pub fn to_cache<W>(&self, mut writable: W) -> Result<(), Error>
    where
        W: Write + Sized,
    {
        let buf = {
            // This currently sits around 3.7MiB, so go up to 4 to fit comfortably
            let mut buf = Vec::with_capacity(4 * 1024 * 1024);
            let mut serializer = Serializer::new(&mut buf);
            self.serialize(&mut serializer)?;
            buf
        };

        info!("Pre-compressed output is {} bytes", buf.len());

        writable.write_all(CACHE_VERSION)?;

        #[cfg(not(feature = "gzip"))]
        let mut enc = zstd::Encoder::new(writable, 21)?;
        #[cfg(feature = "gzip")]
        let mut enc = flate2::write::GzEncoder::new(writable, flate2::Compression::default());

        copy(&mut buf.as_slice(), &mut enc)?;
        enc.finish()?;

        Ok(())
    }
}
