use std::io::Read;

use crate::{
    ApiBackendError, ApiBackendResult, ApiHttpClient, MethodMarkerGetter, Server,
    request::{QueryPayload, ValidRequest, endpoints::Endpoint},
};

pub const AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36 Edg/136.0.0.0";

// split server and client config into separate traits to allow backend to separate multiple servers
pub trait ApiClientServer<C: Server> {}

pub trait ApiClientBackend<C: ApiHttpClient> {
    fn token(&self) -> &str;
    fn backend(&self) -> &C;
}

pub trait ApiClient<C: ApiHttpClient, S: Server>:
    ApiClientServer<S> + ApiClientBackend<C>
{
    fn request<P: Endpoint<Payload = (), Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
    ) -> ApiBackendResult<P::Res, C>
    where
        P::Method: MethodMarkerGetter<C>, // so awesome
    {
        self.inner_request(r, &[])
    }

    fn send_payload<P: Endpoint<Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
        p: &P::Payload,
    ) -> ApiBackendResult<P::Res, C>
    where
        P::Method: MethodMarkerGetter<C>, // so awesome
        P::Payload: QueryPayload,
    {
        self.inner_request(r, serde_json::to_string(p)?.as_bytes())
    }

    fn raw_request<P: Endpoint<Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
        p: &[u8],
    ) -> ApiBackendResult<String, C>
    where
        P::Method: MethodMarkerGetter<C>, // so awesome
    {
        let mut s = String::new();
        self.inner_raw_request(r, p)?
            .read_to_string(&mut s)
            .expect("bad string return");
        Ok(s)
    }
}
// access only permitted, if both specified!
impl<C: ApiHttpClient, S: Server, T: ApiClientServer<S> + ApiClientBackend<C>> ApiClient<C, S>
    for T
{
}

// seal this trait - external users shouldn't be able to see it
trait InnerGetter<C: ApiHttpClient, S: Server>: ApiClient<C, S> {
    // FUCK, you can set bounds on associated types?! This simplifies so much shit
    /*
    enforce that the method is one that implements the getter trait for our client
    -> this way I can move the generic out from Endpoint and keep it independent from the client!
     */
    fn inner_raw_request<P: Endpoint<Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
        p: &[u8],
    ) -> ApiBackendResult<C::R, C>
    where
        P::Method: MethodMarkerGetter<C>,
    {
        Ok(P::Method::request(self.backend(), r.uri(), self.token(), p)
            .map_err(|e| ApiBackendError::HttpError(e))?)
        // pretty cool - P::Method is MethodMarker - but we enforce that it is also MethodMarkerGetter
        // so no need to do <P::Method as MethodMarkerGetter<C>>::request - just do P::Method::request directly!
    }

    fn inner_request<P: Endpoint<Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
        p: &[u8],
    ) -> ApiBackendResult<P::Res, C>
    where
        P::Method: MethodMarkerGetter<C>,
    {
        Ok(serde_json::from_reader(self.inner_raw_request(r, p)?)?)
    }
}
impl<C: ApiHttpClient, S: Server, T: ApiClient<C, S> + ?Sized> InnerGetter<C, S> for T {}
