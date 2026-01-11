use crate::{
    ApiBackendError, ApiBackendResult, ApiHttpClient, MethodMarkerGetter,
    request::{ApiRequest, endpoints::Endpoint},
};

pub const AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36 Edg/136.0.0.0";

pub struct ApiClient<T: ApiHttpClient> {
    inner: T,
    token: String, // token's not thaaat long to bother with lifetimes infesting code
}

// FUCK, you can set bounds on associated types?! This simplifies so much shit
/*
enforce that the method is one that implements the getter trait for our client
-> this way I can move the generic out from Endpoint and keep it independent from the client!
 */
impl<C: ApiHttpClient> ApiClient<C> {
    pub fn request<P>(&self, r: &ApiRequest<P>) -> ApiBackendResult<P::Res, C>
    where
        P: Endpoint,
        P::Method: MethodMarkerGetter<C>, // so awesome
    {
        // here is the whole point of ApiClient - to hide the HttpClient from library users
        Ok(serde_json::from_reader(
            <P::Method as MethodMarkerGetter<C>>::request(&self.inner, r.uri(), &self.token)
                .map_err(|e| ApiBackendError::HttpError(e))?,
        )?)
    }
}

impl<C: ApiHttpClient> ApiClient<C> {
    pub fn new(backend: C, token: &str) -> Self {
        Self {
            inner: backend,
            token: token.into(),
        }
    }

    // pub fn get_raw(&self, uri: &str) -> ApiBackendResult<String, C> {
    //     let mut s = String::new();
    //     self.inner
    //         .get(uri, &self.token)
    //         .map_err(|e| ApiBackendError::HttpError(e))?
    //         .read_to_string(&mut s)?;
    //     Ok(s)
    // }
}
