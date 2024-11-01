use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SelfHref)]
pub fn self_href_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = quote! {
        impl stac_types::SelfHref for #name {
            fn self_href(&self) -> Option<&stac_types::Href> {
                self.self_href.as_ref()
            }
            fn self_href_mut(&mut self) -> &mut Option<stac_types::Href> {
                &mut self.self_href
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(Links)]
pub fn links_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = quote! {
        impl stac_types::Links for #name {
            fn links(&self) -> &[stac_types::Link] {
                &self.links
            }
            fn links_mut(&mut self) -> &mut Vec<stac_types::Link> {
                &mut self.links
            }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(Migrate)]
pub fn migrate_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = quote! {
        impl stac_types::Migrate for #name {}
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(Fields)]
pub fn fields_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = quote! {
        impl stac_types::Fields for #name {
            fn fields(&self) -> &serde_json::Map<String, serde_json::Value> {
                &self.additional_fields
            }
            fn fields_mut(&mut self) -> &mut serde_json::Map<String, Value> {
                &mut self.additional_fields
            }
        }
    };
    TokenStream::from(expanded)
}
