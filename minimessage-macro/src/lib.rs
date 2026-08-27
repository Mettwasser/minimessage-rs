use std::{env, fs};

use minimessage_impl::{parser::Parser, tokenizer::Tokenizer};
use proc_macro::TokenStream;
use quote::quote;

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

    let mut positional_idx = 0;
    let component = codegen::generate_component(&nodes, &input.args, &mut positional_idx);

    quote! {
        {
            use ::pumpkin_plugin_api::{
                common::{NamedColor, RgbColor},
                text::TextComponent,
            };
            use ::minimessage_rs::parser::style::rainbow::Rainbow;

            #component
        }
    }
    .into()
}
