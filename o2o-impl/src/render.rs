mod applicable_attr;
mod impl_context;

#[cfg(feature = "syn2")]
use syn2 as syn;

pub(super) use syn::{Member::Named, Member::Unnamed};
pub(super) use quote::format_ident;

pub(crate) use applicable_attr::*;
pub(crate) use impl_context::*;

use crate::model::*;

pub(crate) fn render_action(expr: &InlineExpression, tilde_postfix: Option<&TokenStream>, ctx: &ImplContext) -> TokenStream {
    let dst = ctx.dst_ty;
    let ident = match ctx.kind {
        Kind::FromOwned | Kind::FromRef => quote!(value),
        _ => quote!(self),
    };
    let path = match ctx.impl_type {
        ImplType::Struct => quote!(#tilde_postfix),
        ImplType::Enum => quote!(#dst::#tilde_postfix),
        ImplType::Variant => quote!(#tilde_postfix),
    };
    replace_tilde_or_at_in_expr(&expr, Some(&ident), Some(&path))
}

pub(crate) fn replace_tilde_or_at_in_expr(input: &InlineExpression, at_tokens: Option<&TokenStream>, tilde_tokens: Option<&TokenStream>) -> TokenStream {
    let mut tokens = Vec::new();

    input.expr.clone().into_iter().for_each(|x| {
        let f = match x {
            proc_macro2::TokenTree::Group(group) => {
                let inner = replace_tilde_or_at_in_expr(&InlineExpression { expr: group.stream() }, at_tokens, tilde_tokens);
                match group.delimiter() {
                    proc_macro2::Delimiter::Parenthesis => quote!(( #inner )),
                    proc_macro2::Delimiter::Brace => quote!({ #inner }),
                    proc_macro2::Delimiter::Bracket => quote!([ #inner ]),
                    proc_macro2::Delimiter::None => quote!(#inner),
                }
            }
            proc_macro2::TokenTree::Punct(punct) => {
                let ch = punct.as_char();

                if ch == '~' {
                    quote!(#tilde_tokens)
                } else if ch == '@' {
                    quote!(#at_tokens)
                } else {
                    quote!(#punct)
                }
            }
            _ => quote!(#x),
        };

        tokens.push(f)
    });

    TokenStream::from_iter(tokens)
}