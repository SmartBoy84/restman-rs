// GPT generated - TODO: implement yourself (I think there are several inefficiencies)

use std::{io::Cursor, str::FromStr, sync::Arc};

use bytes::Bytes;
use reqwest::{
    cookie::Jar,
    header::{ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
    Client,
    Url,
};

use crate::{AGet, APatch, APost, APut, ApiHttpClient};

#[derive(Debug, Clone)]
pub struct ReqwestApiHttpClient {
    client: Client,
    headers: HeaderMap,
    cookies: Arc<Jar>,
    base_url: Url,
}

impl ReqwestApiHttpClient {
    pub fn new(agent: &str, base_url: &str) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("*"));

        let cookies = Arc::new(Jar::default());

        let client = Client::builder()
            .user_agent(agent)
            .cookie_provider(cookies.clone())
            .build()
            .unwrap();

        let base_url = Url::parse(base_url).expect("invalid base url");

        Self {
            client,
            headers,
            cookies,
            base_url,
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

impl ApiHttpClient for ReqwestApiHttpClient {
    type R = Cursor<Vec<u8>>;
    type E = reqwest::Error;

    fn set_cookie(&mut self, name: &str, value: &str) {
        let cookie = format!("{name}={value}");

        self.cookies.add_cookie_str(&cookie, &self.base_url);
    }

    fn set_header(&mut self, key: &str, value: &str) {
        let Ok(name) = HeaderName::from_str(key) else {
            return;
        };

        let Ok(val) = HeaderValue::from_str(value) else {
            return;
        };

        self.headers.insert(name, val);
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