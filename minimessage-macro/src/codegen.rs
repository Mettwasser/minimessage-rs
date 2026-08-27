use std::{borrow::Cow, collections::HashMap, str::FromStr};

use heck::ToPascalCase;
use minimessage_impl::{
    parser::{Node, Parser},
    style::{ClickEvent, Decoration, HoverEvent, Special, color::Color, rainbow::Rainbow},
    tokenizer::Tokenizer,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::{input::FormatArg, resolve};

enum ElementKind {
    Decoration(Decoration),
    Special(Special),
    NamedColor,
}

impl ElementKind {
    fn overrides_rainbow(&self) -> bool {
        matches!(
            self,
            Self::NamedColor | Self::Special(Special::Color(_) | Special::Rainbow(_))
        )
    }
}

fn classify_element(tag: &str, descriptors: &[Cow<'_, str>]) -> Result<ElementKind, String> {
    if descriptors.is_empty() {
        if let Ok(decoration) = Decoration::from_str(tag) {
            return Ok(ElementKind::Decoration(decoration));
        }

        if let Ok(special @ Special::Rainbow(_)) =
            Special::from_descriptor(tag, descriptors.to_vec())
        {
            return Ok(ElementKind::Special(special));
        }

        return Ok(ElementKind::NamedColor);
    }

    Special::from_descriptor(tag, descriptors.to_vec())
        .map(ElementKind::Special)
        .map_err(|error| error.to_string())
}

fn tag_to_color_code(tag: &str) -> TokenStream2 {
    let ident = format_ident!("{}", tag.to_pascal_case());
    quote! { NamedColor::#ident }
}

#[derive(Clone)]
struct RainbowContext {
    rainbow: proc_macro2::Ident,
    total: proc_macro2::Ident,
    index: proc_macro2::Ident,
}

#[derive(Default)]
struct PreparedExpressions {
    bindings: Vec<TokenStream2>,
    values: HashMap<usize, proc_macro2::Ident>,
}

struct Generator<'a, 'b> {
    args: &'a [FormatArg],
    positional_idx: &'b mut usize,
    next_id: usize,
}

impl<'a, 'b> Generator<'a, 'b> {
    fn ident(&mut self, name: &str) -> proc_macro2::Ident {
        let id = self.next_id;
        self.next_id += 1;
        format_ident!("__{name}_{id}")
    }

    fn prepare_expressions(&mut self, nodes: &[Node], prepared: &mut PreparedExpressions) {
        for node in nodes {
            match node {
                Node::Expression(expression) => {
                    let (value, format) =
                        resolve::resolve_expression(expression, self.args, self.positional_idx);
                    let ident = self.ident("text");
                    prepared.bindings.push(quote! {
                        let #ident = format!(#format, #value);
                    });
                    prepared.values.insert(node_key(node), ident);
                },
                Node::Element { children, .. } => self.prepare_expressions(children, prepared),
                Node::Text(_) => {},
            }
        }
    }

    fn expression_text(
        &mut self,
        node: &Node,
        expression: &minimessage_impl::parser::Expression,
        prepared: Option<&PreparedExpressions>,
    ) -> TokenStream2 {
        if let Some(ident) = prepared.and_then(|prepared| prepared.values.get(&node_key(node))) {
            return quote! { #ident };
        }

        let (value, format) =
            resolve::resolve_expression(expression, self.args, self.positional_idx);
        quote! { format!(#format, #value) }
    }

    fn decoration_code(&self, decoration: Decoration, component: TokenStream2) -> TokenStream2 {
        match decoration {
            Decoration::Bold => quote! { #component.bold(true) },
            Decoration::Italic => quote! { #component.italic(true) },
            Decoration::Underlined => quote! { #component.underlined(true) },
            Decoration::Strikethrough => quote! { #component.strikethrough(true) },
            Decoration::Obfuscated => quote! { #component.obfuscated(true) },
        }
    }

    fn special_code(&mut self, special: Special, component: TokenStream2) -> TokenStream2 {
        match special {
            Special::Click(click) => match click {
                ClickEvent::OpenUrl(url) => quote! { #component.click_open_url(#url) },
                ClickEvent::RunCommand(command) => {
                    quote! { #component.click_run_command(#command) }
                },
                ClickEvent::SuggestCommand(command) => {
                    quote! { #component.click_suggest_command(#command) }
                },
                ClickEvent::CopyToClipboard(text) => {
                    quote! { #component.click_copy_to_clipboard(#text) }
                },
                ClickEvent::__Empty => quote! {{
                    compile_error!("Invalid empty click event");
                    #component
                }},
            },
            Special::Hover(hover) => match hover {
                HoverEvent::ShowEntity {
                    entity_type,
                    id,
                    name,
                } => {
                    let name = name
                        .map(|name| quote!(Some(TextComponent::text(#name))))
                        .unwrap_or(quote!(None));
                    quote! { #component.hover_show_entity(#entity_type, #id, #name) }
                },
                HoverEvent::ShowItem(item) => quote! { #component.hover_show_item(#item) },
                HoverEvent::ShowText(text) => {
                    let nodes = Parser::new(Tokenizer::new(&text))
                        .collect::<std::result::Result<Vec<_>, _>>()
                        .unwrap();
                    let mut positional_idx = 0;
                    let mut generator = Generator {
                        args: &[],
                        positional_idx: &mut positional_idx,
                        next_id: self.next_id,
                    };
                    let hover_component = generator.component_code(&nodes, None, None);
                    self.next_id = generator.next_id;
                    quote! { #component.hover_show_text(#hover_component) }
                },
                HoverEvent::__Empty => quote! {{
                    compile_error!("Invalid empty hover event");
                    #component
                }},
            },
            Special::Color(Color(r, g, b)) => {
                quote! { #component.color_rgb(RgbColor { r: #r, g: #g, b: #b }) }
            },
            Special::Rainbow(_) => component,
        }
    }

    fn rainbow_text_code(&self, text: TokenStream2, context: &RainbowContext) -> TokenStream2 {
        let RainbowContext {
            rainbow,
            total,
            index,
        } = context;

        quote! {
            #text.chars().fold(TextComponent::text(""), |__component, __ch| {
                let __color = #rainbow.color_at(#index, #total);
                #index += 1;
                __component.add_child(
                    TextComponent::text(&__ch.to_string()).color_rgb(RgbColor {
                        r: __color.0,
                        g: __color.1,
                        b: __color.2,
                    })
                )
            })
        }
    }

    fn rainbow_char_count(&self, nodes: &[Node], prepared: &PreparedExpressions) -> TokenStream2 {
        let mut count = quote! { 0usize };

        for node in nodes {
            let child_count = match node {
                Node::Text(text) => quote! { #text.chars().count() },
                Node::Expression(_) => {
                    let ident = prepared.values.get(&node_key(node)).unwrap();
                    quote! { #ident.chars().count() }
                },
                Node::Element {
                    tag,
                    children,
                    tag_descriptors,
                } => match classify_element(tag, tag_descriptors) {
                    Ok(kind) if !kind.overrides_rainbow() => {
                        self.rainbow_char_count(children, prepared)
                    },
                    _ => quote! { 0usize },
                },
            };
            count = quote! { #count + #child_count };
        }

        count
    }

    fn rainbow_element_code(
        &mut self,
        rainbow: Rainbow,
        children: &[Node],
        prepared: &PreparedExpressions,
    ) -> TokenStream2 {
        let rainbow_ident = self.ident("rainbow");
        let total_ident = self.ident("rainbow_total");
        let index_ident = self.ident("rainbow_index");
        let inverted = rainbow.inverted;
        let offset = rainbow.offset;
        let char_count = self.rainbow_char_count(children, prepared);
        let context = RainbowContext {
            rainbow: rainbow_ident.clone(),
            total: total_ident.clone(),
            index: index_ident.clone(),
        };
        let component = self.component_code(children, Some(prepared), Some(context));

        quote! {{
            let #rainbow_ident = Rainbow { inverted: #inverted, offset: #offset };
            let #total_ident = #char_count;
            let mut #index_ident = 0usize;
            #component
        }}
    }

    fn element_code(
        &mut self,
        tag: &str,
        children: &[Node],
        tag_descriptors: &[Cow<'_, str>],
        prepared: Option<&PreparedExpressions>,
        inherited_rainbow: Option<RainbowContext>,
    ) -> TokenStream2 {
        let kind = match classify_element(tag, tag_descriptors) {
            Ok(kind) => kind,
            Err(message) => {
                return quote! {{
                    compile_error!(#message);
                    TextComponent::text("")
                }};
            },
        };

        if let ElementKind::Special(Special::Rainbow(rainbow)) = kind {
            if let Some(prepared) = prepared {
                return self.rainbow_element_code(rainbow, children, prepared);
            }

            let mut prepared = PreparedExpressions::default();
            self.prepare_expressions(children, &mut prepared);
            let bindings = &prepared.bindings;
            let component = self.rainbow_element_code(rainbow, children, &prepared);
            return quote! {{
                #(#bindings)*
                #component
            }};
        }

        let overrides_rainbow = kind.overrides_rainbow();
        let base = quote! { TextComponent::text("") };
        let mut component = match kind {
            ElementKind::Decoration(decoration) => self.decoration_code(decoration, base),
            ElementKind::Special(special) => self.special_code(special, base),
            ElementKind::NamedColor => {
                let color = tag_to_color_code(tag);
                quote! { TextComponent::text("").color_named(#color) }
            },
        };
        let child_rainbow = if overrides_rainbow {
            None
        } else {
            inherited_rainbow
        };

        for child in children {
            let child = self.node_code(child, prepared, child_rainbow.clone());
            component = quote! { #component.add_child(#child) };
        }

        component
    }

    fn node_code(
        &mut self,
        node: &Node,
        prepared: Option<&PreparedExpressions>,
        rainbow: Option<RainbowContext>,
    ) -> TokenStream2 {
        match node {
            Node::Text(text) => match rainbow {
                Some(context) => self.rainbow_text_code(quote! { #text }, &context),
                None => quote! { TextComponent::text(#text) },
            },
            Node::Expression(expression) => {
                let text = self.expression_text(node, expression, prepared);
                match rainbow {
                    Some(context) => self.rainbow_text_code(text, &context),
                    None => quote! { TextComponent::text(&#text) },
                }
            },
            Node::Element {
                tag,
                children,
                tag_descriptors,
            } => self.element_code(tag, children, tag_descriptors, prepared, rainbow),
        }
    }

    fn component_code(
        &mut self,
        nodes: &[Node],
        prepared: Option<&PreparedExpressions>,
        rainbow: Option<RainbowContext>,
    ) -> TokenStream2 {
        let mut component = quote! { TextComponent::text("") };
        for node in nodes {
            let child = self.node_code(node, prepared, rainbow.clone());
            component = quote! { #component.add_child(#child) };
        }
        component
    }
}

fn node_key(node: &Node) -> usize {
    std::ptr::from_ref(node) as usize
}

pub fn generate_component(
    nodes: &[Node],
    args: &[FormatArg],
    positional_idx: &mut usize,
) -> TokenStream2 {
    Generator {
        args,
        positional_idx,
        next_id: 0,
    }
    .component_code(nodes, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate(input: &str) -> String {
        let nodes = Parser::new(Tokenizer::new(input))
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();
        generate_component(&nodes, &[], &mut 0).to_string()
    }

    #[test]
    fn nested_color_overrides_rainbow() {
        let code = generate(
            "<hover:show_text:\"<rainbow>Hello world\"><rainbow>Hover me! <blue>for a surprise",
        );

        assert!(code.contains("color_at"));
        assert!(code.contains("color_named (NamedColor :: Blue)"));
        assert!(code.contains("TextComponent :: text (\"for a surprise\")"));
    }

    #[test]
    fn rainbow_cursor_is_shared_through_decorations() {
        let code = generate("<rainbow>a<bold>b</bold>c</rainbow>");

        assert_eq!(code.matches("let mut __rainbow_index").count(), 1);
        assert_eq!(code.matches("color_at").count(), 3);
    }

    #[test]
    fn rainbow_expressions_are_evaluated_once() {
        let nodes = Parser::new(Tokenizer::new("<rainbow><bold>{}</bold></rainbow>"))
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();
        let args = [FormatArg {
            ident: None,
            expr: syn::parse_quote!(value()),
        }];
        let code = generate_component(&nodes, &args, &mut 0).to_string();

        assert_eq!(code.matches("value ()").count(), 1);
    }

    #[test]
    fn components_are_generated_as_builder_expressions() {
        let code = generate("<blue>Hello <bold>world</bold></blue>");

        assert!(!code.contains("let mut"));
        assert!(!code.contains("="));
        assert!(code.contains("color_named"));
        assert!(code.contains("bold"));
        assert!(code.contains("add_child"));
    }
}
