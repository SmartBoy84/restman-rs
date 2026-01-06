// this file contains the different parts of the API

macro_rules! request_part {
    ($name: ident, $word: literal, $default: ty) => {
        pub struct $name<T: RequestPart = $default>(PhantomData<T>);
        impl<T: RequestPart> RequestPart for $name<T> {}

        impl<C: RequestConfig, T: SerialiseRequestPart<C>> SerialiseRequestPart<C> for $name<T> {
            const WORD: &str = $word;
            type Next = T;
        }
    };

    ($name: ident, $word: literal, $default: ty, $config: path, $getter: ident) => {
        pub struct $name<T: RequestPart = $default>(PhantomData<T>);
        impl<T: RequestPart> RequestPart for $name<T> {}

        impl<C: RequestConfig + $config, T: SerialiseRequestPart<C>> SerialiseRequestPart<C>
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
