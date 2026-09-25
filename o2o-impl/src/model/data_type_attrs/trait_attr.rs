use std::ops::Not;

use crate::{kw, model::*};

#[derive(Clone)]
pub(crate) struct TraitAttr {
    pub core: TraitAttrCore,
    pub fallible: bool,
    pub applicable_to: ApplicableTo,
}

#[derive(Clone)]
pub(crate) struct TraitAttrCore {
    pub ty: TypePath,
    pub err_ty: Option<TypePath>,
    pub type_hint: TypeHint,
    pub init_data: Option<Punctuated<InitData, Token![,]>>,
    pub update: Option<InlineExpressionWithSpan>,
    pub quick_return: Option<InlineExpressionWithSpan>,
    pub default_case: Option<InlineExpressionWithSpan>,
    pub match_expr: Option<InlineExpressionWithSpan>,
    pub repeat: Option<TraitRepeatFor>,
    pub skip_repeat: bool,
    pub stop_repeat: bool,
    pub attribute: Option<TokenStream>,
    pub impl_attribute: Option<TokenStream>,
    pub inner_attribute: Option<TokenStream>,
}

impl TraitAttrCore {
    pub fn merge(&mut self, other: Self) -> Result<()> {
        if self.skip_repeat {
            return Ok(());
        }

        if let Some(attr_to_repeat) = other.repeat {
            if attr_to_repeat[&TraitAttrType::Vars] {
                if self.init_data.is_some() {
                    Err(syn::Error::new(self.ty.span, "Vars will be overriden. Did you forget to use 'skip_repeat'?"))?
                }
                self.init_data = other.init_data
            }
            if attr_to_repeat[&TraitAttrType::Update] {
                if let Some(update) = &self.update {
                    Err(syn::Error::new(update.span, "Update instruction will be overriden. Did you forget to use 'skip_repeat'?"))?
                }
                self.update = other.update
            }
            if attr_to_repeat[&TraitAttrType::QuickReturn] {
                if let Some(quick_return) = &self.quick_return {
                    Err(syn::Error::new(quick_return.span, "Quick Return instruction will be overriden. Did you forget to use 'skip_repeat'?"))?
                }
                self.quick_return = other.quick_return
            }
            if attr_to_repeat[&TraitAttrType::DefaultCase] {
                if let Some(default_case) = &self.default_case {
                    Err(syn::Error::new(default_case.span, "Default Case instruction will be overriden. Did you forget to use 'skip_repeat'?"))?
                }
                self.default_case = other.default_case
            }
            if attr_to_repeat[&TraitAttrType::MatchExpr] {
                if let Some(match_expr) = &self.match_expr {
                    Err(syn::Error::new(match_expr.span, "Match instruction will be overriden. Did you forget to use 'skip_repeat'?"))?
                }
                self.match_expr = other.match_expr
            }
        }
        Ok(())
    }
}

impl Parse for TraitAttrCore {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty: TypePath = if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let content_stream = content.parse::<TokenStream>()?;
            quote!((#content_stream)).into()
        } else { input.parse::<syn::Path>()?.into() };
        let type_hint = if ty.nameless_tuple { TypeHint::Tuple } else { input.parse::<TypeHint>()? };
        let err_ty = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(if input.peek(Paren) {
                let content;
                parenthesized!(content in input);
                let content_stream = content.parse::<TokenStream>()?;
                quote!((#content_stream)).into()
            } else { input.parse::<syn::Path>()?.into() })
        } else { None };

        let mut attr = TraitAttrCore { ty, err_ty, type_hint, init_data: None, update: None, quick_return: None, default_case: None, match_expr: None, repeat: None, skip_repeat: false, stop_repeat: false, attribute: None, impl_attribute: None, inner_attribute: None };

        if !input.peek(Token![|]) {
            return Ok(attr);
        }

        input.parse::<Token![|]>()?;

        while parse_trait_instruction_param(input, &mut attr)? {}

        Ok(attr)
    }
}

#[derive(Clone)]
pub(crate) struct InitData {
    pub ident: Ident,
    _colon: Token![:],
    pub action: InlineExpression,
}

impl Parse for InitData {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(InitData {
            ident: input.parse()?,
            _colon: input.parse()?,
            action: input.parse()?,
        })
    }
}

