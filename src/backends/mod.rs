#[cfg(feature = "reqwest")]
pub mod reqwest;
#[cfg(feature = "ureq")]
pub mod ureq;

#[cfg(feature = "reqwest")]
pub use reqwest::ReqwestApiHttpClient;
#[cfg(feature = "ureq")]
pub use ureq::UreqApiHttpClient;

pub const BEARER_TOKEN_HEADER_NAME: &str = "authorization"; // default header name