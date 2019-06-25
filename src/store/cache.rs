// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{io::copy, io::prelude::*};

use failure::Error;
use log::info;
use rmp_serde::Serializer;
use serde::Serialize;

use crate::store::base::Store;

#[cfg(target_arch = "wasm32")]
const CACHE_VERSION: &[u8] = b"askalono-03";
#[cfg(not(target_arch = "wasm32"))]
const CACHE_VERSION: &[u8] = b"askalono-04";

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

        #[cfg(target_arch = "wasm32")]
        {
            let dec = flate2::read::GzDecoder::new(readable);
            {
                let extra = dec
                    .header()
                    .ok_or_else(|| failure::format_err!("cache gzip header invalid"))?
                    .extra()
                    .ok_or_else(|| failure::format_err!("cache gzip extra header missing"))?;
                if extra != CACHE_VERSION {
                    failure::bail!("cache version mismatch");
                }
            }

            Ok(from_read(dec)?)
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // `readable` only has to be mut in the zstd path, so if we put it in the function
            // signature the gzip path complains it doesn't need to be mut, so just move it
            let mut readable = readable;

            let mut header = [0u8; 11];
            readable.read_exact(&mut header)?;

            if header != CACHE_VERSION {
                failure::bail!("cache version mismatch");
            }

            let dec = zstd::Decoder::new(readable)?;
            let store = from_read(dec)?;
            Ok(store)
        }
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

        #[cfg(target_arch = "wasm32")]
        {
            let mut gz = flate2::GzBuilder::new()
                .extra(CACHE_VERSION)
                .write(&mut writable, flate2::Compression::best());
            copy(&mut buf.as_slice(), &mut gz)?;

            Ok(())
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            writable.write_all(CACHE_VERSION)?;
            let mut zenc = zstd::Encoder::new(writable, 21)?;

            copy(&mut buf.as_slice(), &mut zenc)?;
            zenc.finish()?;

            Ok(())
        }
    }
}