fn parse_trait_instruction_param(input: &syn::parse::ParseBuffer, attr: &mut TraitAttrCore) -> Result<bool> {
    if input.peek(kw::stop_repeat) {
        return parse_trait_instruction_param_inner::<kw::stop_repeat, ()>(input, |_, _| Ok(()), attr.stop_repeat, |_| attr.stop_repeat = true, |a| a.span, "stop_repeat")
    } else if input.peek(kw::skip_repeat) {
        return parse_trait_instruction_param_inner::<kw::skip_repeat, ()>(input, |_, _| Ok(()), attr.skip_repeat, |_| attr.skip_repeat = true, |a| a.span, "skip_repeat")
    } else if input.peek(kw::repeat) {
        return parse_parenthesized_trait_instruction_param_inner::<kw::repeat, TraitRepeatForWrap>(input, |c| c.parse(), attr.repeat.is_some(), |x| attr.repeat = Some(x.0), |a| a.span, "repeat")
    } else if input.peek(kw::vars) {
        return parse_parenthesized_trait_instruction_param_inner::<kw::vars, Punctuated<InitData, Comma>>(input, |c| Punctuated::parse_separated_nonempty(&c), attr.init_data.is_some(), |x| attr.init_data = Some(x), |a| a.span, "vars")
    } else if input.peek(Token![..]) {
        return parse_trait_instruction_param_inner::<Token![..], Option<InlineExpressionWithSpan>>(input, |x, t| x.is_empty().not().then(|| x.parse()).map(|x| x.map(|x| InlineExpressionWithSpan::new(x, t.span()))).transpose(), attr.update.is_some(), |x| attr.update = x, |a| a.span(), "update")
    } else if input.peek(Token![return]) {
        return parse_trait_instruction_param_inner::<Token![return], Option<InlineExpressionWithSpan>>(input, |x, t| x.is_empty().not().then(|| x.parse()).map(|x| x.map(|x| InlineExpressionWithSpan::new(x, t.span))).transpose(), attr.quick_return.is_some(), |x| attr.quick_return = x, |a| a.span(), "quick_return")
    } else if input.peek(Token![_]) {
        return parse_trait_instruction_param_inner::<Token![_], Option<InlineExpressionWithSpan>>(input, |x, t| x.is_empty().not().then(|| x.parse()).map(|x| x.map(|x| InlineExpressionWithSpan::new(x, t.span))).transpose(), attr.default_case.is_some(), |x| attr.default_case = x, |a| a.span(), "default_case")
    } else if input.peek(Token![match]) {
        return parse_trait_instruction_param_inner::<Token![match], Option<InlineExpressionWithSpan>>(input, |x, t| x.is_empty().not().then(|| x.parse()).map(|x| x.map(|x| InlineExpressionWithSpan::new(x, t.span))).transpose(), attr.match_expr.is_some(), |x| attr.match_expr = x, |a| a.span(), "match_expr")
    } else if input.peek(kw::attribute) {
        return parse_parenthesized_trait_instruction_param_inner::<kw::attribute, TokenStream>(input, |c| c.parse(), attr.attribute.is_some(), |x| attr.attribute = Some(quote!(#[ #x ])), |a| a.span, "attribute")
    } else if input.peek(kw::impl_attribute) {
        return parse_parenthesized_trait_instruction_param_inner::<kw::impl_attribute, TokenStream>(input, |c| c.parse(), attr.impl_attribute.is_some(), |x| attr.impl_attribute = Some(quote!(#[ #x ])), |a| a.span, "impl_attribute")
    } else if input.peek(kw::inner_attribute) {
        return parse_parenthesized_trait_instruction_param_inner::<kw::inner_attribute, TokenStream>(input, |c| c.parse(), attr.inner_attribute.is_some(), |x| attr.inner_attribute = Some(quote!(#![ #x ])), |a| a.span, "inner_attribute")
    }

    Ok(true)
}

fn parse_trait_instruction_param_inner<T: Parse, U>(input: &syn::parse::ParseBuffer, parser: impl Fn(&ParseBuffer, &T) -> Result<U>, condition: bool, setter: impl FnOnce(U), span: impl Fn(T) -> Span, name: &str) -> Result<bool> {
    let a = input.parse::<T>()?;
    let b = parser(input, &a)?;
    if condition {
        Err(syn::Error::new(span(a), format!("Instruction parameter '{}' was already set.", name)))?
    } else {
        setter(b);
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Ok(true)
        } else { Ok(false) }
    }
}

fn parse_parenthesized_trait_instruction_param_inner<T: Parse, U>(input: &syn::parse::ParseBuffer, parser: impl Fn(ParseBuffer) -> Result<U>, condition: bool, setter: impl FnOnce(U), span: impl Fn(T) -> Span, name: &str) -> Result<bool> {
    let a = input.parse::<T>()?;
    let content;
    parenthesized!(content in input);
    let content = parser(content)?;
    if condition {
        Err(syn::Error::new(span(a), format!("Instruction parameter '{}' was already set.", name)))?
    } else {
        setter(content);
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Ok(true)
        } else { Ok(false) }
    }
}