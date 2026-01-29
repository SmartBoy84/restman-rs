use serde::de::DeserializeOwned;

use crate::{
    MethodMarker, Server,
    request::{QueryParametersInner, QueryParametersOptional, QueryPayloadInner},
};

pub trait Endpoint {
    type Method: MethodMarker;
    type Res: DeserializeOwned;
    type Ser: Server;
    type Para: QueryParametersInner;
    type Payload: QueryPayloadInner;
}

impl QueryPayloadInner for () {}
impl QueryParametersInner for () {}
impl QueryParametersOptional for () {}

// note to self; #[macro_export] exposes them at the ROOT level of the crate, that's why I do $crate::endpoint_common

// With parameters, explicit method
// {server} {visibility} {name} {serialised name} {default next part} {response struct} {parameters} {method}
#[macro_export]
macro_rules! endpoint {
    ($server:ty, $vis:vis $name:ident, $word:literal, $default:ty, $res:ty, $params:ty, $payload:ty, $method:ty) => {
        $vis struct $name<T: $crate::request::RequestPart = $default, S: $crate::Server = $server>(std::marker::PhantomData<T>, std::marker::PhantomData<S>);

        impl<T: $crate::request::RequestPart, S: $crate::Server> $crate::request::endpoints::Endpoint for $name<T, S> {
            type Method = $method;
            type Res = $res;
            type Ser = S; // want to keep Server configurable - so it is an attribute on each struct rather than hard-coded
            type Para = $params;
            type Payload = $payload;
        }

        impl<T: $crate::request::RequestPart, S: $crate::Server> $crate::request::RequestPart for $name<T, S> {}

        impl<C: $crate::request::RequestConfig, T: $crate::request::SerialiseRequestPart<C>, S: $crate::Server> $crate::request::SerialiseRequestPart<C> for $name<T, S> {
            const WORD: &str = $word;
            type Next = T;
        }
    };
}
