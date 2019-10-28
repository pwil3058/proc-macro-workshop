extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed_input: DeriveInput = parse_macro_input!(input);
    let struct_name = parsed_input.ident;
    let builder_name = Ident::new(&format!("{}{}", struct_name, "Builder"), Span::call_site());
    let mut declr_tokens = vec![];
    let mut init_tokens = vec![];
    let mut fn_tokens = vec![];
    let mut build_tokens = vec![];
    let mut list_tokens = vec![];
    if let Data::Struct(s) = parsed_input.data {
        if let Fields::Named(fields) = s.fields {
            println!("num: {}", fields.named.len());
            for field in fields.named.iter() {
                println!("{:?}", field.ident);
                let f_name = field.ident.as_ref().unwrap();
                let f_type = &field.ty;
                let token = quote! {
                    #f_name: Option<#f_type>,
                };
                declr_tokens.push(token);
                let token = quote! {
                    #f_name: None,
                };
                init_tokens.push(token);
                let token = quote! {
                    pub fn #f_name(&mut self, #f_name: #f_type) -> &mut Self {
                        self.#f_name = Some(#f_name);
                        self
                    }
                };
                fn_tokens.push(token);
                let msg = format!("'{}' field has not been set", stringify!(#f_name));
                let token = quote! {
                    let #f_name;
                    if let Some(ref val) = self.#f_name {
                        #f_name = val.clone();
                    } else {
                        return Err(#msg.to_string());
                    };
                };
                build_tokens.push(token);
                let token = quote! {
                    #f_name,
                };
                list_tokens.push(token);
            }
        } else {
            panic!("'Builder' can only be derived for structs with named fields.")
        }
    } else {
        panic!("'Builder' can only be derived for structs.")
    }

    let tokens = quote! {
        pub struct #builder_name {
            #(#declr_tokens)*
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#init_tokens)*
                }
            }
        }

        impl #builder_name {
            #(#fn_tokens)*

            pub fn build(&mut self) -> Result<#struct_name, String> {
                #(#build_tokens)*
                Ok(#struct_name {
                    #(#list_tokens)*
                })
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}
