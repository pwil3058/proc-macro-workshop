use proc_macro::TokenStream;
use syn;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let seq_data = syn::parse_macro_input!(input as SeqData);

    eprintln!("INNER STREAM: {:#?}", seq_data);
    TokenStream::new()
}

#[derive(Debug)]
struct SeqData {
    ident: syn::Ident,
    range: syn::ExprRange,
    token_stream: TokenStream,
}

impl syn::parse::Parse for SeqData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let _in = input.parse::<syn::Token![in]>()?;
        let range = input.parse::<syn::ExprRange>()?;
        let content;
        let _brace_token = syn::braced!(content in input);
        let token_stream = proc_macro2::TokenStream::parse(&content)?.into();
        Ok(Self {
            ident,
            range,
            token_stream,
        })
    }
}
