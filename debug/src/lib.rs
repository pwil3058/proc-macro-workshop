extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let parsed_input: syn::DeriveInput = syn::parse_macro_input!(input);
    let input_name = parsed_input.ident;
    let generics = add_debug_trait_bounds(parsed_input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mut field_names = vec![];
    let mut format_str: String = input_name.to_string() + " {{";
    match parsed_input.data {
        syn::Data::Struct(s) => {
            if let syn::Fields::Named(fields) = s.fields {
                for (i, field) in fields.named.iter().enumerate() {
                    let mut field_format: Option<String> = None;
                    for attr in field.attrs.iter() {
                        if attr.path.is_ident("debug") {
                            if let Ok(meta) = attr.parse_meta() {
                                match meta {
                                    syn::Meta::List(_list) => println!("list"),
                                    syn::Meta::Path(_path) => println!("path"),
                                    syn::Meta::NameValue(nv) => {
                                        if let syn::Lit::Str(lit_str) = &nv.lit {
                                            field_format = Some(lit_str.value());
                                        };
                                    }
                                }
                            }
                        }
                    }
                    let field_name = field.ident.as_ref().unwrap();
                    if let Some(ff) = field_format {
                        if i == 0 {
                            format_str += &format!(" {}: {}", field_name.to_string(), ff);
                        } else {
                            format_str += &format!(", {}: {}", field_name.to_string(), ff);
                        }
                    } else {
                        if i == 0 {
                            format_str += &format!(" {}: {{:?}}", field_name.to_string());
                        } else {
                            format_str += &format!(", {}: {{:?}}", field_name.to_string());
                        }
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
        impl #impl_generics std::fmt::Debug for #input_name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, #format_str #(#field_names)*)
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}

fn add_debug_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(syn::parse_quote!(std::fmt::Debug));
        }
    }
    generics
}
