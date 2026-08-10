use minimessage_impl::parser::Expression;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Expr, Ident, LitStr};

use crate::input::FormatArg;

pub fn resolve_named(
    name: &str,
    args: &[FormatArg],
    positional_idx: &mut usize,
) -> (TokenStream2, LitStr) {
    let (value_part, format_spec) = name.split_once(':').map_or((name, ""), |(v, s)| (v, s));

    if name.contains(':') && format_spec.is_empty() {
        panic!("empty format specifier after ':' in expression {{{name}}}");
    }

    let fmt_str = if format_spec.is_empty() {
        "{}".to_string()
    } else {
        format!("{{:{format_spec}}}")
    };
    let fmt_lit = LitStr::new(&fmt_str, Span::call_site());

    let value_expr = if value_part.is_empty() {
        let expr = &args
            .get(*positional_idx)
            .unwrap_or_else(|| panic!("missing positional format argument {}", *positional_idx))
            .expr;
        *positional_idx += 1;
        quote! { #expr }
    } else if let Ok(idx) = value_part.parse::<usize>() {
        let expr = &args
            .get(idx)
            .unwrap_or_else(|| panic!("missing positional format argument {idx}"))
            .expr;
        quote! { #expr }
    } else if let Some(arg) = args
        .iter()
        .find(|a| a.ident.as_ref().is_some_and(|i| i == value_part))
    {
        let expr = &arg.expr;
        quote! { #expr }
    } else {
        let ident = Ident::new(value_part, Span::call_site());
        quote! { #ident }
    };

    (value_expr, fmt_lit)
}

fn resolve_unnamed(args: &[FormatArg], positional_idx: &mut usize) -> Expr {
    let expr = args
        .get(*positional_idx)
        .unwrap_or_else(|| panic!("missing positional format argument {}", *positional_idx))
        .expr
        .clone();
    *positional_idx += 1;
    expr
}

pub fn resolve_expression(
    expr: &Expression,
    args: &[FormatArg],
    positional_idx: &mut usize,
) -> (TokenStream2, LitStr) {
    match expr {
        Expression::Named(name) => resolve_named(name, args, positional_idx),
        Expression::Unnamed => {
            let expr = resolve_unnamed(args, positional_idx);
            (quote! { #expr }, LitStr::new("{}", Span::call_site()))
        },
    }
}
