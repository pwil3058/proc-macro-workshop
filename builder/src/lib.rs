extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed_input: DeriveInput = parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    let mut tokens = vec![];
    if let Data::Struct(s) = parsed_input.data {
        if let Fields::Named(fields) = s.fields {
            println!("num: {}", fields.named.len());
            for field in fields.named.iter() {
                println!("{:?}", field.ident);
                let f_name = field.ident.as_ref().unwrap();
                let f_type = &field.ty;
                let token = quote! {
                    pub fn #f_name<'a>(&'a mut self, #f_name: #f_type) -> &'a mut Self {
                        self.#f_name = #f_name;
                        self
                    }
                };
                tokens.push(token);
            }
        } else {
            panic!("'Builder' can only be derived for structs with named fields.")
        }
    } else {
        panic!("'Builder' can only be derived for structs.")
    }

    let tokens = quote! {
        impl #struct_name {
            #(#tokens)*
        }
    };

    proc_macro::TokenStream::from(tokens)
}
