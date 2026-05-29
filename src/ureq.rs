// backend is fully pluggable

use std::{collections::HashMap, str::FromStr};

use http::{
    HeaderMap, HeaderName,
    header::{ACCEPT, ACCEPT_LANGUAGE, COOKIE},
};
use ureq::{self, Agent, BodyReader, RequestBuilder, config::Config};

use crate::{ApiHttpClient, Get, Patch, Post, Put};

pub const BEARER_TOKEN_HEADER_NAME: &str = "authorization"; // default header name

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
        headers.insert(ACCEPT, "*/*".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "*".parse().unwrap());

        Self {
            headers,
            agent: Agent::new_with_config(Config::builder().user_agent(agent).build()),
            cookie_jar: HashMap::new(),
        }
    }
}

impl ApiHttpClient for UreqApiHttpClient {
    type R = BodyReader<'static>; // not streaming, so 'static is fine
    type E = ureq::Error;

    // uri is const - 'static enforces that
    fn set_cookie(&mut self, name: &str, value: &str) {
        self.cookie_jar.insert(name.to_string(), value.to_string());

        // regenerate cookie header
        let mut cookie_header = String::new();
        cookie_header.reserve(
            self.cookie_jar
                .iter()
                .map(|(k, v)| k.len() + v.len() + 2)
                .sum(),
        );

        for (i, (k, v)) in self.cookie_jar.iter().enumerate() {
            if i > 0 {
                cookie_header.push(';');
            }
            cookie_header.push_str(k);
            cookie_header.push('=');
            cookie_header.push_str(v);
        }

        // add back cookie header
        self.headers
            .insert(COOKIE, cookie_header.try_into().expect("bad cookie"));
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
        let mut req = self.agent.get(uri);
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
        println!("{}", self.headers.get(COOKIE).unwrap().to_str().unwrap());

        let mut req = self.agent.patch(uri);
        self.append_headers(&mut req);

        Ok(req.send(payload)?.into_body().into_reader())
    }
}
