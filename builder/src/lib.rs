extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta, PathArguments, Type};

#[proc_macro_derive(Builder, attributes(builder))]
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
            for field in fields.named.iter() {
                let f_name = field.ident.as_ref().unwrap();
                println!("{:?}", f_name);
                println!("attributes: {:?}", field.attrs.len());
                let mut each: Option<syn::Ident> = None;
                for attr in field.attrs.iter() {
                    if attr.path.is_ident("builder") {
                        if let Ok(meta) = attr.parse_meta() {
                            match meta {
                                Meta::List(list) => {
                                    println!("list: {:?}", list.nested.len());
                                    for item in list.nested.iter() {
                                        match item {
                                            syn::NestedMeta::Meta(imeta) => {
                                                println!("meta");
                                                match imeta {
                                                    Meta::NameValue(inv) => {
                                                        println!("inv: {:?}", inv.path.get_ident(),);
                                                        if !inv.path.is_ident("each") {
                                                            let tokens = quote_spanned! {inv.path.get_ident().unwrap().span()=>
                                                            compile_error!(
                                                                "expected `builder(each = \"...\")`"
                                                            );
                                                            };
                                                            return proc_macro::TokenStream::from(
                                                                tokens,
                                                            );
                                                        };
                                                        if let syn::Lit::Str(lit_str) = &inv.lit {
                                                            println!(
                                                                "string: {:?}",
                                                                lit_str.value()
                                                            );
                                                            each = Some(syn::Ident::new(
                                                                &lit_str.value(),
                                                                lit_str.span(),
                                                            ));
                                                        } else {
                                                            panic!("panic #4")
                                                        }
                                                    }
                                                    _ => panic!("panic #3"),
                                                }
                                            }
                                            _ => panic!("panic #2"),
                                        }
                                    }
                                }
                                _ => panic!("panic #1"),
                            }
                        }
                    }
                }
                if let Type::Path(ref f_path_type) = field.ty {
                    let token = quote! {
                        #f_name: std::option::Option::None,
                    };
                    init_tokens.push(token);
                    let token = quote! {
                        #f_name,
                    };
                    list_tokens.push(token);
                    let segment = f_path_type.path.segments.first().unwrap();
                    match segment.ident.to_string().as_str() {
                        "Option" => {
                            let f_type =
                                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                                    &args.args
                                } else {
                                    panic!("Expected angle brackets");
                                };
                            println!("option");
                            let token = quote! {
                                #f_name: std::option::Option<#f_type>,
                            };
                            declr_tokens.push(token);
                            let token = quote! {
                                pub fn #f_name(&mut self, #f_name: #f_type) -> &mut Self {
                                    self.#f_name = std::option::Option::Some(#f_name);
                                    self
                                }
                            };
                            fn_tokens.push(token);
                            let token = quote! {
                                let #f_name;
                                if let Some(ref val) = self.#f_name {
                                    #f_name = std::option::Option::Some(val.clone());
                                } else {
                                    #f_name = std::option::Option::None
                                };
                            };
                            build_tokens.push(token);
                        }
                        other => {
                            println!("other: {:?}", other);
                            let f_type = f_path_type;
                            let token = quote! {
                                #f_name: std::option::Option<#f_type>,
                            };
                            declr_tokens.push(token);
                            let token = quote! {
                                pub fn #f_name(&mut self, #f_name: #f_type) -> &mut Self {
                                    self.#f_name = Some(#f_name);
                                    self
                                }
                            };
                            fn_tokens.push(token);
                            if let Some(each) = each {
                                if other != "Vec" {
                                    panic!("expected 'Vec'");
                                };
                                let token = quote! {
                                    pub fn #each(&mut self, #each: String) -> &mut Self {
                                        let v = self.#f_name.get_or_insert(vec![]);
                                        (*v).push(#each);
                                        self
                                    }
                                };
                                fn_tokens.push(token);
                                let token = quote! {
                                    let #f_name;
                                    if let std::option::Option::Some(ref val) = self.#f_name {
                                        #f_name = val.clone();
                                    } else {
                                        #f_name = vec![];
                                    };
                                };
                                build_tokens.push(token);
                            } else {
                                let msg =
                                    format!("'{}' field has not been set", stringify!(#f_name));
                                let token = quote! {
                                    let #f_name;
                                    if let std::option::Option::Some(ref val) = self.#f_name {
                                        #f_name = val.clone();
                                    } else {
                                        return std::result::Result::Err(#msg.to_string());
                                    };
                                };
                                build_tokens.push(token);
                            }
                        }
                    }
                } else {
                    panic!("'Builder' can only be derived normal field types")
                }
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

            pub fn build(&mut self) -> std::result::Result<#struct_name, String> {
                #(#build_tokens)*
                std::result::Result::Ok(#struct_name {
                    #(#list_tokens)*
                })
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}
