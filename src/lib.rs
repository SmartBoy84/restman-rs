pub mod client;
pub mod request;
pub mod ureq;

use request::{ApiRequest, endpoints::Endpoint};
use ureq::UreqApiHttpClient;

use serde::de::DeserializeOwned;

type SelectedHttpClient = UreqApiHttpClient;

pub trait HttpMethod {
    fn request<H: RequestHandler, T: DeserializeOwned, P: Endpoint>(
        handler: &H,
        r: &ApiRequest<P>,
    ) -> Result<T, H::E>;
}

// pluggable backend
pub struct PATCH;
pub struct GET;
pub struct PUT;
pub struct POST;

impl HttpMethod for PATCH {
    fn request<H: RequestHandler, T: DeserializeOwned, P: Endpoint>(
        handler: &H,
        r: &ApiRequest<P>,
    ) -> Result<T, H::E> {
        handler.patch(r)
    }
}

impl HttpMethod for GET {
    fn request<H: RequestHandler, T: DeserializeOwned, P: Endpoint>(
        handler: &H,
        r: &ApiRequest<P>,
    ) -> Result<T, H::E> {
        handler.get(r)
    }
}
impl HttpMethod for PUT {
    fn request<H: RequestHandler, T: DeserializeOwned, P: Endpoint>(
        handler: &H,
        r: &ApiRequest<P>,
    ) -> Result<T, H::E> {
        handler.put(r)
    }
}
impl HttpMethod for POST {
    fn request<H: RequestHandler, T: DeserializeOwned, P: Endpoint>(
        handler: &H,
        r: &ApiRequest<P>,
    ) -> Result<T, H::E> {
        handler.post(r)
    }
}

pub trait RequestHandler {
    type E: std::error::Error;
    fn get<T, P>(&self, r: &ApiRequest<P>) -> Result<T, Self::E>
    where
        T: DeserializeOwned,
        P: Endpoint;

    fn patch<T, P>(&self, r: &ApiRequest<P>) -> Result<T, Self::E>
    where
        T: DeserializeOwned,
        P: Endpoint;

    fn put<T, P>(&self, r: &ApiRequest<P>) -> Result<T, Self::E>
    where
        T: DeserializeOwned,
        P: Endpoint;
    fn post<T, P>(&self, r: &ApiRequest<P>) -> Result<T, Self::E>
    where
        T: DeserializeOwned,
        P: Endpoint;
}
