use std::{collections::HashMap, io::Cursor, str::FromStr};

use reqwest::{
    Client,
    header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, COOKIE, HeaderMap, HeaderName, HeaderValue},
};

use crate::{AGet, APatch, APost, APut, ApiHttpClient};

#[derive(Debug)]
pub struct ReqwestApiHttpClient {
    client: Client,
    headers: HeaderMap,
    cookie_jar: HashMap<String, String>,
}

impl ReqwestApiHttpClient {
    pub fn new(agent: &str) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(ACCEPT, "*/*".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "*".parse().unwrap());

        Self {
            headers,
            client: Client::builder().user_agent(agent).build().unwrap(),
            cookie_jar: HashMap::new(),
        }
    }

    fn append_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.headers(self.headers.clone())
    }

    async fn send_with_body(
        &self,
        req: reqwest::RequestBuilder,
    ) -> Result<Cursor<Vec<u8>>, reqwest::Error> {
        let response = req.send().await?;
        let bytes = response.bytes().await?;
        Ok(Cursor::new(bytes.to_vec()))
    }
}

impl ApiHttpClient for ReqwestApiHttpClient {
    type R = Cursor<Vec<u8>>;
    type E = reqwest::Error;

    fn set_cookie(&mut self, name: &str, value: &str) {
        self.cookie_jar.insert(name.to_string(), value.to_string());

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

        self.headers
            .insert(COOKIE, cookie_header.try_into().expect("bad cookie"));
    }

    fn set_header(&mut self, key: &str, value: &str) {
        let header_value: HeaderValue = value.parse().expect("bad value name");
        if let Some(val) = self.headers.get_mut(key) {
            *val = header_value;
        } else {
            self.headers.insert(
                HeaderName::from_str(key).expect("bad header name"),
                value.parse().expect("bad value name"),
            );
        }
    }
}

impl AGet for ReqwestApiHttpClient {
    fn async_get(
        &self,
        uri: &str,
        _payload: &[u8],
    ) -> impl std::future::Future<Output = Result<Self::R, Self::E>> + Send {
        let req = self.append_headers(self.client.get(uri));

        async move { self.send_with_body(req).await }
    }
}

impl APut for ReqwestApiHttpClient {
    fn async_put(
        &self,
        uri: &str,
        payload: &[u8],
    ) -> impl std::future::Future<Output = Result<Self::R, Self::E>> + Send {
        let req = self
            .append_headers(self.client.put(uri))
            .body(payload.to_vec());

        async move { self.send_with_body(req).await }
    }
}

impl APost for ReqwestApiHttpClient {
    fn async_post(
        &self,
        uri: &str,
        payload: &[u8],
    ) -> impl std::future::Future<Output = Result<Self::R, Self::E>> + Send {
        let req = self
            .append_headers(self.client.post(uri))
            .body(payload.to_vec());

        async move { self.send_with_body(req).await }
    }
}

impl APatch for ReqwestApiHttpClient {
    fn async_patch(
        &self,
        uri: &str,
        payload: &[u8],
    ) -> impl std::future::Future<Output = Result<Self::R, Self::E>> + Send {
        let req = self
            .append_headers(self.client.patch(uri))
            .body(payload.to_vec());

        async move { self.send_with_body(req).await }
    }
}
