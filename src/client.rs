use super::{HttpMethod, RequestHandler};
use crate::request::{ApiRequest, endpoints::Endpoint};

use serde::de::DeserializeOwned;

use std::io::{self, Read};

use thiserror::Error;

pub const AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36 Edg/136.0.0.0";

#[derive(Error, Debug)]
pub enum ApiBackendError<E: std::error::Error> {
    #[error("http error: {0:?}")]
    HttpError(E),

    #[error("json error")]
    ParseError(#[from] serde_json::Error),

    #[error("io err")]
    ReadError(#[from] io::Error),
}

pub type ApiBackendResult<O, C> = Result<O, ApiBackendError<<C as ApiHttpClient>::Error>>;

pub struct ApiClient<T: ApiHttpClient> {
    inner: T,
    token: String, // token's not thaaat long to bother with lifetimes infesting code
    root: String,
}

impl<C: ApiHttpClient> ApiClient<C> {
    pub fn request<P>(&self, r: &ApiRequest<P>) -> ApiBackendResult<P::Res, C>
    where
        P: Endpoint,
    {
        P::Method::request(self, &r)
    }
}

impl<C: ApiHttpClient> RequestHandler for ApiClient<C> {
    type E = ApiBackendError<C::Error>;

    fn get<T, P>(&self, r: &ApiRequest<P>) -> Result<T, Self::E>
    where
        T: DeserializeOwned,
        P: Endpoint,
    {
        Ok(serde_json::from_reader(
            self.inner
                .get(r.uri(), &self.token)
                .map_err(|e| ApiBackendError::HttpError(e))?,
        )?)
    }

    fn patch<T, P>(&self, r: &ApiRequest<P>) -> Result<T, Self::E>
    where
        T: DeserializeOwned,
        P: Endpoint,
    {
        Ok(serde_json::from_reader(
            self.inner
                .patch(r.uri(), &self.token)
                .map_err(|e| ApiBackendError::HttpError(e))?,
        )?)
    }

    fn put<T, P>(&self, r: &ApiRequest<P>) -> Result<T, Self::E>
    where
        T: DeserializeOwned,
        P: Endpoint,
    {
        // needed to set READ status on notification
        Ok(serde_json::from_reader(
            self.inner
                .put(r.uri(), &self.token)
                .map_err(|e| ApiBackendError::HttpError(e))?,
        )?)
    }

    fn post<T, P>(&self, r: &ApiRequest<P>) -> Result<T, Self::E>
    where
        T: DeserializeOwned,
        P: Endpoint,
    {
        // needed to set READ status on notification
        Ok(serde_json::from_reader(
            self.inner
                .post(r.uri(), &self.token)
                .map_err(|e| ApiBackendError::HttpError(e))?,
        )?)
    }
}

impl<C: ApiHttpClient> ApiClient<C> {
    pub(super) fn new(backend: C, token: &str, root: &str) -> Self {
        Self {
            inner: backend,
            token: token.into(),
            root: root.to_owned(),
        }
    }

    pub(super) fn get_raw(&self, uri: &str) -> ApiBackendResult<String, C> {
        let mut s = String::new();
        self.inner
            .get(uri, &self.token)
            .map_err(|e| ApiBackendError::HttpError(e))?
            .read_to_string(&mut s)?;
        Ok(s)
    }
}

// trait can be public - inner methods will never be leaked because encapsulated in struct
pub trait ApiHttpClient {
    // impl Trait in trait method return types is permitted, however I want to enforce that *all* methods return the same type for consistency
    type Reader: std::io::Read; // no need for GAT because not doing massive downloads
    type Error: std::error::Error;

    fn set_cookie(&self, cookie: &str, uri: &'static str); // must be able to set a single persistent cookie once
    fn patch(&self, uri: &str, bearer_token: &str) -> Result<Self::Reader, Self::Error>; // this is all we need for patch, nothing more
    fn get(&self, uri: &str, bearer_token: &str) -> Result<Self::Reader, Self::Error>;
    fn put(&self, uri: &str, bearer_token: &str) -> Result<Self::Reader, Self::Error>;
    fn post(&self, uri: &str, bearer_token: &str) -> Result<Self::Reader, Self::Error>;
}
