use core::fmt;
use std::{collections::HashMap, str::FromStr};

use minimessage_impl::{
    parser::{Expression, Node, Parser},
    style::{self, ClickEvent, Decoration, HoverEvent, Special, SpecialError, rainbow::Rainbow},
    tokenizer::Tokenizer,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    #[error("A positional arg for position {_0} could not be found")]
    PositionalArgNotFound(usize),

    #[error("A positional arg for named argument {_0} could not be found")]
    NamedArgNotFound(String),

    Special(#[from] SpecialError),

    Parser(#[from] minimessage_impl::error::Error),
}

pub use minimessage_impl::style::NamedColor;

#[derive(Debug, Clone)]
pub enum ComponentColor {
    Named(NamedColor),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone)]
pub struct Component {
    pub text: String,
    pub color: Option<ComponentColor>,
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub strikethrough: bool,
    pub obfuscated: bool,
    pub click_event: Option<ClickEvent>,
    pub hover_event: Option<HoverEvent>,
    pub rainbow: Option<Rainbow>,
    pub children: Vec<Component>,
}

impl Component {
    pub fn text(text: impl Into<String>) -> Self {
        Component {
            text: text.into(),
            color: None,
            bold: false,
            italic: false,
            underlined: false,
            strikethrough: false,
            obfuscated: false,
            click_event: None,
            hover_event: None,
            rainbow: None,
            children: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct ArgumentCollection<'a> {
    positionals: Vec<Box<dyn fmt::Display + 'a>>,
    named: HashMap<&'a str, Box<dyn fmt::Display + 'a>>,
}

impl<'a> ArgumentCollection<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn positional(mut self, data: impl fmt::Display + 'a) -> Self {
        self.positionals.push(Box::new(data));
        self
    }

    pub fn named(mut self, name: &'a str, data: impl fmt::Display + 'a) -> Self {
        self.named.insert(name, Box::new(data));
        self
    }

    fn resolve(&self, name: &str) -> Option<String> {
        self.named.get(name).map(|val| val.to_string())
    }

    fn resolve_unnamed(&self, pos: usize) -> Option<String> {
        self.positionals.get(pos).map(|val| val.to_string())
    }
}

#[derive(Default)]
pub struct FormatArgs<'a> {
    args: ArgumentCollection<'a>,
    pos: usize,
}

impl<'a> FormatArgs<'a> {
    pub fn new() -> Self {
        FormatArgs {
            args: ArgumentCollection::new(),
            pos: 0,
        }
    }

    pub fn new_args(args: ArgumentCollection<'a>) -> Self {
        FormatArgs { args, pos: 0 }
    }

    fn resolve(&mut self, name: &str) -> Option<String> {
        self.args.resolve(name)
    }

    fn resolve_unnamed(&mut self) -> Option<String> {
        let value = self.args.resolve_unnamed(self.pos)?;
        self.pos += 1;
        Some(value)
    }
}

fn build_node(node: &Node, args: &mut FormatArgs) -> Result<Component> {
    match node {
        Node::Text(text) => Ok(Component::text(text.as_ref())),

        Node::Expression(Expression::Named(name)) => {
            let text = args
                .resolve(name)
                .ok_or_else(|| Error::NamedArgNotFound(name.to_string()))?;
            Ok(Component::text(text))
        },

        Node::Expression(Expression::Unnamed) => {
            let text = args
                .resolve_unnamed()
                .ok_or(Error::PositionalArgNotFound(args.pos))?;
            Ok(Component::text(text))
        },

        Node::Element {
            tag,
            children,
            tag_descriptors,
        } => {
            let mut comp = Component::text("");
            let mut matched = false;

            if let Ok(decoration) = Decoration::from_str(tag) {
                matched = true;

                match decoration {
                    Decoration::Bold => comp.bold = true,
                    Decoration::Italic => comp.italic = true,
                    Decoration::Underlined => comp.underlined = true,
                    Decoration::Strikethrough => comp.strikethrough = true,
                    Decoration::Obfuscated => comp.obfuscated = true,
                };
            }

            match Special::from_descriptor(tag, tag_descriptors.clone()) {
                Ok(special) => {
                    matched = true;
                    apply_special(&mut comp, special)?;
                },
                Err(SpecialError::NotFound(_)) => {},
                Err(err) => return Err(err.into()),
            }

            if let Ok(color) = style::NamedColor::from_str(tag) {
                matched = true;
                comp.color = Some(ComponentColor::Named(color));
            }

            if !matched {
                return Err(Error::Special(SpecialError::UnknownTag(tag.to_string())));
            }

            for child in children {
                let child_comp = build_node(child, args)?;
                comp.children.push(child_comp);
            }

            Ok(comp)
        },
    }
}

fn apply_special(comp: &mut Component, special: Special) -> Result<()> {
    match special {
        Special::Click(click) => {
            comp.click_event = Some(match click {
                style::ClickEvent::OpenUrl(url) => ClickEvent::OpenUrl(url),
                style::ClickEvent::RunCommand(cmd) => ClickEvent::RunCommand(cmd),
                style::ClickEvent::SuggestCommand(cmd) => ClickEvent::SuggestCommand(cmd),
                style::ClickEvent::CopyToClipboard(text) => ClickEvent::CopyToClipboard(text),
                style::ClickEvent::__Empty => return Ok(()),
            });
        },
        Special::Hover(hover) => {
            comp.hover_event = Some(match hover {
                style::HoverEvent::ShowText(text) => HoverEvent::ShowText(text),
                style::HoverEvent::ShowItem(item) => HoverEvent::ShowItem(item),
                style::HoverEvent::ShowEntity {
                    entity_type,
                    id,
                    name,
                } => HoverEvent::ShowEntity {
                    entity_type,
                    id,
                    name,
                },
                style::HoverEvent::__Empty => return Ok(()),
            });
        },
        Special::Color(style::Color(r, g, b)) => {
            comp.color = Some(ComponentColor::Rgb(r, g, b));
        },
        Special::Rainbow(rainbow) => {
            comp.rainbow = Some(rainbow);
        },
    }
    Ok(())
}

fn build_component(nodes: &[Node], args: &mut FormatArgs) -> Result<Component> {
    let mut root = Component::text("");
    for node in nodes {
        let child = build_node(node, args)?;
        root.children.push(child);
    }
    Ok(root)
}

pub fn deserialize(input: &str) -> Result<Component> {
    deserialize_with_args(input, ArgumentCollection::new())
}

pub fn deserialize_with_args(input: &str, args: ArgumentCollection<'_>) -> Result<Component> {
    let nodes =
        Parser::new(Tokenizer::new(input)).collect::<minimessage_impl::error::Result<Vec<_>>>()?;
    let mut fmt_args = FormatArgs::new_args(args);
    build_component(&nodes, &mut fmt_args)
}
