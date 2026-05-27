pub mod endpoints;
mod parts;

use std::marker::PhantomData;

use endpoints::Endpoint;
use serde::Serialize;

use crate::{ConstServer, DynamicServer};

pub trait QueryPayloadInner {}
pub trait QueryPayload: Serialize {}
impl<T: QueryPayload> QueryPayloadInner for T {} // blanket impl so that users don't have to implement the inner trait (needed for ())

pub trait QueryParametersInner {}

pub trait QueryParametersOptional: QueryParametersInner {}

pub trait QueryParameters: Serialize {
    fn add_str(&self, s: &mut String) {
        s.push('?');
        unsafe {
            // WOWZERS! Alright, serde_url_params can't ever fail because I vet my structs before using the unchecked unwrap
            // In addition, Serde will always yield utf8 so I can write directly to the string's underlying buffer
            serde_url_params::to_writer(s.as_mut_vec(), self).unwrap();
            // .unwrap_unchecked();
        }
    }
}

impl<T: QueryParameters> QueryParametersInner for T {}

pub trait RequestPart {
    /*
    marker trait to not have to add the config generic parameter in the struct
     */
}

pub trait SerialiseRequestPart<C: RequestConfig>: RequestPart {
    const WORD: &str;
    type Next: SerialiseRequestPart<C>;

    fn get_val(_config: &C) -> Option<&str> {
        None
    }

    // Wish I could make this const but config parameters in the URL makes that impossible
    fn add_str(s: &mut String, config: &C) {
        Self::Next::add_str(s, config);

        // should get optimised away since WORD is const
        if Self::WORD.len() > 0 {
            s.push('/');
            s.push_str(Self::WORD);
        }

        // should get optimised away?
        if let Some(v) = Self::get_val(config) {
            s.push('/');
            s.push_str(v);
        }
    }
}

pub trait RequestConfig {}

impl RequestConfig for () {}

// serialisation is expensive, and user may want to use same payload multiple times
pub struct ApiPayload<Q: QueryPayload> {
    data: String,
    _payload: PhantomData<Q>,
}

// QueryPayload, ApiPayload - runnin' outta names here man ;(
impl<Q: QueryPayload> ApiPayload<Q> {
    pub fn new(value: Q) -> serde_json::Result<Self> {
        let data = serde_json::to_string(&value)?;
        Ok(Self {
            data,
            _payload: PhantomData,
        })
    }
    pub fn payload(&self) -> &str {
        &self.data
    }
}


#[derive(Debug)]
// use the more general Endpoint here to avoid leaking implementation detail `Config`
pub struct ApiRequest<P: Endpoint> {
    uri: String,
    inner: PhantomData<P>,
}

#[derive(Debug)]
pub struct ApiRequestWithPara<P: Endpoint> {
    uri: String,
    uri_len: usize,
    inner: PhantomData<P>,
}

impl<E: Endpoint> ApiRequest<E> {
    fn new_inner<C: RequestConfig>(c: &C, root: &str) -> Self
    where
        E: SerialiseRequestPart<C>,
    {
        let mut uri = root.to_owned();
        E::add_str(&mut uri, c);
        Self {
            uri,
            inner: PhantomData,
        }
    }
}

// ensure correct request (with para or not) - page out of the MethodGetter trick in client.rs request() function
// it works... but the errors are bound to be difficult to interpret
// <E: Endpoint> says "where ever this trait is used, the Endpoint detail of the type is accessible"
pub trait ValidRequest<E: Endpoint> {
    fn uri(&self) -> &str;
}

// ApiRequest (no para specified) only valid if optional
impl<E> ValidRequest<E> for ApiRequest<E>
where
    E: Endpoint,
    E::Para: QueryParametersOptional,
{
    fn uri(&self) -> &str {
        &self.uri
    }
}

// you can only get ApiRequestWithPara by specifying parameters via ApiRequest, so all types valid
impl<E: Endpoint> ValidRequest<E> for ApiRequestWithPara<E> {
    fn uri(&self) -> &str {
        &self.uri
    }
}

impl<E: Endpoint> ApiRequest<E> {
    pub fn new<C: RequestConfig>(c: &C) -> Self
    where
        E::Ser: ConstServer,
        E: SerialiseRequestPart<C>,
    {
        Self::new_inner(c, E::Ser::ROOT)
    }

    pub fn new_with_server<C: RequestConfig>(c: &C, server: &E::Ser) -> Self
    where
        E::Ser: DynamicServer,
        E: SerialiseRequestPart<C>,
    {
        Self::new_inner(c, server.get_root())
    }

    // pretty cool - if optional then user doesn't *have* to call this method
    // but if optional trait not implemented, there's no way to call request() on the client without having called this
    pub fn add_para(self, p: &E::Para) -> ApiRequestWithPara<E>
    where
        E::Para: QueryParameters,
    {
        ApiRequestWithPara::new(self, p)
    }
}

// pretty cool
impl<E: Endpoint> ApiRequestWithPara<E>
where
    E::Para: QueryParameters,
{
    fn new(r: ApiRequest<E>, p: &E::Para) -> Self {
        let mut uri = r.uri;
        let uri_len = uri.len();
        p.add_str(&mut uri);
        Self {
            uri,
            uri_len,
            inner: PhantomData,
        }
    }

    pub fn change_para(&mut self, p: E::Para) {
        self.uri.truncate(self.uri_len);
        p.add_str(&mut self.uri);
    }
}