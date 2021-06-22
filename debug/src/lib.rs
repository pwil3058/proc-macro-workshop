use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use std::collections::hash_set::*;
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote};

fn fail(span: proc_macro2::Span, msg: &str) -> TokenStream {
    syn::Error::new(span, msg).into_compile_error().into()
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &ast.ident;

    let attributes: Vec<&syn::Attribute> = ast
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("debug"))
        .collect();
    if attributes.len() > 1 {
        return fail(attributes[1].span(), "multiple 'debug' attributes").into();
    };
    if let Some(attribute) = attributes.first() {
        match attribute.parse_meta() {
            Ok(syn::Meta::List(syn::MetaList { ref nested, .. })) => {
                for bound in nested {
                    eprintln!("bound: {:#?}", bound);
                }
            }
            _ => return fail(attribute.span(), "expected #[debug(bound = \"...\")]"),
        }
    }

    let generic_idents: HashSet<syn::Ident> = ast
        .generics
        .type_params()
        .map(|t| t.ident.clone())
        .collect();

    let mut where_predicates: HashSet<syn::WherePredicate> =
        if let Some(where_clause) = &ast.generics.where_clause {
            where_clause.predicates.iter().cloned().collect()
        } else {
            HashSet::new()
        };

    let mut viable_params: HashSet<syn::Ident> = HashSet::new();
    let mut field_tokens = vec![];

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

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        viable_params = &viable_params | &used_params(&field.ty, &generic_idents);
        where_predicates =
            &where_predicates | &associated_type_predicates(&field.ty, &generic_idents);
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
                    field_tokens.push(quote! {
                        .field(stringify!(#field_name), &format_args!(#lit, &self.#field_name))
                    });
                }
                _ => return fail(attribute.tokens.span(), "expected #[debug = \"...\"]").into(),
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
    let where_predicates: Vec<syn::WherePredicate> = where_predicates.into_iter().collect();
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let tokens = if where_predicates.len() > 0 {
        quote! {
            impl#impl_generics std::fmt::Debug for #struct_name #ty_generics where #(#where_predicates),* {
                fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    fmt.debug_struct(stringify!(#struct_name))
                        #(#field_tokens)*
                        .finish()
                }
            }
        }
    } else {
        quote! {
            impl#impl_generics std::fmt::Debug for #struct_name #ty_generics #where_clause {
                fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    fmt.debug_struct(stringify!(#struct_name))
                        #(#field_tokens)*
                        .finish()
                }
            }
        }
    };

    tokens.into()
}

fn segments_match_tail(
    segments: &syn::punctuated::Punctuated<syn::PathSegment, syn::token::Colon2>,
    names: &[&str],
) -> bool {
    if segments.len() > 0 && segments.len() <= names.len() {
        let start = names.len() - segments.len();
        segments
            .iter()
            .map(|s| &s.ident)
            .zip(names[start..].iter())
            .all(|(a, b)| a == b)
    } else {
        false
    }
}

fn is_phantom_data_type(path: &syn::Path) -> bool {
    segments_match_tail(&path.segments, &["std", "marker", "PhantomData"])
}

fn used_params(ty: &syn::Type, params: &HashSet<syn::Ident>) -> HashSet<syn::Ident> {
    let mut set = HashSet::new();
    if let syn::Type::Path(syn::TypePath { ref path, .. }) = ty {
        if !is_phantom_data_type(path) {
            if let Some(segment) = path.segments.last() {
                if path.segments.len() == 1 && params.contains(&segment.ident) {
                    set.insert(segment.ident.clone());
                }
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    ref args,
                    ..
                }) = segment.arguments
                {
                    for arg in args {
                        if let syn::GenericArgument::Type(ty) = arg {
                            set = &set | &used_params(ty, params);
                        }
                    }
                }
            }
        }
    }
    set
}

fn associated_type_predicates(
    ty: &syn::Type,
    params: &HashSet<syn::Ident>,
) -> HashSet<syn::WherePredicate> {
    let mut set = HashSet::new();
    if let syn::Type::Path(syn::TypePath { ref path, .. }) = ty {
        if !is_phantom_data_type(path) {
            if path.segments.len() == 2 && params.contains(&path.segments[0].ident) {
                let wp = syn::parse2::<syn::WherePredicate>(quote! { #ty: std::fmt::Debug });
                set.insert(wp.unwrap());
            }
            if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                ref args,
                ..
            }) = path.segments.last().unwrap().arguments
            {
                for arg in args {
                    if let syn::GenericArgument::Type(ty) = arg {
                        set = &set | &associated_type_predicates(ty, params);
                    }
                }
            }
        }
    }
    set
}
