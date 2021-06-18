use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    //eprintln!("input: {:#?}", ast);

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
        //eprintln!("Fields: {:#?}", field);
        let field_name = field.ident.as_ref().unwrap();
        field_name_tokens.push(quote! {#field_name,});
        let field_type = &field.ty;
        let token = quote! {
            #field_name: std::option::Option<#field_type>,
        };
        declr_tokens.push(token);
        let token = quote! {
            pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                self.#field_name = Some(#field_name);
                self
            }
        };
        fn_tokens.push(token);
        let msg = format!("'{}' field has not been set", stringify!(#field_name));
        let token = quote! {
            let #field_name = match self.#field_name {
                Some(ref #field_name) => #field_name.clone(),
                None => return Err(#msg.to_string().into()),
            };
        };
        build_tokens.push(token);
        // match field.ty {
        //     syn::Type::Path(syn::TypePath { ref path, .. }) => (),
        //     _ => panic!("\"#[derive(Builder)]\" only implemented for structs with named fields"),
        // }
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
