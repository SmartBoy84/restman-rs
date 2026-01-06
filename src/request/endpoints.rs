use serde::de::DeserializeOwned;

use crate::{HttpMethod, Server, request::QueryParameters};

pub trait Endpoint {
    type Method: HttpMethod;
    type Res: DeserializeOwned;
    type Ser: Server;
}

pub trait EndpointWithNoPara: Endpoint {}
pub trait EndpointWithParameters: Endpoint {
    type P: QueryParameters;
}

#[macro_export]
macro_rules! endpoint_common {
    ($vis:vis $name:ident, $word:literal, $default:ty, $res:ty, $method:ty, $server:ty) => {
        $vis struct $name<T: $crate::request::RequestPart = $default, S: $crate::Server = $server>(std::marker::PhantomData<T>, std::marker::PhantomData<S>);

        impl<T: $crate::request::RequestPart, S: $crate::Server> $crate::request::endpoints::Endpoint for $name<T, S> {
            type Method = $method;
            type Res = $res;
            type Ser = S; // want to keep Server configurable - so it is an attribute on each struct rather than hard-coded
        }

        impl<T: $crate::request::RequestPart, S: $crate::Server> $crate::request::RequestPart for $name<T, S> {}

        impl<C: $crate::request::RequestConfig, T: $crate::request::SerialiseRequestPart<C>, S: $crate::Server> $crate::request::SerialiseRequestPart<C> for $name<T, S> {
            const WORD: &str = $word;
            type Next = T;
        }
    };
}

// note to self; #[macro_export] exposes them at the ROOT level of the crate, that's why I do $crate::endpoint_common

#[macro_export]
macro_rules! endpoint {
    // No parameters, default GET
    ($vis:vis $name:ident, $word:literal, $default:ty, $res:ty, $server:ty) => {
        $crate::endpoint_common!($vis $name, $word, $default, $res, $crate::GET, $server);
        impl<T: $crate::request::RequestPart, S: $crate::Server> $crate::request::endpoints::EndpointWithNoPara for $name<T, S> {}
    };
    // No parameters, explicit method
    ($vis:vis $name:ident, $word:literal, $default:ty, $res:ty, method = $method:ty, $server:ty) => {
        $crate::endpoint_common!($vis $name, $word, $default, $res, $method, $server);
        impl<T: $crate::request::RequestPart, S: $crate::Server> $crate::request::endpoints::EndpointWithNoPara for $name<T, S> {}
    };
    // With parameters, default GET
    ($vis:vis $name:ident, $word:literal, $default:ty, $res:ty, $params:ty, $server:ty) => {
        $crate::endpoint_common!($vis $name, $word, $default, $res, $crate::GET, $server);
        impl<T: $crate::request::RequestPart, S: $crate::Server> $crate::request::endpoints::EndpointWithParameters for $name<T, S> {
            type P = $params;
        }
    };
    // With parameters, explicit method
    ($vis:vis $name:ident, $word:literal, $default:ty, $res:ty, $params:ty, method = $method:ty, $server:ty) => {
        $crate::endpoint_common!($vis $name, $word, $default, $res, $method, $server);
        impl<T: $crate::request::RequestPart, S: $crate::Server> $crate::request::endpoints::EndpointWithParameters for $name<T, S> {
            type P = $params;
        }
    };
}
