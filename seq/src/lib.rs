extern crate proc_macro;

use proc_macro::TokenStream;
use syn;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let _seq_data = syn::parse_macro_input!(input as SeqData);

    //eprintln!("SEQ_DATA: {:#?}", seq_data);

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
        let range = input.parse::<syn::ExprRange>()?;
        let block = input.parse::<syn::Block>()?;
        Ok(Self {
            ident,
            range,
            block,
        })
    }
}
