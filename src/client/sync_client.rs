use std::io::Read;

use crate::{
    ApiBackendError, ApiBackendResult, ApiHttpClient, MethodMarkerGetter, Server,
    client::{ApiClientBackend, ApiClientServer},
    request::{ApiPayload, QueryPayload, ValidRequest, endpoints::Endpoint},
};

pub trait ApiClient<C: ApiHttpClient, S: Server>: ApiClientServer<S> + ApiClientBackend<C> {
    fn request<P: Endpoint<Payload = (), Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
    ) -> ApiBackendResult<P::Res, C>
    where
        P::Method: MethodMarkerGetter<C>,
    {
        self.inner_request(r, &[])
    }

    fn send_payload<P: Endpoint<Payload: QueryPayload, Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
        p: &ApiPayload<P::Payload>,
    ) -> ApiBackendResult<P::Res, C>
    where
        P::Method: MethodMarkerGetter<C>,
        P::Payload: QueryPayload,
    {
        self.inner_request(r, p.payload().as_bytes())
    }

    fn raw_request<P: Endpoint<Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
        p: &[u8],
    ) -> ApiBackendResult<String, C>
    where
        P::Method: MethodMarkerGetter<C>,
    {
        let mut s = String::new();
        self.inner_raw_request(r, p)?
            .read_to_string(&mut s)
            .expect("bad string return");
        Ok(s)
    }
}

// access only permitted, if both specified!
// splitting them allows me to permit user to support multiple server backends
impl<C: ApiHttpClient, S: Server, T: ApiClientServer<S> + ApiClientBackend<C>> ApiClient<C, S>
    for T
{
}

// seal this trait - external users shouldn't be able to see it
trait InnerGetter<C: ApiHttpClient, S: Server>: ApiClient<C, S> {
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
        Ok(P::Method::request(self.backend(), r.uri(), p)
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
