// GPT generated - TODO: implement yourself (I think there are several inefficiencies)

use std::{collections::HashMap, io::Cursor};

use bytes::Bytes;
use reqwest::{
    header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, HeaderMap, HeaderValue},
    Client,
};

use crate::{ADelete, AGet, APatch, APost, APut, ApiHttpClient};

use super::ConvenienceImpl;

#[derive(Debug, Clone)]
pub struct ReqwestApiHttpClient {
    client: Client,
    headers: HeaderMap,
    cookie_jar: HashMap<String, String>,
}

impl ReqwestApiHttpClient {
    pub fn new(agent: &str) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("*"));

        let client = Client::builder()
            .user_agent(agent)
            .build()
            .unwrap();


        Self {
            client,
            headers,
            cookie_jar: HashMap::new(),
        }
    }

    fn apply_headers(&self, mut req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        for (k, v) in self.headers.iter() {
            req = req.header(k, v);
        }
        req
    }

    fn body_bytes(payload: &[u8]) -> Bytes {
        Bytes::copy_from_slice(payload) // eek - anyway to avoid this intermediate allocation? TODO
    }

    async fn send_with_body(
        &self,
        req: reqwest::RequestBuilder,
    ) -> Result<Cursor<Vec<u8>>, reqwest::Error> {
        let bytes = req.send().await?.bytes().await?;
        Ok(Cursor::new(bytes.to_vec())) // this is fine - read the response fully
    }
}

impl ConvenienceImpl for ReqwestApiHttpClient {
    fn get_headers(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
    fn get_cookie_jar(&mut self) -> &mut HashMap<String, String> {
        &mut self.cookie_jar
    }
}

impl ApiHttpClient for ReqwestApiHttpClient {
    type R = Cursor<Vec<u8>>;
    type E = reqwest::Error;

    fn set_cookie(&mut self, name: &str, value: &str, root: &str) {
        ConvenienceImpl::set_cookie(self, name, value, root);
    }

    fn set_header(&mut self, key: &str, value: &str) {
        ConvenienceImpl::set_header(self, key, value);
    }
}

impl AGet for ReqwestApiHttpClient {
    fn async_get(
        &self,
        uri: &str,
        _payload: &[u8],
    ) -> impl std::future::Future<Output = Result<Self::R, Self::E>> + Send {
        let req = self.apply_headers(self.client.get(uri));

        async move { self.send_with_body(req).await }
    }
}

impl ADelete for ReqwestApiHttpClient {
    fn async_delete(
        &self,
        uri: &str,
        _payload: &[u8],
    ) -> impl std::future::Future<Output = Result<Self::R, Self::E>> + Send {
        let req = self.apply_headers(self.client.delete(uri));

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
            .apply_headers(self.client.put(uri))
            .body(Self::body_bytes(payload));

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
            .apply_headers(self.client.post(uri))
            .body(Self::body_bytes(payload));

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
            .apply_headers(self.client.patch(uri))
            .body(Self::body_bytes(payload));

        async move { self.send_with_body(req).await }
    }
}