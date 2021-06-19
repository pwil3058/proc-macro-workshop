use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &ast.ident;
    let builder_name = Ident::new(&format!("{}Builder", struct_name), Span::call_site());

    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => panic!("\"#[derive(Builder)]\" only implemented for structs with named fields"),
    };

    let mut field_name_tokens = vec![];
    let mut declr_tokens = vec![];
    let mut fn_tokens = vec![];
    let mut build_tokens = vec![];
    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let msg = format!("'{}' field has not been set", stringify!(#field_name));
        field_name_tokens.push(quote! {#field_name,});
        let mut each: Option<syn::Ident> = None;
        for attr in field.attrs.iter().filter(|a| a.path.is_ident("builder")) {
            match attr.parse_meta() {
                Ok(syn::Meta::List(ref list)) => {
                    //syn::MetaList { ref nested, .. })) => {
                    assert_eq!(list.nested.len(), 1);
                    match list.nested.first() {
                        Some(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                            ref path,
                            ref lit,
                            ..
                        }))) => {
                            if !path.is_ident("each") {
                                return syn::Error::new_spanned(
                                    list,
                                    "expected `builder(each = \"...\")`",
                                )
                                .to_compile_error()
                                .into();
                            }
                            if let syn::Lit::Str(ref lit_str) = lit {
                                each = Some(syn::Ident::new(&lit_str.value(), lit_str.span()))
                            } else {
                                panic!("whatever")
                            }
                        }
                        _ => panic!("whatever"),
                    }
                }
                _ => panic! {"Expected 'each=<name>'"},
            }
        }
        match field.ty {
            syn::Type::Path(syn::TypePath { ref path, .. }) => {
                let segment = path.segments.first().unwrap();
                if segment.ident == "Option" {
                    assert!(each.is_none());
                    let option_arg = match segment.arguments {
                        syn::PathArguments::AngleBracketed(ref args) => &args.args,
                        _ => panic!("expected angle brackets"),
                    };
                    declr_tokens.push(quote! {
                        #field_name: std::option::Option<#option_arg>,
                    });
                    fn_tokens.push(quote! {
                        pub fn #field_name(&mut self, #field_name: #option_arg) -> &mut Self {
                            self.#field_name = Some(#field_name);
                            self
                        }
                    });
                    build_tokens.push(quote! {
                        let #field_name = match self.#field_name {
                            Some(ref #field_name) => Some(#field_name.clone()),
                            None => None,
                        };
                    });
                } else if let Some(each) = each {
                    assert_eq!(segment.ident, "Vec");
                    let field_type = &field.ty;
                    declr_tokens.push(quote! {
                        #field_name: #field_type,
                    });
                    let vec_type = match segment.arguments {
                        syn::PathArguments::AngleBracketed(ref args) => &args.args,
                        _ => panic!("expected angle brackets"),
                    };
                    fn_tokens.push(quote! {
                        pub fn #each(&mut self, #each: #vec_type) -> &mut Self {
                            self.#field_name.push(#each);
                            self
                        }
                    });
                    if each.to_string() != field_name.to_string() {
                        fn_tokens.push(quote! {
                            pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                                self.#field_name.extend(#field_name);
                                self
                            }
                        });
                    }
                    build_tokens.push(quote! {
                        let #field_name = self.#field_name.clone();
                    });
                } else {
                    let field_type = &field.ty;
                    declr_tokens.push(quote! {
                        #field_name: std::option::Option<#field_type>,
                    });
                    fn_tokens.push(quote! {
                        pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                            self.#field_name = Some(#field_name);
                            self
                        }
                    });
                    build_tokens.push(quote! {
                        let #field_name = match self.#field_name {
                            Some(ref #field_name) => #field_name.clone(),
                            None => return Err(#msg.to_string().into()),
                        };
                    });
                }
            }
            _ => panic!("expected TypePath"),
        };
    }

    let tokens = quote!(
        #[derive(Default)]
        pub struct #builder_name {
            #(#declr_tokens)*
        }

        impl #builder_name {
            #(#fn_tokens)*

            pub fn build(&self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                #(#build_tokens)*

                Ok(#struct_name{#(#field_name_tokens)*})
            }
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name::default()
            }
        }
    );

    TokenStream::from(tokens)
}
