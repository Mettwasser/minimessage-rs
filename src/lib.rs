pub mod parser {
    pub use minimessage_impl::{
        error::{Error, Result},
        parser::{Expression, Node, Parser},
        style,
        token::Token,
        tokenizer::Tokenizer,
    };
}
pub use minimessage_macro::minimessage;
pub use minimessage_runtime::{
    ArgumentCollection,
    Error,
    FormatArgs,
    Result,
    deserialize,
    deserialize_with_args,
};
