use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Href)]
pub fn href_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = quote! {
        impl stac_types::Href for #name {
            fn href(&self) -> Option<&str> {
                self.href.as_deref()
            }
            fn set_href(&mut self, href: impl ToString) {
                self.href = Some(href.to_string());
            }
            fn clear_href(&mut self) {
                self.href = None;
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
