use syn::{
    Expr,
    Ident,
    LitStr,
    Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

mod keywords {
    syn::custom_keyword!(file);
}

pub struct FormatArg {
    pub ident: Option<Ident>,
    pub expr: Expr,
}

impl Parse for FormatArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) && input.peek2(Token![=]) {
            let ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let expr = input.parse()?;
            Ok(FormatArg {
                ident: Some(ident),
                expr,
            })
        } else {
            let expr = input.parse()?;
            Ok(FormatArg { ident: None, expr })
        }
    }
}

pub struct MacroInput {
    pub is_file: bool,
    pub format_str: LitStr,
    pub args: Vec<FormatArg>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let is_file = input
            .peek(keywords::file)
            .then(|| {
                input.parse::<keywords::file>().unwrap();
                input.parse::<Token![:]>().unwrap();
            })
            .is_some();

        let format_str = input.parse()?;
        let args = if input.peek(Token![,]) {
            input.parse::<Token![,]>().unwrap();
            Punctuated::<FormatArg, Token![,]>::parse_terminated(input)
                .unwrap_or_default()
                .into_iter()
                .collect()
        } else {
            Vec::new()
        };

        Ok(MacroInput {
            format_str,
            args,
            is_file,
        })
    }
}
