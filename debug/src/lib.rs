extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed_input: syn::DeriveInput = syn::parse_macro_input!(input);
    let _input_name = parsed_input.ident;

    let tokens = quote! {
        impl std::fmt::Debug for Whatever {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Whatever{{}}")
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}
