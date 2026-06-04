// NOTE; no comments here, refer to "sync_client.rs"

use std::{future::Future, io::Read};

use crate::{
    ApiBackendError, ApiBackendResult, ApiHttpClient, AsyncMethodMarkerGetter, Server,
    client::{ApiClientBackend, ApiClientServer},
    request::{ApiPayload, QueryPayload, ValidRequest, endpoints::Endpoint},
};

pub trait ApiClient<C: ApiHttpClient, S: Server>: ApiClientServer<S> + ApiClientBackend<C> {
    fn async_request<P: Endpoint<Payload = (), Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
    ) -> impl Future<Output = ApiBackendResult<P::Res, C>> + Send
    where
        P::Method: AsyncMethodMarkerGetter<C>,
        Self: Sync,
        R: Sync,
    {
        self.async_inner_request(r, &[])
    }

    fn async_send_payload<P: Endpoint<Payload: QueryPayload, Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
        p: &ApiPayload<P::Payload>,
    ) -> impl Future<Output = ApiBackendResult<P::Res, C>> + Send
    where
        P::Method: AsyncMethodMarkerGetter<C>,
        P::Payload: QueryPayload,
        Self: Sync,
        R: Sync,
    {
        self.async_inner_request(r, p.payload().as_bytes())
    }

    fn async_raw_request<P: Endpoint<Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
        p: &[u8],
    ) -> impl Future<Output = ApiBackendResult<String, C>> + Send
    where
        P::Method: AsyncMethodMarkerGetter<C>,
        Self: Sync,
        R: Sync,
    {
        let mut s = String::new();
        async {
            self.async_inner_raw_request(r, p)
                .await?
                .read_to_string(&mut s)
                .expect("bad string return");
            Ok(s)
        }
    }
}

impl<C: ApiHttpClient, S: Server, T: ApiClientServer<S> + ApiClientBackend<C>> ApiClient<C, S>
    for T
{
}

trait InnerGetter<C: ApiHttpClient, S: Server>: ApiClient<C, S> {
    fn async_inner_raw_request<P: Endpoint<Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
        p: &[u8],
    ) -> impl Future<Output = ApiBackendResult<C::R, C>> + Send
    where
        P::Method: AsyncMethodMarkerGetter<C>,
        R: Sync,
        Self: Sync,
    {
        async move {
            Ok(P::Method::async_request(self.backend(), r.uri(), p)
                .await
                .map_err(|e| ApiBackendError::HttpError(e))?)
        }
    }

    fn async_inner_request<P: Endpoint<Ser = S>, R: ValidRequest<P>>(
        &self,
        r: &R,
        p: &[u8],
    ) -> impl Future<Output = ApiBackendResult<P::Res, C>> + Send
    where
        P::Method: AsyncMethodMarkerGetter<C>,
        Self: Sync,
        R: Sync,
    {
        async {
            Ok(serde_json::from_reader::<_, P::Res>(
                self.async_inner_raw_request(r, p).await?,
            )?)
        }
    }
}

impl<C: ApiHttpClient, S: Server, T: ApiClient<C, S> + ?Sized> InnerGetter<C, S> for T {}
