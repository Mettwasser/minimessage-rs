use std::str::FromStr;

use heck::ToPascalCase;
use minimessage_impl::{
    parser::{Node, Parser},
    style::{ClickEvent, Decoration, HoverEvent, Special, color::Color},
    tokenizer::Tokenizer,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Ident, LitStr};

use crate::{input::FormatArg, resolve};

pub fn var_ids(counter: &mut usize) -> (Ident, Ident) {
    let idx = *counter;
    *counter += 1;
    (format_ident!("__c{idx}"), format_ident!("__t{idx}"))
}

fn tag_to_color_code(tag: &str) -> TokenStream2 {
    let pascal = tag.to_pascal_case();
    let ident = format_ident!("{pascal}");
    quote! { NamedColor::#ident }
}

fn decoration_code(decoration: Decoration, var: &Ident) -> TokenStream2 {
    let call = match decoration {
        Decoration::Bold => quote! { bold(true) },
        Decoration::Italic => quote! { italic(true) },
        Decoration::Underlined => quote! { underlined(true) },
        Decoration::Strikethrough => quote! { strikethrough(true) },
        Decoration::Obfuscated => quote! { obfuscated(true) },
    };
    quote! { #var.#call; }
}

fn text_node_code(text: &str, parent: &Ident, counter: &mut usize) -> TokenStream2 {
    let (var, _) = var_ids(counter);
    quote! {
        let #var = TextComponent::text(#text);
        #parent.add_child(#var);
    }
}

pub fn format_node_code(
    value_expr: TokenStream2,
    fmt_lit: &LitStr,
    parent: &Ident,
    counter: &mut usize,
) -> TokenStream2 {
    let (var, text_var) = var_ids(counter);
    quote! {
        let #text_var = format!(#fmt_lit, #value_expr);
        let #var = TextComponent::text(&#text_var);
        #parent.add_child(#var);
    }
}

fn combined_string_code(parts: Vec<TokenStream2>) -> TokenStream2 {
    if parts.is_empty() {
        return quote! { String::new() };
    }
    if parts.len() == 1 {
        return parts.into_iter().next().unwrap();
    }
    let mut iter = parts.into_iter();
    let first = iter.next().unwrap();
    iter.fold(first, |acc, p| quote! { #acc + &(#p) })
}

fn stringify_nodes(nodes: &[Node], args: &[FormatArg], positional_idx: &mut usize) -> TokenStream2 {
    let mut parts = Vec::new();
    for node in nodes {
        match node {
            Node::Text(text) => parts.push(quote! { #text.to_string() }),
            Node::Expression(expr) => {
                let (val, fmt) = resolve::resolve_expression(expr, args, positional_idx);
                parts.push(quote! { format!(#fmt, #val) });
            },
            Node::Element { children, .. } => {
                parts.push(stringify_nodes(children, args, positional_idx));
            },
        }
    }
    combined_string_code(parts)
}

fn special_code(special: Special, var: Ident, counter: &mut usize) -> TokenStream2 {
    match special {
        Special::Click(click) => match click {
            ClickEvent::OpenUrl(url) => quote! { #var.click_open_url(#url); },
            ClickEvent::RunCommand(cmd) => quote! { #var.click_run_command(#cmd); },
            ClickEvent::SuggestCommand(cmd) => quote! { #var.click_suggest_command(#cmd); },
            ClickEvent::CopyToClipboard(text) => quote! { #var.click_copy_to_clipboard(#text); },
            ClickEvent::__Empty => quote! { compile_error!("No"); },
        },
        Special::Hover(hover) => match hover {
            HoverEvent::ShowEntity {
                entity_type,
                id,
                name,
            } => {
                let name = name.map(|n| quote!(Some(#n))).unwrap_or(quote!(None));
                quote! { #var.hover_show_entity(#entity_type, #id, #name); }
            },
            HoverEvent::ShowItem(item) => quote! { #var.hover_show_item(#item); },
            HoverEvent::ShowText(text) => {
                let nodes = Parser::new(Tokenizer::new(&text))
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .unwrap();
                let mut positional = 0;
                let child_code = generate_nodes(&nodes, &[], &mut positional, counter, &var);
                quote! { #child_code }
            },
            HoverEvent::__Empty => quote! { compile_error!("No"); },
        },
        Special::Color(Color(r, g, b)) => {
            quote! { #var.color_rgb(RgbColor { r: #r, g: #g, b: #b }); }
        },
        Special::Rainbow(_) => {
            panic!("Rainbow should be handled at the element level, not here");
        },
    }
}

fn generate_element(
    tag: &str,
    children: &[Node],
    tag_descriptors: &[std::borrow::Cow<'_, str>],
    args: &[FormatArg],
    positional_idx: &mut usize,
    counter: &mut usize,
    parent: &Ident,
) -> TokenStream2 {
    if let Ok(Special::Rainbow(rainbow)) = Special::from_descriptor(tag, tag_descriptors.to_vec()) {
        let combined = stringify_nodes(children, args, positional_idx);
        let inverted = rainbow.inverted;
        let offset = rainbow.offset;
        return quote! {
            {
                let __text = #combined;
                if !__text.is_empty() {
                    let __rainbow = Rainbow { inverted: #inverted, offset: #offset };
                    for (__color, __ch) in __rainbow.text(&__text) {
                        let __comp = TextComponent::text(&__ch.to_string());
                        __comp.color_rgb(RgbColor {
                            r: __color.0,
                            g: __color.1,
                            b: __color.2,
                        });
                        #parent.add_child(__comp);
                    }
                }
            }
        };
    }

    let (var, _) = var_ids(counter);
    let child_code = generate_nodes(children, args, positional_idx, counter, &var);

    let style_code = if let Ok(decoration) = Decoration::from_str(tag) {
        decoration_code(decoration, &var)
    } else if !tag_descriptors.is_empty() {
        match Special::from_descriptor(tag, tag_descriptors.to_vec()) {
            Ok(special) => special_code(special, var.clone(), counter),
            Err(e) => {
                let msg = e.to_string();
                quote! { compile_error!(#msg); }
            },
        }
    } else {
        let color = tag_to_color_code(tag);
        quote! { #var.color_named(#color); }
    };

    quote! {
        let #var = TextComponent::text("");
        #style_code
        #child_code
        #parent.add_child(#var);
    }
}

pub fn generate_nodes(
    nodes: &[Node],
    args: &[FormatArg],
    positional_idx: &mut usize,
    counter: &mut usize,
    parent: &Ident,
) -> TokenStream2 {
    let mut code = TokenStream2::new();
    for node in nodes {
        match node {
            Node::Text(text) => {
                code.extend(text_node_code(text, parent, counter));
            },
            Node::Expression(expr) => {
                let (val, fmt) = resolve::resolve_expression(expr, args, positional_idx);
                code.extend(format_node_code(val, &fmt, parent, counter));
            },
            Node::Element {
                tag,
                children,
                tag_descriptors,
            } => {
                code.extend(generate_element(
                    tag,
                    children,
                    tag_descriptors,
                    args,
                    positional_idx,
                    counter,
                    parent,
                ));
            },
        }
    }
    code
}
