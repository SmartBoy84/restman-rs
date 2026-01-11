pub mod client;
pub mod request;
pub mod ureq;

use std::{error::Error, io::Read};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiBackendError<E: Error> {
    #[error("http error")]
    HttpError(E),

    #[error("parse error")]
    ParseError(#[from] serde_json::Error),
}

pub type ApiBackendResult<T, C> = Result<T, ApiBackendError<<C as ApiHttpClient>::E>>;

pub trait Server {
    const ROOT: &str;
}

pub trait ApiHttpClient {
    type R: Read;
    type E: Error;

    fn set_cookie(&self, cookie: &str, uri: &'static str);
}

pub trait MethodMarker {}
pub trait MethodMarkerGetter<C: ApiHttpClient> {
    fn request(c: &C, uri: &str, bearer_token: &str) -> Result<C::R, C::E>;
}

#[macro_export]
macro_rules! method {
    ($name:ident, $trait:ident, $getter:ident) => {
        // create the method trait for http clients to implement
        trait $trait: ApiHttpClient {
            fn $getter(&self, uri: &str, bearer_token: &str) -> Result<Self::R, Self::E>;
        }

        // create a method marker struct to set in endpoints
        pub struct $name;
        impl MethodMarker for $name {}
        impl<C: $trait> MethodMarkerGetter<C> for $name {
            fn request(c: &C, uri: &str, bearer_token: &str) -> Result<C::R, C::E> {
                c.$getter(uri, bearer_token)
            }
        }
    };
}

method!(GET, Get, get);
method!(PATCH, Patch, patch);
method!(POST, Post, post);
method!(PUT, Put, put);
