use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote};

fn fail(span: proc_macro2::Span, msg: &str) -> TokenStream {
    syn::Error::new(span, msg).into_compile_error().into()
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::DeriveInput);

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

    let generic_idents: Vec<syn::Ident> = ast
        .generics
        .type_params()
        .map(|t| t.ident.clone())
        .collect();

    let mut viable_params = vec![];
    let mut field_tokens = vec![];
    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        if let syn::Type::Path(syn::TypePath { ref path, .. }) = field.ty {
            if let Some(segment) = path.segments.last() {
                if generic_idents.contains(&segment.ident) {
                    viable_params.push(segment.ident.clone())
                } else if segment.ident != "PhantomData" {
                    //eprintln!("VIABLE: {:?}", segment);
                }
            }
        }
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
                        field_tokens.push(quote! {
                            .field(stringify!(#field_name), &format_args!(#lit, &self.#field_name))
                        });
                    } else {
                        return fail(lit.span(), "expected string literal").into();
                    }
                }
                _ => return fail(attribute.span(), "expected #[debug = \"...\"]").into(),
            }
        } else {
            field_tokens.push(quote! {
                .field(stringify!(#field_name), &self.#field_name)
            });
        }
    }
    for param in &mut ast.generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            if viable_params.contains(&type_param.ident) {
                type_param.bounds.push(parse_quote!(std::fmt::Debug));
            }
        }
    }
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let tokens = quote! {
        impl#impl_generics std::fmt::Debug for #struct_name #ty_generics #where_clause {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.debug_struct(stringify!(#struct_name))
                    #(#field_tokens)*
                    .finish()
            }
        }
    };

    tokens.into()
}
