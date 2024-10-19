use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn duckdb_test(_args: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_duckdb_test(ast)
}

fn impl_duckdb_test(ast: ItemFn) -> TokenStream {
    let ident = &ast.sig.ident;
    let gen = quote! {
        #[test]
        fn #ident() {
            let _mutex = MUTEX.lock().unwrap();
            #ast
            #ident();
        }
    };
    gen.into()
}
