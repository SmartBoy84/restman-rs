use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemStruct, Fields};

#[proc_macro_attribute]
pub fn bon_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_ident = input.ident;
    let vis = input.vis;
    let generics = input.generics;

    // inner name: prefix with underscore
    let inner_ident = format_ident!("_{}", struct_ident);

    // gather fields
    let mut field_defs = Vec::new();
    let mut trait_impls = Vec::new();

    if let Fields::Named(fields_named) = input.fields {
        for f in fields_named.named.iter() {
            let fname = f.ident.as_ref().expect("expected named field");
            let ftype = &f.ty;
            // build getter name
            let getter_name = format_ident!("get_{}_internal", fname);
            field_defs.push(quote! {
                #[builder(into, getter(name = #getter_name, vis = ""))]
                #fname: #ftype,
            });

            // compute camel-case for trait/state names
            let fname_str = fname.to_string();
            let parts: Vec<String> = fname_str.split('_').map(|s| s.to_string()).collect();
            let mut camel: String = parts.iter().map(|p| {
                if p == "id" { "Id".to_string() } else { let mut c = p.clone(); c.get_mut(0..1).map(|s| s.make_ascii_uppercase()); c }
            }).collect();

            // for trait name, make trailing id uppercase ID
            let mut camel_id = String::new();
            if parts.last().map(|s| s.as_str()) == Some("id") {
                let mut all_but_last = parts[..parts.len()-1].iter().map(|p| {
                    let mut c = p.clone(); c.get_mut(0..1).map(|s| s.make_ascii_uppercase()); c
                }).collect::<String>();
                camel_id = format!("{}ID", all_but_last);
            } else {
                camel_id = camel.clone();
            }

            let trait_ident = format_ident!("Has{}", camel_id);
            let state_ident = format_ident!("Set{}", camel);
            let method_ident = fname.clone();
            let getter_call = getter_name.clone();

            trait_impls.push(quote! {
                impl<'a> #trait_ident for #struct_ident<'a, echo_request_config::#state_ident> {
                    fn #method_ident(&self) -> &str {
                        self.#getter_call().as_ref()
                    }
                }
            });
        }
    } else {
        return syn::Error::new_spanned(&struct_ident, "bon_config only supports named fields").to_compile_error().into();
    }

    let output = quote! {
        // generated inner struct with bon builder
        #[derive(Debug, bon::Builder)]
        #[builder(builder_type(name = #struct_ident, vis = "pub"), finish_fn(vis = ""))]
        #vis struct #inner_ident #generics {
            #(#field_defs)*
        }

        // implement RequestConfig for the builder types
        impl<'a, S: echo_request_config::State> RequestConfig for #struct_ident<'a, S> {}

        impl<'a> #struct_ident<'a, echo_request_config::Empty> {
            pub fn new() -> Self {
                #inner_ident::builder()
            }
        }

        // generated trait impls
        #(#trait_impls)*
    };

    output.into()
}
