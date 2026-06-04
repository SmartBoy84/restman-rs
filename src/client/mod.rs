use crate::{ApiHttpClient, Server};

pub mod async_client;
pub mod sync_client; // literally copy and paste of above with async primitives

pub const AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36 Edg/136.0.0.0";

// split server and client config into separate traits to allow backend to separate multiple servers
pub trait ApiClientServer<C: Server> {}

pub trait ApiClientBackend<C: ApiHttpClient> {
    fn backend(&self) -> &C;
}
