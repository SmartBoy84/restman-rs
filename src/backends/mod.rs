use std::{collections::HashMap, str::FromStr};

use http::{HeaderMap, HeaderName, header::COOKIE};

#[cfg(feature = "reqwest")]
pub mod reqwest; // heavy - use if you need async (only implemented sync functions)
#[cfg(feature = "ureq")]
pub mod ureq; // lighter - use if you only need sync

pub const BEARER_TOKEN_HEADER_NAME: &str = "authorization"; // default header name

pub(crate) trait ConvenienceImpl {
    fn get_headers(&mut self) -> &mut HeaderMap;
    fn get_cookie_jar(&mut self) -> &mut HashMap<String, String>;

    // uri is const - 'static enforces that
    fn set_cookie(&mut self, name: &str, value: &str, _root: &str) {
        let cookie_jar = self.get_cookie_jar();

        cookie_jar.insert(name.to_string(), value.to_string());

        // regenerate cookie header
        let mut cookie_header = String::new();
        cookie_header.reserve(cookie_jar.iter().map(|(k, v)| k.len() + v.len() + 2).sum());

        for (i, (k, v)) in cookie_jar.iter().enumerate() {
            if i > 0 {
                cookie_header.push(';');
            }
            cookie_header.push_str(k);
            cookie_header.push('=');
            cookie_header.push_str(v);
        }

        // add back cookie header
        self.get_headers()
            .insert(COOKIE, cookie_header.try_into().expect("bad cookie"));
    }

    /// This should be a set-and-forget at initialisation - this is why I use panicing methods
    fn set_header(&mut self, key: &str, value: &str) {
        let headers = self.get_headers();
        let cookie_val = value.parse().expect("bad value name");
        if let Some(val) = headers.get_mut(key) {
            *val = cookie_val;
        } else {
            headers.insert(
                HeaderName::from_str(key).expect("bad header name"),
                value.parse().expect("bad value name"),
            );
        }
    }
}
