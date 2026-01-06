use std::marker::PhantomData;

use serde::de::DeserializeOwned;

use crate::{HttpMethod, request::QueryParameters};

pub trait Endpoint {
    type Method: HttpMethod;
    type Res: DeserializeOwned;
}

pub trait EndpointWithNoPara: Endpoint {}
pub trait EndpointWithParameters: Endpoint {
    type P: QueryParameters;
}

#[macro_export]
macro_rules! endpoint_common {
    ($vis:vis $name:ident, $word:literal, $default:ty, $res:ty, $method:ty) => {
        $vis struct $name<T: RequestPart = $default>(PhantomData<T>);

        impl<T: RequestPart> Endpoint for $name<T> {
            type Method = $method;
            type Res = $res;
        }

        impl<T: RequestPart> RequestPart for $name<T> {}

        impl<C: RequestConfig, T: SerialiseRequestPart<C>> SerialiseRequestPart<C> for $name<T> {
            const WORD: &str = $word;
            type Next = T;
        }
    };
}

#[macro_export]
macro_rules! endpoint {
    // No parameters, default GET
    ($vis:vis $name:ident, $word:literal, $default:ty, $res:ty) => {
        endpoint_common!($vis $name, $word, $default, $res, GET);
        impl<T: RequestPart> EndpointWithNoPara for $name<T> {}
    };
    // No parameters, explicit method
    ($vis:vis $name:ident, $word:literal, $default:ty, $res:ty, method = $method:ty) => {
        endpoint_common!($vis $name, $word, $default, $res, $method);
        impl<T: RequestPart> EndpointWithNoPara for $name<T> {}
    };
    // With parameters, default GET
    ($vis:vis $name:ident, $word:literal, $default:ty, $res:ty, $params:ty) => {
        endpoint_common!($vis $name, $word, $default, $res, GET);
        impl<T: RequestPart> EndpointWithParameters for $name<T> {
            type P = $params;
        }
    };
    // With parameters, explicit method
    ($vis:vis $name:ident, $word:literal, $default:ty, $res:ty, $params:ty, method = $method:ty) => {
        endpoint_common!($vis $name, $word, $default, $res, $method);
        impl<T: RequestPart> EndpointWithParameters for $name<T> {
            type P = $params;
        }
    };
}
