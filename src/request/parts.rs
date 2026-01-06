// this file contains the different parts of the API

use crate::request::{RequestConfig, RequestPart, SerialiseRequestPart};

#[macro_export]
macro_rules! request_part {
    ($name: ident, $word: literal, $default: ty) => {        
        pub struct $name<T: $crate::request::RequestPart = $default>(std::marker::PhantomData<T>);
        impl<T: $crate::request::RequestPart> $crate::request::RequestPart for $name<T> {}

        impl<C: $crate::request::RequestConfig, T: $crate::request::SerialiseRequestPart<C>> $crate::request::SerialiseRequestPart<C> for $name<T> {
            const WORD: &str = $word;
            type Next = T;
        }
    };

    ($name: ident, $word: literal, $default: ty, $config: path, $getter: ident) => {
        pub struct $name<T: $crate::request::RequestPart = $default>(std::marker::PhantomData<T>);
        impl<T: $crate::request::RequestPart> $crate::request::RequestPart for $name<T> {}

        impl<C: $crate::request::RequestConfig + $config, T: $crate::request::SerialiseRequestPart<C>> $crate::request::SerialiseRequestPart<C>
            for $name<T>
        {
            const WORD: &str = $word;
            type Next = T;

            fn get_val(config: &C) -> Option<&str> {
                Some(config.$getter())
            }
        }
    };
}

impl RequestPart for () {}

impl<C: RequestConfig> SerialiseRequestPart<C> for () {
    const WORD: &str = "";
    type Next = ();

    fn add_str(_s: &mut String, _config: &C) {
        ()
    }
}

