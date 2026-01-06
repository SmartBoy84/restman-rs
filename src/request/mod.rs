// // /v1/companies/{company id}/users/{user id}/notifications

mod parts;
pub mod endpoints;

use std::marker::PhantomData;

use endpoints::{Endpoint, EndpointWithParameters};
use serde::Serialize;

use crate::request::endpoints::EndpointWithNoPara;

pub trait QueryParameters: Serialize {
    fn add_str(&self, s: &mut String) {
        unsafe {
            s.push('?');
            // WOWZERS! Alright, serde_url_params can't ever fail because I vet my structs before using the unchecked unwrap
            // In addition, Serde will always yield utf8 so I can write directly to the string's underlying buffer
            serde_url_params::to_writer(s.as_mut_vec(), self).unwrap();
            // .unwrap_unchecked();
        }
    }
}

pub trait SerialiseRequestPart<C: RequestConfig>: RequestPart {
    const WORD: &str;
    type Next: SerialiseRequestPart<C>;

    fn get_val(_config: &C) -> Option<&str> {
        None
    }

    // Wish I could make this const but config parameters in the URL makes that impossible
    fn add_str(s: &mut String, config: &C) {
        <<Self as SerialiseRequestPart<C>>::Next>::add_str(s, config);

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

pub trait RequestPart {
    /*
    Marker trait prevents creation of RequestParts with invalid inner types
    */
}

pub trait RequestConfig {}

impl RequestConfig for () {}

#[derive(Default)]
// use the more general Endpoint here to avoid leaking implementation detail `Config`
pub struct ApiRequest<P: Endpoint> {
    uri: String,
    uri_len: usize,
    inner: PhantomData<P>,
}

impl<E: Endpoint> ApiRequest<E> {
    fn new_inner<C: RequestConfig>(c: &C, root: &str) -> Self
    where
        E: SerialiseRequestPart<C>, // guaranteed, since I do SerialiseEndpoint: Endpoint
    {
        let mut uri = root.to_string();
        E::add_str(&mut uri, c);
        let uri_len = uri.len();
        Self {
            uri,
            uri_len,
            inner: PhantomData,
        }
    }
}

impl<E: Endpoint + EndpointWithNoPara> ApiRequest<E> {
    pub fn new<C: RequestConfig>(c: &C, root: &str) -> Self
    where
        E: SerialiseRequestPart<C>, // guaranteed, since I do SerialiseEndpoint: Endpoint
    {
        Self::new_inner(c, root)
    }
}

impl<E: Endpoint + EndpointWithParameters> ApiRequest<E> {
    pub fn new_with_para<C>(c: &C, p: E::P, root: &str) -> Self
    where
        C: RequestConfig,
        E: SerialiseRequestPart<C>,
    {
        let mut s = Self::new_inner(c, root);
        p.add_str(s.uri_mut());
        s
    }
}

impl<T: Endpoint> ApiRequest<T> {
    pub fn change_para(&mut self, p: T::P)
    where
        T: EndpointWithParameters,
    {
        self.uri.truncate(self.uri_len);
        p.add_str(&mut self.uri);
    }

    fn uri_mut(&mut self) -> &mut String {
        &mut self.uri
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }
}
