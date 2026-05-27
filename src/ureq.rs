// backend is fully pluggable

use std::str::FromStr;

use http::{
    HeaderMap, HeaderName,
    header::{ACCEPT, ACCEPT_LANGUAGE},
};
use ureq::{self, BodyReader, Cookie, RequestBuilder, config::Config, http::Uri};

use crate::{ApiHttpClient, Get, Patch, Post, Put};

const BEARER_TOKEN_HEADER_NAME: &str = "authorization"; // default header name

#[derive(Debug)]
pub struct UreqApiHttpClient {
    a: ureq::Agent,

    // allow user to configure their own how they want to
    headers: HeaderMap,
}

impl UreqApiHttpClient {
    // apprently, bad practise to enforce constructors with traits...
    pub fn new(agent: &str) -> Self {
        let a = ureq::Agent::new_with_config(Config::builder().user_agent(agent).build());

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "*/*".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "*".parse().unwrap());

        Self { a, headers }
    }
}

impl ApiHttpClient for UreqApiHttpClient {
    type R = BodyReader<'static>; // not streaming, so 'static is fine
    type E = ureq::Error;

    // uri is const - 'static enforces that
    fn set_cookie(&self, cookie: &str, uri: &'static str) {
        let mut c = self.a.cookie_jar_lock();

        let uri = Uri::from_static(uri);
        let cookie = Cookie::parse(cookie.to_owned(), &uri).unwrap();

        c.insert(cookie, &uri).unwrap();
        c.release();
    }

    /// This should be a set-and-forget at initialisation - this is why I use panicing methods
    fn set_header(&mut self, key: &str, value: &str) {
        let cookie_val = value.parse().expect("bad value name");
        if let Some(val) = self.headers.get_mut(key) {
            *val = cookie_val;
        } else {
            self.headers.insert(
                HeaderName::from_str(key).expect("bad header name"),
                value.parse().expect("bad value name"),
            );
        }
    }
}

impl UreqApiHttpClient {
    fn append_headers<B>(&self, req: &mut RequestBuilder<B>) {
        *req.headers_mut().unwrap() = self.headers.clone(); // hm, doesn't seem very efficient...
    }
}

impl Get for UreqApiHttpClient {
    fn get(&self, uri: &str, _payload: &[u8]) -> Result<Self::R, Self::E> {
        let mut req = self.a.get(uri);
        self.append_headers(&mut req);

        Ok(self.a.get(uri).call()?.into_body().into_reader())
    }
}

impl Put for UreqApiHttpClient {
    fn put(&self, uri: &str, payload: &[u8]) -> Result<Self::R, Self::E> {
        let mut req = self.a.put(uri);
        self.append_headers(&mut req);

        Ok(req.send(payload)?.into_body().into_reader())
    }
}

impl Post for UreqApiHttpClient {
    fn post(&self, uri: &str, payload: &[u8]) -> Result<Self::R, Self::E> {
        let mut req = self.a.post(uri);
        self.append_headers(&mut req);

        Ok(req.send(payload)?.into_body().into_reader())
    }
}

impl Patch for UreqApiHttpClient {
    fn patch(&self, uri: &str, payload: &[u8]) -> Result<Self::R, Self::E> {
        let mut req = self.a.patch(uri);
        self.append_headers(&mut req);

        Ok(req.send(payload)?.into_body().into_reader())
    }
}
