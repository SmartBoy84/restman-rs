use std::io::Read;

use crate::{
    ApiBackendError, ApiBackendResult, ApiHttpClient, MethodMarkerGetter,
    request::{ApiRequest, endpoints::Endpoint},
};

pub const AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36 Edg/136.0.0.0";

pub struct ApiClient<T: ApiHttpClient> {
    inner: T,
    token: String, // token's not thaaat long to bother with lifetimes infesting code
}

impl<C: ApiHttpClient> ApiClient<C> {
    pub fn new(backend: C, token: &str) -> Self {
        Self {
            inner: backend,
            token: token.into(),
        }
    }
}

impl<C: ApiHttpClient> ApiClient<C> {
    // FUCK, you can set bounds on associated types?! This simplifies so much shit
    /*
    enforce that the method is one that implements the getter trait for our client
    -> this way I can move the generic out from Endpoint and keep it independent from the client!
     */
    pub fn request<P>(&self, r: &ApiRequest<P>) -> ApiBackendResult<P::Res, C>
    where
        P: Endpoint,
        P::Method: MethodMarkerGetter<C>, // so awesome
    {
        // here is the whole point of ApiClient - to hide the HttpClient from library users
        Ok(serde_json::from_reader(
            P::Method::request(&self.inner, r.uri(), &self.token)
                .map_err(|e| ApiBackendError::HttpError(e))?,
        )?)
        // pretty cool - P::Method is MethodMarker - but we enforce that it is also MethodMarkerGetter
        // so no need to do <P::Method as MethodMarkerGetter<C>>::request - just do P::Method::request directly!
    }

    /// for debugging - read into a string
    pub fn raw_request<P>(&self, r: &ApiRequest<P>) -> ApiBackendResult<String, C>
    where
        P: Endpoint,
        P::Method: MethodMarkerGetter<C>,
    {
        let mut s = String::new();
        P::Method::request(&self.inner, r.uri(), &self.token)
            .map_err(|e| ApiBackendError::HttpError(e))?
            .read_to_string(&mut s)
            .unwrap();
        Ok(s)
    }
}
