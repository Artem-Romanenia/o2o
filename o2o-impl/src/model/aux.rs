mod parse;
mod applicable_to;

pub(crate) use parse::*;
pub(crate) use applicable_to::*;

use std::{fmt::Display, ops::Index};
use std::hash::Hash;

use crate::{kw, model::*};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Kind {
    OwnedInto,
    RefInto,
    FromOwned,
    FromRef,
    OwnedIntoExisting,
    RefIntoExisting,
}

impl Kind {
    pub fn is_ref(self) -> bool {
        self == Kind::FromRef || self == Kind::RefInto || self == Kind::RefIntoExisting
    }
    pub fn is_from(self) -> bool {
        self == Kind::FromOwned || self == Kind::FromRef
    }
    pub fn is_into_existing(self) -> bool {
        self == Kind::OwnedIntoExisting || self == Kind::RefIntoExisting
    }
}

pub(crate) struct FallibleKind(pub Kind, pub bool);

impl Display for FallibleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FallibleKind(Kind::OwnedInto, false) => f.write_str("owned_into"),
            FallibleKind(Kind::RefInto, false) => f.write_str("ref_into"),
            FallibleKind(Kind::FromOwned, false) => f.write_str("from_owned"),
            FallibleKind(Kind::FromRef, false) => f.write_str("from_ref"),
            FallibleKind(Kind::OwnedIntoExisting, false) => f.write_str("owned_into_existing"),
            FallibleKind(Kind::RefIntoExisting, false) => f.write_str("ref_into_existing"),
            FallibleKind(Kind::OwnedInto, true) => f.write_str("owned_try_into"),
            FallibleKind(Kind::RefInto, true) => f.write_str("ref_try_into"),
            FallibleKind(Kind::FromOwned, true) => f.write_str("try_from_owned"),
            FallibleKind(Kind::FromRef, true) => f.write_str("try_from_ref"),
            FallibleKind(Kind::OwnedIntoExisting, true) => f.write_str("owned_try_into_existing"),
            FallibleKind(Kind::RefIntoExisting, true) => f.write_str("ref_try_into_existing"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ChildPath {
    pub child_path: Punctuated<Member, Token![.]>,
    pub child_path_str: Vec<String>,
}

impl ChildPath {
    pub(crate) fn get_child_path_str(&self, depth: Option<usize>) -> &str {
        match depth {
            None => self.child_path_str.last().map(|x| x.as_str()).unwrap_or(""),
            Some(depth) => &self.child_path_str[depth],
        }
    }

    pub(crate) fn new<I: Iterator<Item = Member>>(root: Member, sub_path: I) -> ChildPath {
        let mut child_path = Punctuated::new();
        child_path.push(root);
        sub_path.for_each(|x|child_path.push(x));

        let child_path_str = build_child_path_str(&child_path);
        ChildPath { child_path, child_path_str }
    }
}

#[derive(Clone)]
pub(crate) struct TypePath {
    pub span: Span,
    pub path: TokenStream,
    pub path_str: String,
    pub generics: Option<AngleBracketedGenericArguments>,
    pub nameless_tuple: bool,
}

impl From<syn::Path> for TypePath {
    fn from(value: syn::Path) -> Self {
        let (path, generics) = if let PathArguments::AngleBracketed(g) = &value.segments.last().unwrap().arguments {
            let mut cl = value.clone();
            cl.segments.last_mut().unwrap().arguments = PathArguments::None;
            (cl.to_token_stream(), Some(g.clone()))
        } else {
            (value.to_token_stream(), None)
        };

        TypePath {
            span: value.span(),
            path,
            path_str: value.to_token_stream().to_string(),
            generics,
            nameless_tuple: false,
        }
    }
}

impl From<TokenStream> for TypePath {
    fn from(value: TokenStream) -> Self {
        TypePath {
            span: value.span(),
            path_str: value.to_string(),
            path: value,
            generics: None,
            nameless_tuple: true,
        }
    }
}

impl PartialEq for TypePath {
    fn eq(&self, other: &Self) -> bool {
        self.path_str == other.path_str
    }
}
impl Eq for TypePath {}
impl Hash for TypePath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path_str.hash(state)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum TypeHint {
    Unit = 0,
    Struct = 1,
    Tuple = 2,
    Unspecified = 3,
}

impl TypeHint {
    pub fn maybe(self, maybe: Self) -> bool {
        self == maybe || self == TypeHint::Unspecified
    }
}

impl Parse for TypeHint {
    fn parse(input: ParseStream) -> Result<Self> {
        if !input.peek(Token![as]) {
        return Ok(TypeHint::Unspecified);
    }

    input.parse::<Token![as]>()?;

    let mut _content;
    if input.peek(Brace) {
        braced!(_content in input);
        return Ok(TypeHint::Struct);
    }

    if input.peek(Paren) {
        parenthesized!(_content in input);
        return Ok(TypeHint::Tuple);
    }

    if input.peek(kw::Unit) {
        input.parse::<kw::Unit>()?;
        return Ok(TypeHint::Unit);
    }

    Err(input.error("Only '()', '{}', and 'Unit' are supported type hints."))
    }
}

pub (super) enum TraitAttrType {
    Vars,
    Update,
    QuickReturn,
    DefaultCase,
    MatchExpr
}

pub(super) type TraitRepeatFor = [bool; 5];

impl Index<&TraitAttrType> for TraitRepeatFor {
    type Output = bool;

    fn index(&self, index: &TraitAttrType) -> &Self::Output {
        match index {
            TraitAttrType::Vars => &self[0],
            TraitAttrType::Update => &self[1],
            TraitAttrType::QuickReturn => &self[2],
            TraitAttrType::DefaultCase => &self[3],
            TraitAttrType::MatchExpr => &self[4]
        }
    }
}

pub(crate) struct TraitRepeatForWrap(pub TraitRepeatFor);

impl Parse for TraitRepeatForWrap {
    fn parse(input: ParseStream) -> Result<Self> {
        let types: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(input)?;
        if types.is_empty() {
            return Ok(TraitRepeatForWrap([true, true, true, true, true]));
        }

        let mut repeat: TraitRepeatFor = [false, false, false, false, false];

        for ty in types {
            let str = ty.to_token_stream().to_string();

            match TRAIT_REPEAT_TYPES.iter().position(|x| *x == str.as_str()) {
                Some(idx) => repeat[idx] = true,
                None => return Err(Error::new(ty.span(), format!("#[repeat] of instruction type '{}' is not supported. Supported types are: {}", str, TRAIT_REPEAT_TYPES.join(", ")))),
            };
        }

        Ok(TraitRepeatForWrap(repeat))
    }
}

const TRAIT_REPEAT_TYPES: [&str; 5] = ["vars", "update", "quick_return", "default_case", "match_expr"];

pub(super) enum MemberAttrType {
    Attr,
    Child,
    Parent,
    Ghost,
    TypeHint,
}

pub(super) type MemberRepeatFor = [bool; 5];

impl Index<&MemberAttrType> for MemberRepeatFor {
    type Output = bool;

    fn index(&self, index: &MemberAttrType) -> &Self::Output {
        match index {
            MemberAttrType::Attr => &self[0],
            MemberAttrType::Child => &self[1],
            MemberAttrType::Parent => &self[2],
            MemberAttrType::Ghost => &self[3],
            MemberAttrType::TypeHint => &self[4],
        }
    }
}

pub(super) const MEMBER_REPEAT_TYPES: [&str; 5] = ["map", "child", "parent", "ghost", "type_hint"];

#[derive(Clone)]
pub(crate) struct InlineExpressionWithSpan {
    pub expr: InlineExpression,
    pub span: Span
}

impl InlineExpressionWithSpan {
    pub fn new (expr: InlineExpression, span: Span) -> InlineExpressionWithSpan {
        InlineExpressionWithSpan { expr, span }
    }
}

#[derive(Clone)]
pub(crate) struct InlineExpression {
    pub expr: TokenStream
}

impl Parse for InlineExpression {
    fn parse(input: ParseStream) -> Result<Self> {
        if !input.peek(Brace) {
            let f: CommaTerminatedTokenStream = input.parse()?;
            Ok(InlineExpression { expr: f.token_stream })
        } else {
            let content;
            braced!(content in input);
            return Ok(InlineExpression { expr: content.parse()? });
        }
    }
}

struct CommaTerminatedTokenStream {
    token_stream: TokenStream
}

impl Parse for CommaTerminatedTokenStream {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut tokens: Vec<TokenTree> = vec![];

        loop {
            if input.peek(Token![,]) || input.is_empty() { break; }
            tokens.push(input.parse()?);
        }

        Ok(CommaTerminatedTokenStream { token_stream: TokenStream::from_iter(tokens) })
    }
}

pub(super) struct OptionalParenthesizedTokenStream {
    content: Option<TokenStream>,
}

impl Parse for OptionalParenthesizedTokenStream {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(OptionalParenthesizedTokenStream {
            content: if input.peek(Paren) {
                let content;
                parenthesized!(content in input);
                Some(content.parse()?)
            } else { None },
        })
    }
}

impl OptionalParenthesizedTokenStream {
    pub(super) fn content(self) -> TokenStream {
        match self.content {
            Some(content) => content,
            None => TokenStream::new(),
        }
    }
}

pub(crate) fn build_child_path_str(child_path: &Punctuated<Member, Token![.]>) -> Vec<String> {
    let mut child_path_str = vec![];
    child_path.iter().for_each(|x: &Member| {
        if child_path_str.is_empty() {
            child_path_str.push(x.to_token_stream().to_string())
        } else {
            child_path_str.push(format!("{}.{}", child_path_str.last().map(|x| x.as_str()).unwrap_or(""), x.to_token_stream()))
        }
    });
    child_path_str
}