// backend is fully pluggable

use std::collections::HashMap;

use http::{
    HeaderMap,
    header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE},
};
use ureq::{self, Agent, BodyReader, RequestBuilder, config::Config};

use crate::{ApiHttpClient, Delete, Get, Patch, Post, Put};

use super::ConvenienceImpl;

#[derive(Debug)]
pub struct UreqApiHttpClient {
    // allow user to configure their own how they want to
    agent: Agent,
    headers: HeaderMap,
    cookie_jar: HashMap<String, String>,
}

impl UreqApiHttpClient {
    // apprently, bad practise to enforce constructors with traits...
    pub fn new(agent: &str) -> Self {
        // let a = ureq::Agent::new_with_config(Config::builder().user_agent(agent).build());

        let mut headers = HeaderMap::new();

        // safe deafult, but may need to adjust depending on application
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        // headers.insert(ACCEPT_CHARSET, "utf-8".parse().unwrap());
        headers.insert(ACCEPT, "*/*".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "*".parse().unwrap());

        Self {
            headers,
            agent: Agent::new_with_config(Config::builder().user_agent(agent).build()),
            cookie_jar: HashMap::new(),
        }
    }
}

impl ConvenienceImpl for UreqApiHttpClient {
    fn get_headers(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    fn get_cookie_jar(&mut self) -> &mut HashMap<String, String> {
        &mut self.cookie_jar
    }
}

impl ApiHttpClient for UreqApiHttpClient {
    type R = BodyReader<'static>; // not streaming, so 'static is fine
    type E = ureq::Error;

    // uri is const - 'static enforces that
    fn set_cookie(&mut self, name: &str, value: &str, root: &str) {
        ConvenienceImpl::set_cookie(self, name, value, root);
    }

    /// This should be a set-and-forget at initialisation - this is why I use panicing methods
    fn set_header(&mut self, key: &str, value: &str) {
        ConvenienceImpl::set_header(self, key, value);
    }
}

impl UreqApiHttpClient {
    fn append_headers<B>(&self, req: &mut RequestBuilder<B>) {
        *req.headers_mut().unwrap() = self.headers.clone(); // hm, doesn't seem very efficient...
    }
}

impl Get for UreqApiHttpClient {
    fn get(&self, uri: &str, _payload: &[u8]) -> Result<Self::R, Self::E> {
        let mut req = self.agent.get(uri);
        self.append_headers(&mut req);

        Ok(req.call()?.into_body().into_reader())
    }
}

impl Delete for UreqApiHttpClient {
    fn delete(&self, uri: &str, _payload: &[u8]) -> Result<Self::R, Self::E> {
        let mut req = self.agent.delete(uri);
        self.append_headers(&mut req);

        Ok(req.call()?.into_body().into_reader())
    }
}

impl Put for UreqApiHttpClient {
    fn put(&self, uri: &str, payload: &[u8]) -> Result<Self::R, Self::E> {
        let mut req = self.agent.put(uri);
        self.append_headers(&mut req);

        Ok(req.send(payload)?.into_body().into_reader())
    }
}

impl Post for UreqApiHttpClient {
    fn post(&self, uri: &str, payload: &[u8]) -> Result<Self::R, Self::E> {
        let mut req = self.agent.post(uri);
        self.append_headers(&mut req);

        Ok(req.send(payload)?.into_body().into_reader())
    }
}

impl Patch for UreqApiHttpClient {
    fn patch(&self, uri: &str, payload: &[u8]) -> Result<Self::R, Self::E> {
        let mut req = self.agent.patch(uri);
        self.append_headers(&mut req);

        Ok(req.send(payload)?.into_body().into_reader())
    }
}
