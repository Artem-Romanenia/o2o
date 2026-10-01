use std::borrow::Cow;
use crate::render::*;

#[derive(Debug)]
pub(crate) struct Expression<'a> {
    pub expr: Cow<'a, TokenStream>,
    pub postfix: Option<TokenStream>,
    pub at_tokens: Option<Box<Expression<'a>>>,
    pub tilde_tokens: Option<Box<Expression<'a>>>
}

impl<'a> Expression<'a> {
    pub fn empty() -> Expression<'a> {
        Expression { expr: Cow::Owned(TokenStream::new()), postfix: None, at_tokens: None, tilde_tokens: None }
    }

    pub fn new_owned(expr: TokenStream) -> Expression<'a> {
        Expression { expr: Cow::Owned(expr), postfix: None, at_tokens: None, tilde_tokens: None }
    }

    pub fn new(expr: &'a TokenStream) -> Expression<'a> {
        Expression { expr: Cow::Borrowed(expr), postfix: None, at_tokens: None, tilde_tokens: None }
    }

    pub fn new_with_tilde(expr: &'a TokenStream, tilde_tokens: Box<Expression<'a>>) -> Expression<'a> {
        Expression { expr: Cow::Borrowed(expr), postfix: None, at_tokens: None, tilde_tokens: Some(tilde_tokens) }
    }
}

impl<'a> Render for Expression<'a> {
    fn render(&self, ctx: &RenderContext, stream: &mut TokenStream) {
        let expr = &self.expr;
        let ident = match ctx.kind {
            Kind::FromOwned | Kind::FromRef => quote!(value),
            _ => quote!(self),
        };

        let val = replace_tilde_or_at_in_expr(
            &expr,
            self.at_tokens.as_ref().map(|x| x.render_imm(ctx)).as_ref().or(Some(&ident)),
            self.tilde_tokens.as_ref().map(|x| x.render_imm(ctx)).as_ref());

        let postfix = &self.postfix;

        stream.extend(quote!(#val #postfix));
    }
}