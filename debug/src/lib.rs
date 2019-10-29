extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed_input: syn::DeriveInput = syn::parse_macro_input!(input);
    let input_name = parsed_input.ident;
    let mut field_names = vec![];
    let mut format_str: String = input_name.to_string() + " {{";
    match parsed_input.data {
        syn::Data::Struct(s) => {
            if let syn::Fields::Named(fields) = s.fields {
                for (i, field) in fields.named.iter().enumerate() {
                    let field_name = field.ident.as_ref().unwrap();
                    if i == 0 {
                        format_str += &format!(" {}: {{:?}}", field_name.to_string());
                    } else {
                        format_str += &format!(", {}: {{:?}}", field_name.to_string());
                    }
                    let token = quote! {
                        , self.#field_name
                    };
                    field_names.push(token);
                }
            }
        }
        _ => panic!("not yet implemented"),
    }
    format_str += " }}";

    let tokens = quote! {
        impl std::fmt::Debug for #input_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #format_str #(#field_names)*)
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}
