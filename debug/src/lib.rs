use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(CustomDebug)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &ast.ident;

    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => panic!("\"#[derive(Builder)]\" only implemented for structs with named fields"),
    };

    let field_tokens = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote! {
            .field(stringify!(#field_name), &self.#field_name)
        }
    });

    let tokens = quote! {
        impl std::fmt::Debug for #struct_name {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.debug_struct(stringify!(#struct_name))
                    #(#field_tokens)*
                    .finish()
            }
        }
    };

    tokens.into()
}
