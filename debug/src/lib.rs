use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;

fn fail(span: proc_macro2::Span, msg: &str) -> TokenStream {
    syn::Error::new(span, msg).into_compile_error().into()
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &ast.ident;

    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => {
            return fail(
                ast.ident.span(),
                "\"#[derive(CustomDebug)]\" only implemented for structs with named fields",
            )
        }
    };

    let field_tokens = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let attributes: Vec<&syn::Attribute> = field
            .attrs
            .iter()
            .filter(|a| a.path.is_ident("debug"))
            .collect();
        if attributes.len() > 1 {
            let msg = format!(
                "multiple 'debug' attributes for {}",
                stringify!(#field_name)
            );
            return fail(attributes[1].span(), &msg).into();
        };
        if let Some(attribute) = attributes.first() {
            match attribute.parse_meta() {
                Ok(syn::Meta::NameValue(syn::MetaNameValue { ref lit, .. })) => {
                    if let syn::Lit::Str(ref lit) = lit {
                        eprintln!("LIT: {:#?}", lit.value());
                        quote! {
                            .field(stringify!(#field_name), &format_args!(#lit, &self.#field_name))
                        }
                    } else {
                        return fail(lit.span(), "expected string literal").into();
                    }
                }
                _ => return fail(attribute.span(), "expected #[debug = \"...\"]").into(),
            }
        } else {
            quote! {
                .field(stringify!(#field_name), &self.#field_name)
            }
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
