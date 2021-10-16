use std::error::Error;

use lambda_http::{lambda, Body, IntoResponse, Request, Response};
use lambda_runtime::{error::HandlerError, Context};
use lazy_static::lazy_static;

use askalono::{Store, TextData};

// borrowing this from the CLI
static CACHE_DATA: &[u8] = include_bytes!("../embedded-cache.bin.zstd");

pub fn main() -> Result<(), Box<dyn Error>> {
    lambda!(handler);

    Ok(())
}

pub fn handler(e: Request, _c: Context) -> Result<impl IntoResponse, HandlerError> {
    lazy_static! {
        static ref STORE: Store = Store::from_cache(CACHE_DATA).unwrap();
    }

    if let Body::Text(body) = e.body() {
        let data = TextData::from(body.as_str());
        let result = STORE.analyze(&data);

        return Ok(Response::builder()
            .status(200)
            .body(format!("{}\n{}\n", result.name, result.score,))
            .unwrap());
    }
    Ok(Response::builder()
        .status(400)
        .body("uh oh".to_string())
        .unwrap())
}
