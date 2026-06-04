#[cfg(feature = "reqwest")]
pub mod reqwest; // heavy - use if you need async (only implemented sync functions)
#[cfg(feature = "ureq")]
pub mod ureq; // lighter - use if you only need sync

pub const BEARER_TOKEN_HEADER_NAME: &str = "authorization"; // default header name