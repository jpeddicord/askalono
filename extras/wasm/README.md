# EXPERIMENTAL

The content in this directory should not be considered stable.

It has no defined API and its use is not currently supported.

As of writing, it requires nightly Rust.

It's pretty neat though! askalono can run in a web browser! This presents some
interesting use cases:

* In-browser license identification/diffing tools
* Firefox/Chrome/WebExtension utility to ID licenses
* Easy Node.js integration without the need for a compiler toolchain

The excitement is real.

## Build

### Library

Build it with `wasm-pack build --out-name askalono`.

### Demo

Build the library first, then `cd demo` and `npm run build`. The output will be in `dist`. Alternatively, run `npm start` for webpack-dev-server.