use proc_macro::TokenStream;
use syn;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let _seq_data = syn::parse_macro_input!(input as SeqData);

    TokenStream::new()
}

#[derive(Debug)]
struct SeqData {
    ident: syn::Ident,
    range: syn::ExprRange,
    block: syn::Block,
}

impl syn::parse::Parse for SeqData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let _in = input.parse::<syn::Token![in]>()?;
        Ok(Self {
            ident,
            range: input.parse::<syn::ExprRange>()?,
            block: input.parse::<syn::Block>()?,
        })
    }
}
