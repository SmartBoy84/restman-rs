use proc_macro::TokenStream;

use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, Fields, GenericParam, Generics, ItemStruct};

fn generic_type_args_with_state(generics: &Generics, state: TokenStream2) -> TokenStream2 {
    let args = generics.params.iter().map(|param| match param {
        GenericParam::Lifetime(lifetime) => {
            let lifetime = &lifetime.lifetime;
            quote! { #lifetime }
        }
        GenericParam::Type(ty) => {
            let ident = &ty.ident;
            quote! { #ident }
        }
        GenericParam::Const(const_param) => {
            let ident = &const_param.ident;
            quote! { #ident }
        }
    });

    quote! { <#(#args),*, #state> }
}

#[proc_macro_attribute]
pub fn bon_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_ident = input.ident;
    let vis = input.vis;
    let generics = input.generics;
    let config_module_ident = format_ident!("{}", struct_ident.to_string().to_snake_case());

    let mut builder_generics: Generics = generics.clone();
    builder_generics.params.push(parse_quote!(S));
    let (builder_impl_generics, _, builder_where_clause) = builder_generics.split_for_impl();
    let (orig_impl_generics, _, orig_where_clause) = generics.split_for_impl();
    let builder_type_args = generic_type_args_with_state(&generics, quote! { S });
    let empty_type_args = generic_type_args_with_state(&generics, quote! { #config_module_ident::Empty });

    let inner_ident = format_ident!("_{}", struct_ident);

    let mut field_defs = Vec::new();
    let mut trait_defs = Vec::new();
    let mut trait_impls = Vec::new();

    if let Fields::Named(fields_named) = input.fields {
        for f in fields_named.named.iter() {
            let fname = f.ident.as_ref().expect("expected named field");
            let ftype = &f.ty;
            let getter_name = format_ident!("get_{}_internal", fname);
            field_defs.push(quote! {
                #[builder(into, getter(name = #getter_name, vis = ""))]
                #fname: #ftype,
            });

            let fname_str = fname.to_string();
            let camel = fname_str.to_upper_camel_case();
            let camel_id = if fname_str == "id" {
                "ID".to_string()
            } else if fname_str.ends_with("_id") {
                let mut base = fname_str.trim_end_matches("_id").to_upper_camel_case();
                base.push_str("ID");
                base
            } else {
                camel.clone()
            };

            let trait_ident = format_ident!("Has{}", camel_id);
            let state_ident = format_ident!("Set{}", camel);
            let method_ident = fname.clone();
            let getter_call = getter_name.clone();
            let state_type_args = generic_type_args_with_state(&generics, quote! { #config_module_ident::#state_ident });

            trait_defs.push(quote! {
                pub trait #trait_ident {
                    fn #method_ident(&self) -> &str;
                }
            });

            trait_impls.push(quote! {
                impl #orig_impl_generics #trait_ident for #struct_ident #state_type_args #orig_where_clause {
                    fn #method_ident(&self) -> &str {
                        self.#getter_call().as_ref()
                    }
                }
            });
        }
    } else {
        return syn::Error::new_spanned(&struct_ident, "bon_config only supports named fields")
            .to_compile_error()
            .into();
    }

    let output = quote! {
        #[derive(Debug, bon::Builder)]
        #[builder(builder_type(name = #struct_ident, vis = "pub"), finish_fn(vis = ""))]
        #vis struct #inner_ident #generics {
            #(#field_defs)*
        }

        #(#trait_defs)*

        impl #builder_impl_generics ::restman_rs::request::RequestConfig for #struct_ident #builder_type_args
        where
            S: #config_module_ident::State,
            #builder_where_clause
        {}

        impl #orig_impl_generics #struct_ident #empty_type_args #orig_where_clause {
            pub fn new() -> Self {
                #inner_ident::builder()
            }
        }

        #(#trait_impls)*
    };

    output.into()
}
