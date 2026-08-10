use std::{env, fs};

use minimessage_impl::{parser::Parser, tokenizer::Tokenizer};
use proc_macro::TokenStream;
use quote::{format_ident, quote};

mod codegen;
mod input;
mod resolve;

#[proc_macro]
pub fn minimessage(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as input::MacroInput);
    let mut value = input.format_str.value();

    if input.is_file {
        let path = std::path::Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(&value);
        value = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("File not found: {}", path.to_str().unwrap()));
    }

    let nodes = Parser::new(Tokenizer::new(&value))
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();

    let root = format_ident!("__root");
    let mut counter = 0;
    let mut positional_idx = 0;
    let child_code = codegen::generate_nodes(
        &nodes,
        &input.args,
        &mut positional_idx,
        &mut counter,
        &root,
    );

    quote! {
        {
            use ::pumpkin_plugin_api::{common::NamedColor, text::{TextComponent, RgbColor}};
            use minimessage_rs::parser::style::rainbow::Rainbow;

            let #root = TextComponent::text("");
            #child_code
            #root
        }
    }
    .into()
}
