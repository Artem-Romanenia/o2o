mod applicable_attr;

mod implementation;
mod attribute;
mod function;
mod function_body;
mod generics;
mod where_clause;
mod struct_init_block;
mod enum_init;
mod expression;

pub(crate) use applicable_attr::*;

pub(crate) use implementation::*;
pub(crate) use attribute::*;
pub(crate) use function::*;
pub(crate) use function_body::*;
pub(crate) use generics::*;
pub(crate) use where_clause::*;
pub(crate) use struct_init_block::*;
pub(crate) use enum_init::*;
pub(crate) use expression::*;

#[cfg(feature = "syn2")]
use syn2 as syn;

use syn::{Ident, Index, Member, Member::Named, Member::Unnamed, Generics, AngleBracketedGenericArguments, GenericParam};
use quote::{format_ident, quote, ToTokens};
use proc_macro2::{TokenStream, Span};

use crate::model::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ImplType {
    Struct,
    Enum,
    Variant,
}

impl ImplType {
    pub(crate) fn is_variant(self) -> bool {
        self == ImplType::Variant
    }
}

pub(crate) struct RenderContext<'a> {
    pub impl_type: ImplType,
    pub kind: Kind,
    pub dst_ty: &'a TokenStream,
    pub src_ty: &'a TokenStream,
    pub has_post_init: bool,
    pub fallible: bool,
}

impl<'a> RenderContext<'a> {
    pub(crate) fn with<'b, T: Render + ?Sized>(&'b self, node: &'b T) -> WithCtx<'b, T> {
        WithCtx(node, self)
    }
}

pub(crate) trait Render: std::fmt::Debug {
    fn render(&self, ctx: &RenderContext<'_>, stream: &mut TokenStream);

    #[inline]
    fn render_imm(&self, ctx: &RenderContext) -> TokenStream {
        let mut stream = TokenStream::new();
        self.render(ctx, &mut stream);
        stream
    }
}

pub(crate) struct WithCtx<'a, T: ?Sized>(&'a T, &'a RenderContext<'a>);

impl<T: Render + ?Sized> ToTokens for WithCtx<'_, T> {
    fn to_tokens(&self, out: &mut TokenStream) {
        self.0.render(self.1, out);
    }
}

pub(crate) fn render_action(expr: &TokenStream, tilde_postfix: Option<&TokenStream>, ctx: &RenderContext) -> TokenStream {
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

pub(crate) fn replace_tilde_or_at_in_expr(expr: &TokenStream, at_tokens: Option<&TokenStream>, tilde_tokens: Option<&TokenStream>) -> TokenStream {
    let mut tokens = Vec::new();

    expr.clone().into_iter().for_each(|x| {
        let f = match x {
            proc_macro2::TokenTree::Group(group) => {
                let inner = replace_tilde_or_at_in_expr(&group.stream(), at_tokens, tilde_tokens);
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