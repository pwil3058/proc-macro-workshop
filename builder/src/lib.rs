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

    let mut fn_tokens = vec![];
    for field in fields.iter() {
        //eprintln!("Fields: {:#?}", field);
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let token = quote! {
            pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
                let _ = #field_name;
                self
            }
        };
        fn_tokens.push(token);
        // match field.ty {
        //     syn::Type::Path(syn::TypePath { ref path, .. }) => (),
        //     _ => panic!("\"#[derive(Builder)]\" only implemented for structs with named fields"),
        // }
    }

    let tokens = quote!(
        pub struct #builder_name {
        }

        impl #builder_name {
            #(#fn_tokens)*
        }

        impl #struct_name {
            pub fn builder() -> #builder_name {
                #builder_name {}
            }
        }
    );

    TokenStream::from(tokens)
}
