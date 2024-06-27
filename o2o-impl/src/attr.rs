use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::ops::Index;

use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::parse::{ParseStream, Parse};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Brace, Paren};
use syn::{Attribute, Ident, Result, Token, Member, parenthesized, braced, WherePredicate, Error};

use crate::ast::SynDataTypeMember;
use crate::kw;

struct OptionalParenthesizedTokenStream {
    content: Option<TokenStream>
}

impl Parse for OptionalParenthesizedTokenStream{
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(OptionalParenthesizedTokenStream{
            content: if input.peek(Paren) {
                let content;
                parenthesized!(content in input);
                Some(content.parse()?)
            } else { None }
        })
    }
}

impl OptionalParenthesizedTokenStream {
    fn content(self) -> TokenStream {
        match self.content {
            Some(content) => content,
            None => TokenStream::new()
        }
    }
}

pub(crate) enum DataTypeInstruction {
    Map(TraitAttr),
    Ghosts(GhostsAttr), 
    Where(WhereAttr),
    Children(ChildrenAttr),
    AllowUnknown,

    Misplaced { instr: &'static str, span: Span, own: bool },
    Misnamed { instr: &'static str, span: Span, guess_name: &'static str, own: bool },
    UnrecognizedWithError { instr: String, span: Span },
    Unrecognized,
}

#[derive(Clone)]
pub(crate) enum MemberInstruction {
    Map(MemberAttr),
    Ghost(GhostAttr),
    Ghosts(GhostsAttr),
    Child(ChildAttr),
    Parent(ParentAttr),
    As(AsAttr),
    Lit(LitAttr),
    Pat(PatAttr),
    VariantTypeHint(VariantTypeHintAttr),
    Repeat(MemberRepeatAttr),
    StopRepeat,

    Misplaced { instr: &'static str, span: Span, own: bool },
    Misnamed { instr: &'static str, span: Span, guess_name: &'static str, own: bool },
    UnrecognizedWithError { instr: String, span: Span },
    Unrecognized,
}

#[derive(Clone)]
pub(crate) struct TypePath {
    pub span: Span,
    pub path: TokenStream,
    pub path_str: String,
    pub nameless_tuple: bool,
}

impl From<syn::Path> for TypePath {
    fn from(value: syn::Path) -> Self {
        TypePath {
            span: value.span(),
            path: value.to_token_stream(),
            path_str: value.to_token_stream().to_string(),
            nameless_tuple: false
        }
    }
}

impl From<TokenStream> for TypePath {
    fn from(value: TokenStream) -> Self {
        TypePath {
            span: value.span(),
            path_str: value.to_string(),
            path: value,
            nameless_tuple: true
        }
    }
}

impl PartialEq for TypePath {
    fn eq(&self, other: &Self) -> bool {
        self.path_str == other.path_str
    }
}
impl Eq for TypePath{}
impl Hash for TypePath {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path_str.hash(state)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Kind {
    OwnedInto,
    RefInto,
    FromOwned,
    FromRef,
    OwnedIntoExisting,
    RefIntoExisting
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

type ApplicableTo = [bool; 6];

impl Index<&Kind> for ApplicableTo {
    type Output = bool;

    fn index(&self, index: &Kind) -> &Self::Output {
        match index {
            Kind::OwnedInto => &self[0],
            Kind::RefInto => &self[1],
            Kind::FromOwned => &self[2],
            Kind::FromRef => &self[3],
            Kind::OwnedIntoExisting => &self[4],
            Kind::RefIntoExisting => &self[5],
        }
    }
}

#[derive(Default)]
pub(crate) struct DataTypeAttrs {
    pub attrs: Vec<TraitAttr>,
    pub ghosts_attrs: Vec<GhostsAttr>,
    pub where_attrs: Vec<WhereAttr>,
    pub children_attrs: Vec<ChildrenAttr>,

    pub error_instrs: Vec<DataTypeInstruction>
}

impl<'a> DataTypeAttrs {
    pub(crate) fn iter_for_kind(&'a self, kind: &'a Kind, fallible: bool) -> impl Iterator<Item = &TraitAttr> {
        self.attrs.iter().filter(move |x| x.fallible == fallible && x.applicable_to[kind])
    }

    pub(crate) fn iter_for_kind_core(&'a self, kind: &'a Kind, fallible: bool) -> impl Iterator<Item = &TraitAttrCore> {
        self.iter_for_kind(kind, fallible).map(|x| &x.core)
    }

    pub(crate) fn ghost_attr(&'a self, container_ty: &'a TypePath, kind: &'a Kind) -> Option<&StructGhostAttrCore> {
        self.ghosts_attrs.iter()
            .find(|x| x.applicable_to[kind] && x.attr.container_ty.is_some() && x.attr.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.ghosts_attrs.iter().find(|x| x.applicable_to[kind] && x.attr.container_ty.is_none())).map(|x| &x.attr)
    }

    pub(crate) fn where_attr(&'a self, container_ty: &TypePath) -> Option<&WhereAttr>{
        self.where_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.where_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn children_attr(&'a self, container_ty: &TypePath) -> Option<&ChildrenAttr>{
        self.children_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.children_attrs.iter().find(|x| x.container_ty.is_none()))
    }
}

type MemberRepeatFor = [bool; 5];

enum MemberAttrType {
    Attr,
    Child,
    Parent,
    Ghost,
    TypeHint
}

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

#[derive(Clone)]
pub(crate) struct MemberRepeatAttr {
    pub permeate: bool,
    pub repeat_for: MemberRepeatFor,
}

const MEMBER_REPEAT_TYPES: [&str; 5] = ["map", "child", "parent", "ghost", "type_hint"];

impl Parse for MemberRepeatAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let permeate = input.peek(kw::permeate);
        if permeate {
            input.parse::<kw::permeate>()?;
            let _content;
            parenthesized!(_content in input);
        }

        if permeate && !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        let types: Punctuated<Ident, Token![,]>  = Punctuated::parse_terminated(input)?;
        if types.is_empty() {
            return Ok(MemberRepeatAttr{
                permeate,
                repeat_for: [true,  true, true, true, true]
            })
        }

        let mut repeat_for: MemberRepeatFor = [false, false, false, false, false];

        for ty in types {
            let str = ty.to_token_stream().to_string();

            match MEMBER_REPEAT_TYPES.iter().position(|x|*x == str.as_str()) {
                Some(idx) => repeat_for[idx] = true,
                None => return Err(Error::new(ty.span(), format!("#[repeat] of instruction type '{}' is not supported. Supported types are: {}", str, MEMBER_REPEAT_TYPES.join(", "))))
            };
        }

        Ok(MemberRepeatAttr { permeate, repeat_for })
    }
}

#[derive(Clone, Default)]
pub(crate) struct MemberAttrs {
    pub attrs: Vec<MemberAttr>,
    pub child_attrs: Vec<ChildAttr>,
    pub parent_attrs: Vec<ParentAttr>,
    pub ghost_attrs: Vec<GhostAttr>,
    pub ghosts_attrs: Vec<GhostsAttr>,
    pub lit_attrs: Vec<LitAttr>,
    pub pat_attrs: Vec<PatAttr>,
    pub repeat: Option<MemberRepeatAttr>,
    pub stop_repeat: bool,
    pub type_hint_attrs: Vec<VariantTypeHintAttr>,

    pub error_instrs: Vec<MemberInstruction>
}

impl<'a> MemberAttrs {
    pub(crate) fn iter_for_kind(&'a self, kind: &'a Kind, fallible: bool) -> impl Iterator<Item = &MemberAttr> {
        self.attrs.iter().filter(move |x| x.fallible == fallible && x.applicable_to[kind])
    }

    pub(crate) fn iter_for_kind_core(&'a self, kind: &'a Kind, fallible: bool) -> impl Iterator<Item = &MemberAttrCore> {
        self.iter_for_kind(kind, fallible).map(|x| &x.attr)
    }

    pub(crate) fn applicable_attr(&'a self, kind: &'a Kind, fallible: bool, container_ty: &TypePath) -> Option<ApplicableAttr> {
        self.ghost(container_ty, kind)
            .map(ApplicableAttr::Ghost)
            .or_else(|| self.field_attr_core(kind, fallible, container_ty)
                .or_else(|| if fallible { self.field_attr_core(kind, false, container_ty) } else { None })
                .or_else(|| if kind == &Kind::OwnedIntoExisting { self.field_attr_core(&Kind::OwnedInto, fallible, container_ty) } else { None })
                .or_else(|| if kind == &Kind::OwnedIntoExisting && fallible { self.field_attr_core(&Kind::OwnedInto, false, container_ty) } else { None })
                .or_else(|| if kind == &Kind::RefIntoExisting { self.field_attr_core(&Kind::RefInto, fallible, container_ty) } else { None })
                .or_else(|| if kind == &Kind::RefIntoExisting && fallible { self.field_attr_core(&Kind::RefInto, false, container_ty) } else { None })
                .map(ApplicableAttr::Field))
    }

    pub(crate) fn applicable_field_attr(&'a self, kind: &'a Kind, fallible: bool, container_ty: &TypePath) -> Option<&'a MemberAttr> {
        self.field_attr(kind, fallible, container_ty)
            .or_else(|| if kind == &Kind::OwnedIntoExisting { self.field_attr(&Kind::OwnedInto, fallible, container_ty) } else { None })
            .or_else(|| if kind == &Kind::RefIntoExisting { self.field_attr(&Kind::RefInto, fallible, container_ty) } else { None })
    }

    pub(crate) fn child(&'a self, container_ty: &TypePath) -> Option<&ChildAttr>{
        self.child_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.child_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn ghost(&'a self, container_ty: &TypePath, kind: &'a Kind) -> Option<&FieldGhostAttrCore>{
        self.ghost_attrs.iter()
            .find(|x| x.applicable_to[kind] && x.attr.container_ty.is_some() && x.attr.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.ghost_attrs.iter().find(|x| x.applicable_to[kind] && x.attr.container_ty.is_none())).map(|x| &x.attr)
    }

    pub(crate) fn lit(&'a self, container_ty: &TypePath) -> Option<&LitAttr>{
        self.lit_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.lit_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn pat(&'a self, container_ty: &TypePath) -> Option<&PatAttr>{
        self.pat_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.pat_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn type_hint(&'a self, container_ty: &TypePath) -> Option<&VariantTypeHintAttr>{
        self.type_hint_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.type_hint_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn has_parent_attr(&'a self, container_ty: &TypePath) -> bool {
        self.parent_attrs.iter()
            .any(|x| x.container_ty.is_none() || x.container_ty.as_ref().unwrap() == container_ty)
    }

    pub(crate) fn field_attr(&'a self, kind: &'a Kind, fallible: bool, container_ty: &TypePath) -> Option<&MemberAttr>{
        self.iter_for_kind(kind, fallible)
            .find(|x| x.attr.container_ty.is_some() && x.attr.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.iter_for_kind(kind, fallible).find(|x| x.attr.container_ty.is_none()))
    }

    pub(crate) fn field_attr_core(&'a self, kind: &'a Kind, fallible: bool, container_ty: &TypePath) -> Option<&MemberAttrCore>{
        self.iter_for_kind_core(kind, fallible)
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.iter_for_kind_core(kind, fallible).find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn merge(&'a mut self, other: Self) {
        if let Some(repeat) =  other.repeat  {
            if repeat.repeat_for[&MemberAttrType::Attr] {
                self.attrs.extend(other.attrs);
            }
            if repeat.repeat_for[&MemberAttrType::Child] {
                self.child_attrs.extend(other.child_attrs);
            }
            if repeat.repeat_for[&MemberAttrType::Parent] {
                self.parent_attrs.extend(other.parent_attrs);
            }
            if repeat.repeat_for[&MemberAttrType::Ghost] {
                self.ghost_attrs.extend(other.ghost_attrs);
            }
            if repeat.repeat_for[&MemberAttrType::TypeHint] {
                self.type_hint_attrs.extend(other.type_hint_attrs);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum TypeHint {
    Unit = 0,
    Struct = 1,
    Tuple = 2,
    Unspecified = 3
}

impl TypeHint {
    pub fn maybe(self, maybe: Self) -> bool {
        self == maybe || self == TypeHint::Unspecified
    }
}

type TraitRepeatFor = [bool; 4];
struct TraitRepeatForWrap(TraitRepeatFor);

enum TraitAttrType {
    Vars,
    Update,
    QuickReturn,
    DefaultCase
}

impl Index<&TraitAttrType> for TraitRepeatFor {
    type Output = bool;

    fn index(&self, index: &TraitAttrType) -> &Self::Output {
        match index {
            TraitAttrType::Vars => &self[0],
            TraitAttrType::Update => &self[1],
            TraitAttrType::QuickReturn => &self[2],
            TraitAttrType::DefaultCase => &self[3],
        }
    }
}

const TRAIT_REPEAT_TYPES: [&str; 4] = ["vars", "update", "quick_return", "default_case"];

impl Parse for TraitRepeatForWrap {
    fn parse(input: ParseStream) -> Result<Self> {
        let types: Punctuated<Ident, Token![,]>  = Punctuated::parse_terminated(input)?;
        if types.is_empty() {
            return Ok(TraitRepeatForWrap([true, true, true, true]))
        }

        let mut repeat: TraitRepeatFor = [false, false, false, false];

        for ty in types {
            let str = ty.to_token_stream().to_string();
            
            match TRAIT_REPEAT_TYPES.iter().position(|x|*x == str.as_str()) {
                Some(idx) => repeat[idx] = true,
                None => return Err(Error::new(ty.span(), format!("#[repeat] of instruction type '{}' is not supported. Supported types are: {}", str, TRAIT_REPEAT_TYPES.join(", "))))
            };
        }

        Ok(TraitRepeatForWrap(repeat))
    }
}

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
    pub update: Option<TokenStream>,
    pub quick_return: Option<TokenStream>,
    pub default_case: Option<TokenStream>,
    pub repeat: Option<TraitRepeatFor>,
    pub skip_repeat: bool,
    pub stop_repeat: bool,
    pub attribute: Option<TokenStream>,
    pub impl_attribute: Option<TokenStream>,
    pub inner_attribute: Option<TokenStream>,
}

impl TraitAttrCore {
    fn merge(&mut self, other: Self) -> Result<()> {
        if self.skip_repeat {
            return Ok(())
        }

        if let Some(attr_to_repeat) = other.repeat {
            if attr_to_repeat[&TraitAttrType::Vars] {
                if self.init_data.is_some() {
                    Err(syn::Error::new(self.ty.span, "Vars will be overriden. Did you forget to use 'skip_repeat'?"))?
                }
                self.init_data = other.init_data
            }
            if attr_to_repeat[&TraitAttrType::Update] {
                if self.update.is_some() {
                    Err(syn::Error::new(self.update.span(), "Update statement will be overriden. Did you forget to use 'skip_repeat'?"))?
                }
                self.update = other.update
            }
            if attr_to_repeat[&TraitAttrType::QuickReturn] {
                if self.quick_return.is_some() {
                    Err(syn::Error::new(self.quick_return.span(), "Quick Return statement will be overriden. Did you forget to use 'skip_repeat'?"))?
                }
                self.quick_return = other.quick_return
            }
            if attr_to_repeat[&TraitAttrType::DefaultCase] {
                if self.default_case.is_some() {
                    Err(syn::Error::new(self.default_case.span(), "Default Case statement will be overriden. Did you forget to use 'skip_repeat'?"))?
                }
                self.default_case = other.default_case
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
        let type_hint = if ty.nameless_tuple { TypeHint::Tuple } else { try_parse_type_hint(input)? };
        let err_ty = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse::<syn::Path>()?.into())
        } else {
            None
        };

        let mut attr = TraitAttrCore { ty, err_ty, type_hint, init_data: None, update: None, quick_return: None, default_case: None, repeat: None, skip_repeat: false, stop_repeat: false, attribute: None, impl_attribute: None, inner_attribute: None };

        if !input.peek(Token![|]){
            return Ok(attr)
        }

        input.parse::<Token![|]>()?;

        while parse_trait_instruction_param(input, &mut attr)? {}

        Ok(attr)
    }
}

fn parse_trait_instruction_param(input: &syn::parse::ParseBuffer, attr: &mut TraitAttrCore) -> Result<bool> {
    if input.peek(kw::stop_repeat) {
        let a = input.parse::<kw::stop_repeat>()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if attr.stop_repeat {
            Err(syn::Error::new(a.span, "Instruction parameter 'stop_repeat' was already set."))?
        } else {
            attr.stop_repeat = true;
            return Ok(true)
        }
    } else if input.peek(kw::skip_repeat) {
        let a = input.parse::<kw::skip_repeat>()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if attr.skip_repeat {
            Err(syn::Error::new(a.span, "Instruction parameter 'skip_repeat' was already set."))?
        } else {
            attr.skip_repeat = true;
            return Ok(true)
        }
    } else if input.peek(kw::repeat) {
        let a = input.parse::<kw::repeat>()?;
        let content;
        parenthesized!(content in input);
        let repeat  = content.parse::<TraitRepeatForWrap>()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if attr.repeat.is_some() {
            Err(syn::Error::new(a.span, "Instruction parameter 'repeat' was already set."))?
        } else {
            attr.repeat = Some(repeat.0);
            return Ok(true)
        }
    } else if input.peek(kw::vars) {
        let a = input.parse::<kw::vars>()?;
        let content;
        parenthesized!(content in input);
        let vars = Some(Punctuated::parse_separated_nonempty(&content)?);
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if attr.init_data.is_some() {
            Err(syn::Error::new(a.span, "Instruction parameter 'vars' was already set."))?
        } else {
            attr.init_data = vars;
            return Ok(true)
        }
    } else if input.peek(Token![..]) {
        input.parse::<Token![..]>()?;
        attr.update = try_parse_action(input, true)?;
        return Ok(false)
    } else if input.peek(Token![return]) {
        input.parse::<Token![return]>()?;
        attr.quick_return = try_parse_action(input, true)?;
        return Ok(false)
    } else if input.peek(Token![_]) {
        input.parse::<Token![_]>()?;
        attr.default_case = try_parse_action(input, true)?;
        return Ok(false)
    } else if input.peek(kw::attribute) {
        let a = input.parse::<kw::attribute>()?;
        let content;
        parenthesized!(content in input);
        let attribute  = content.parse::<TokenStream>()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if attr.attribute.is_some() {
            Err(syn::Error::new(a.span, "Instruction parameter 'attribute' was already set."))?
        } else {
            attr.attribute = Some(quote!(#[ #attribute ]));
            return Ok(true)
        }
    }
    else if input.peek(kw::impl_attribute) {
        let a = input.parse::<kw::impl_attribute>()?;
        let content;
        parenthesized!(content in input);
        let attribute  = content.parse::<TokenStream>()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if attr.impl_attribute.is_some() {
            Err(syn::Error::new(a.span, "Instruction parameter 'impl_attribute' was already set."))?
        } else {
            attr.impl_attribute = Some(quote!(#[ #attribute ]));
            return Ok(true)
        }
    }
    if input.peek(kw::inner_attribute) {
        let a = input.parse::<kw::inner_attribute>()?;
        let content;
        parenthesized!(content in input);
        let attribute  = content.parse::<TokenStream>()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if attr.inner_attribute.is_some() {
            Err(syn::Error::new(a.span, "Instruction parameter 'inner_attribute' was already set."))?
        } else {
            attr.inner_attribute = Some(quote!(#![ #attribute ]));
            return Ok(true)
        }
    }

    Ok(false)
}

#[derive(Clone)]
pub(crate) struct InitData {
    pub ident: Ident,
    _colon: Token![:],
    pub action: TokenStream,
}

impl Parse for InitData {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(InitData {
            ident: input.parse()?,
            _colon: input.parse()?,
            action: try_parse_braced_action(input)?
        })
    }
}

#[derive(Clone)]
pub(crate) struct GhostsAttr {
    pub attr: StructGhostAttrCore,
    pub applicable_to: ApplicableTo,
}

#[derive(Clone)]
pub(crate) struct StructGhostAttrCore {
    pub container_ty: Option<TypePath>,
    pub ghost_data: Punctuated<GhostData, Token![,]>,
}

impl Parse for StructGhostAttrCore{
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(StructGhostAttrCore { 
            container_ty: try_parse_container_ident(input, false),
            ghost_data: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

#[derive(Clone)]
pub(crate) struct GhostData {
    pub child_path: Option<ChildPath>,
    pub ghost_ident: Member,
    pub action: TokenStream,
}

impl GhostData {
    pub(crate) fn get_child_path_str(&self, depth: Option<usize>) -> &str {
        self.child_path.as_ref().map(|x| x.get_child_path_str(depth)).unwrap_or("")
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
}

impl Parse for GhostData {
    fn parse(input: ParseStream) -> Result<Self> {
        let child_path = if !peek_ghost_field_name(input) {
            let child_path = Some(Punctuated::parse_separated_nonempty(input)?).map(|child_path| {
                let child_path_str = build_child_path_str(&child_path);
                ChildPath { child_path, child_path_str }
            });
            input.parse::<Token![@]>()?;
            child_path
        } else { None };
        let ghost_ident = input.parse()?;
        input.parse::<Token![:]>()?;
        Ok(GhostData {
            child_path,
            ghost_ident,
            action: try_parse_braced_action(input)?
        })
    }
}

pub(crate) struct WhereAttr {
    pub container_ty: Option<TypePath>,
    pub where_clause: Punctuated<WherePredicate, Token![,]>,
}

impl Parse for WhereAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(WhereAttr { 
            container_ty: try_parse_container_ident(input, false),
            where_clause: Punctuated::parse_separated_nonempty(input)?
        })
    }
}

pub(crate) struct ChildrenAttr {
    pub container_ty: Option<TypePath>,
    pub children: Punctuated<ChildData, Token![,]>,
}

impl Parse for ChildrenAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ChildrenAttr { 
            container_ty: try_parse_container_ident(input, false),
            children: try_parse_children(input)?,
        })
    }
}

pub(crate) struct ChildData {
    pub ty: syn::Path,
    pub type_hint: TypeHint,
    pub field_path: Punctuated<Member, Token![.]>,
    field_path_str: String,
}

impl ChildData {
    pub(crate) fn check_match(&self, path: &str) -> bool {
        self.field_path_str == path
    }
}

impl PartialEq for ChildData {
    fn eq(&self, other: &Self) -> bool {
        self.field_path_str == other.field_path_str
    }
}
impl Eq for ChildData {}
impl Hash for ChildData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.field_path_str.hash(state)
    }
}

#[derive(Clone)]
pub(crate) struct MemberAttr  {
    pub attr: MemberAttrCore,
    pub fallible: bool,
    pub original_instr: String,
    applicable_to: ApplicableTo,
}

#[derive(Clone)]
pub(crate) struct MemberAttrCore {
    pub container_ty: Option<TypePath>,
    pub member: Option<Member>,
    pub action: Option<TokenStream>,
}

impl Parse for MemberAttrCore {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(MemberAttrCore {
            container_ty: try_parse_container_ident(input, false),
            member: try_parse_optional_ident(input),
            action: try_parse_action(input, true)?,
        })
    }
}

#[derive(Clone)]
pub(crate) struct ParentAttr {
    pub container_ty: Option<TypePath>,
}

impl Parse for ParentAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ParentAttr { 
            container_ty: try_parse_container_ident(input, true),
        })
    }
}

#[derive(Clone)]
pub(crate) struct GhostAttr {
    pub attr: FieldGhostAttrCore,
    pub applicable_to: ApplicableTo,
}

#[derive(Clone)]
pub(crate) struct FieldGhostAttrCore {
    pub container_ty: Option<TypePath>,
    pub action: Option<TokenStream>,
}

impl Parse for FieldGhostAttrCore {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FieldGhostAttrCore { 
            container_ty: try_parse_container_ident(input, true),
            action: try_parse_action(input, true)?,
        })
    }
}

pub(crate) enum ApplicableAttr<'a> {
    Field(&'a MemberAttrCore),
    Ghost(&'a FieldGhostAttrCore),
}

#[derive(Clone)]
pub(crate) struct ChildAttr {
    pub container_ty: Option<TypePath>,
    pub child_path: ChildPath,
}

impl ChildAttr {
    pub(crate) fn get_child_path_str(&self, depth: Option<usize>) -> &str {
        self.child_path.get_child_path_str(depth)
    }
}

impl Parse for ChildAttr{
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        let child_path: Punctuated<Member, Token![.]> = Punctuated::parse_separated_nonempty(input)?;
        let child_path_str = build_child_path_str(&child_path);
        Ok(ChildAttr { 
            container_ty,
            child_path: ChildPath { 
                child_path,
                child_path_str,
            }
        })
    }
}

#[derive(Clone)]
pub(crate) struct AsAttr {
    pub container_ty: Option<TypePath>,
    pub member: Option<Member>,
    pub tokens: TokenStream,
}

impl Parse for AsAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        let (ident, tokens) = if peek_member(input) && input.peek2(Token![,]) {
            let ident = Some(input.parse()?);
            input.parse::<Token![,]>()?;
            let tokens = input.parse()?;
            (ident, tokens)
        } else { (None, input.parse()?) };

        Ok(AsAttr { container_ty, member: ident, tokens })
    }
}

#[derive(Clone)]
pub(crate) struct LitAttr {
    pub container_ty: Option<TypePath>,
    pub tokens: TokenStream,
}

impl Parse for LitAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        Ok(LitAttr { container_ty, tokens: input.parse()? })
    }
}

#[derive(Clone)]
pub(crate) struct PatAttr {
    pub container_ty: Option<TypePath>,
    pub tokens: TokenStream,
}

impl Parse for PatAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        Ok(PatAttr { container_ty, tokens: input.parse()? })
    }
}

#[derive(Clone)]
pub(crate) struct VariantTypeHintAttr {
    pub container_ty: Option<TypePath>,
    pub type_hint: TypeHint
}

impl Parse for VariantTypeHintAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        let type_hint = try_parse_type_hint(input)?;
        Ok(VariantTypeHintAttr { container_ty, type_hint })
    }
}

pub(crate) fn get_data_type_attrs(input: &[Attribute]) -> Result<(DataTypeAttrs, bool)> {
    let mut bark = true;

    let mut instrs: Vec<DataTypeInstruction> = vec![];
    for x in input.iter() {
        if x.path.is_ident("doc"){
            continue;
        } else if x.path.is_ident("o2o") {
            x.parse_args_with(|input: ParseStream| {
                let new_instrs: Punctuated<DataTypeInstruction, Token![,]> = Punctuated::parse_terminated_with(input, |input| {
                    let instr = input.parse::<Ident>()?;
                    let p: OptionalParenthesizedTokenStream = input.parse()?;
                    parse_data_type_instruction(&instr, p.content(), true, true)
                })?;

                if new_instrs.iter().any(|x| matches!(x, DataTypeInstruction::AllowUnknown)) {
                    bark = false;
                }

                instrs.extend(new_instrs.into_iter());
                Ok(())
            })?;
        } else {
            let instr = x.path.get_ident().unwrap();
            let p: OptionalParenthesizedTokenStream = syn::parse2(x.tokens.clone())?;
            instrs.push(parse_data_type_instruction(instr, p.content(), false, bark)?);
        }
    }

    let mut attrs = DataTypeAttrs::default();

    let mut trait_attrs_to_repeat = HashMap::<(ApplicableTo, bool), TraitAttr>::new();
    
    for instr in  instrs {
        match instr {
            DataTypeInstruction::Map(mut trait_attr) => {
                let k = (trait_attr.applicable_to, trait_attr.fallible);

                if trait_attr.core.stop_repeat {
                    trait_attrs_to_repeat.remove(&k);
                }

                let trait_attr_to_repeat = trait_attrs_to_repeat.get_mut(&k);

                if trait_attr.core.repeat.is_some() {
                    if trait_attr_to_repeat.is_some() && !trait_attr.core.stop_repeat {
                        Err(syn::Error::new(trait_attr.core.ty.span, "Previous repeat() instruction must be terminated with 'stop_repeat'"))?
                    }

                    trait_attrs_to_repeat.insert(k, trait_attr.clone());
                } else if let Some(trait_attr_to_repeat) = &trait_attr_to_repeat {
                    trait_attr.core.merge(trait_attr_to_repeat.core.clone())?;
                }

                attrs.attrs.push(trait_attr)
            },
            DataTypeInstruction::Ghosts(attr) => attrs.ghosts_attrs.push(attr),
            DataTypeInstruction::Where(attr) => attrs.where_attrs.push(attr),
            DataTypeInstruction::Children(attr) => attrs.children_attrs.push(attr),
            DataTypeInstruction::AllowUnknown | DataTypeInstruction::Unrecognized => (),
            _ => attrs.error_instrs.push(instr)
        };
    }
    Ok((attrs, bark))
}

pub(crate) fn get_member_attrs(input: SynDataTypeMember, bark: bool) -> Result<MemberAttrs> {
    let mut instrs: Vec<MemberInstruction> = vec![];
    for x in input.get_attrs().iter() {
        if x.path.is_ident("doc"){
            continue;
        } else if x.path.is_ident("o2o") {
            x.parse_args_with(|input: ParseStream| {
                let new_instrs: Punctuated<MemberInstruction, Token![,]> = Punctuated::parse_terminated_with(input, |input| {
                    let instr = input.parse::<Ident>()?;
                    let p: OptionalParenthesizedTokenStream = input.parse()?;
                    parse_member_instruction(&instr, p.content(), true, true)
                })?;
                instrs.extend(new_instrs.into_iter());
                Ok(())
            })?;
        } else {
            let instr = x.path.get_ident().unwrap();
            let p: OptionalParenthesizedTokenStream = syn::parse2(x.tokens.clone())?;
            instrs.push(parse_member_instruction(instr, p.content(), false, bark)?);
        }
    }

    let mut attrs = MemberAttrs::default();

    for instr in  instrs {
        match instr {
            MemberInstruction::Map(attr) => attrs.attrs.push(attr),
            MemberInstruction::Child(attr) => attrs.child_attrs.push(attr),
            MemberInstruction::Ghost(attr) => attrs.ghost_attrs.push(attr),
            MemberInstruction::Ghosts(attr) => attrs.ghosts_attrs.push(attr),
            MemberInstruction::Parent(attr) => attrs.parent_attrs.push(attr),
            MemberInstruction::As(attr) => {
                match input {
                    SynDataTypeMember::Field(f) => add_as_type_attrs(f, attr, &mut attrs.attrs),
                    SynDataTypeMember::Variant(_) => unreachable!("1")
                };
            },
            MemberInstruction::Lit(attr) => attrs.lit_attrs.push(attr),
            MemberInstruction::Pat(attr) => attrs.pat_attrs.push(attr),
            MemberInstruction::Repeat(repeat_for) => attrs.repeat = Some(repeat_for),
            MemberInstruction::StopRepeat => attrs.stop_repeat = true,
            MemberInstruction::VariantTypeHint(attr) => attrs.type_hint_attrs.push(attr),
            MemberInstruction::Unrecognized => (),
            _ => attrs.error_instrs.push(instr)
        };
    }
    Ok(attrs)
}

fn parse_data_type_instruction(instr: &Ident, input: TokenStream, own_instr: bool, bark: bool) -> Result<DataTypeInstruction>
{
    let instr_str = &instr.to_token_stream().to_string();
    match instr_str.as_ref() {
        "allow_unknown" if own_instr => Ok(DataTypeInstruction::AllowUnknown),
        "owned_into" | "ref_into" | "into" | "from_owned" | "from_ref" | "from" | 
        "map_owned" | "map_ref" | "map" | "owned_into_existing" | "ref_into_existing" | "into_existing" => 
            Ok(DataTypeInstruction::Map(TraitAttr { 
                core: syn::parse2(input)?,
                fallible: false,
                applicable_to: [
                    appl_owned_into(instr_str),
                    appl_ref_into(instr_str),
                    appl_from_owned(instr_str),
                    appl_from_ref(instr_str),
                    appl_owned_into_existing(instr_str), 
                    appl_ref_into_existing(instr_str)
                ]
            })),
        "owned_try_into" | "ref_try_into" | "try_into" | "try_from_owned" | "try_from_ref" | "try_from" | 
        "try_map_owned" | "try_map_ref" | "try_map" | "owned_try_into_existing" | "ref_try_into_existing" | "try_into_existing" => {
            Ok(DataTypeInstruction::Map(TraitAttr {
                core: syn::parse2(input)?,
                fallible: true,
                applicable_to: [
                    appl_owned_into(instr_str),
                    appl_ref_into(instr_str),
                    appl_from_owned(instr_str),
                    appl_from_ref(instr_str),
                    appl_owned_into_existing(instr_str), 
                    appl_ref_into_existing(instr_str)
                ]
            }))
        },
        "ghosts" | "ghosts_ref" | "ghosts_owned" => Ok(DataTypeInstruction::Ghosts(GhostsAttr {
            attr: syn::parse2(input)?,
            applicable_to: [
                appl_ghosts_owned(instr_str),
                appl_ghosts_ref(instr_str),
                appl_ghosts_owned(instr_str),
                appl_ghosts_ref(instr_str),
                appl_ghosts_owned(instr_str),
                appl_ghosts_ref(instr_str)
            ]
        })),
        "children" => Ok(DataTypeInstruction::Children(syn::parse2(input)?)),
        "where_clause" => Ok(DataTypeInstruction::Where(syn::parse2(input)?)),
        "ghost" if bark => Ok(DataTypeInstruction::Misnamed { instr: "ghost", span: instr.span(), guess_name: "ghosts", own: own_instr }),
        "ghost_ref" if bark => Ok(DataTypeInstruction::Misnamed { instr: "ghost_ref", span: instr.span(), guess_name: "ghosts_ref", own: own_instr }),
        "ghost_owned" if bark => Ok(DataTypeInstruction::Misnamed { instr: "ghost_owned", span: instr.span(), guess_name: "ghosts_owned", own: own_instr }),
        "child" if bark => Ok(DataTypeInstruction::Misnamed { instr: "child", span: instr.span(), guess_name: "children", own: own_instr }),
        "parent" if bark => Ok(DataTypeInstruction::Misplaced { instr: "parent", span: instr.span(), own: own_instr }),
        "as_type" if bark => Ok(DataTypeInstruction::Misplaced { instr: "as_type", span: instr.span(), own: own_instr }),
        "literal" if bark => Ok(DataTypeInstruction::Misplaced { instr: "literal", span: instr.span(), own: own_instr }),
        "pattern" if bark => Ok(DataTypeInstruction::Misplaced { instr: "pattern", span: instr.span(), own: own_instr }),
        "repeat" if bark => Ok(DataTypeInstruction::Misplaced { instr: "repeat", span: instr.span(), own: own_instr }),
        "stop_repeat" if bark => Ok(DataTypeInstruction::Misplaced { instr: "stop_repeat", span: instr.span(), own: own_instr }),
        "type_hint" if bark => Ok(DataTypeInstruction::Misplaced { instr: "type_hint", span: instr.span(), own: own_instr }),
        _ if own_instr => Ok(DataTypeInstruction::UnrecognizedWithError { instr: instr_str.clone(), span: instr.span() }),
        _ => Ok(DataTypeInstruction::Unrecognized),
    }
}

fn parse_member_instruction(instr: &Ident, input: TokenStream, own_instr: bool, bark: bool) -> Result<MemberInstruction> {
    let instr_str = &instr.to_string();
    match instr_str.as_ref() {
        "owned_into" | "ref_into" | "into" | "from_owned" | "from_ref" | "from" | 
        "map_owned" | "map_ref" | "map" | "owned_into_existing" | "ref_into_existing" | "into_existing" => 
            Ok(MemberInstruction::Map(MemberAttr { 
                attr: syn::parse2(input)?,
                fallible: false,
                original_instr: instr_str.clone(),
                applicable_to: [
                    appl_owned_into(instr_str),
                    appl_ref_into(instr_str),
                    appl_from_owned(instr_str),
                    appl_from_ref(instr_str),
                    appl_owned_into_existing(instr_str), 
                    appl_ref_into_existing(instr_str)
                ]
            })),
        "owned_try_into" | "ref_try_into" | "try_into" | "try_from_owned" | "try_from_ref" | "try_from" | 
        "try_map_owned" | "try_map_ref" | "try_map" => 
            Ok(MemberInstruction::Map(MemberAttr { 
                attr: syn::parse2(input)?,
                fallible: true,
                original_instr: instr_str.clone(),
                applicable_to: [
                    appl_owned_into(instr_str),
                    appl_ref_into(instr_str),
                    appl_from_owned(instr_str),
                    appl_from_ref(instr_str),
                    appl_owned_into_existing(instr_str), 
                    appl_ref_into_existing(instr_str)
                ]
            })),
        "ghost" | "ghost_ref" | "ghost_owned" => Ok(MemberInstruction::Ghost(GhostAttr {
            attr: syn::parse2(input)?,
            applicable_to: [
                appl_ghost_owned(instr_str),
                appl_ghost_ref(instr_str),
                appl_ghost_owned(instr_str),
                appl_ghost_ref(instr_str),
                appl_ghost_owned(instr_str),
                appl_ghost_ref(instr_str)
            ]
        })),
        "ghosts" | "ghosts_ref" | "ghosts_owned" => Ok(MemberInstruction::Ghosts(GhostsAttr { 
            attr: syn::parse2(input)?, 
            applicable_to: [
                appl_ghosts_owned(instr_str),
                appl_ghosts_ref(instr_str),
                appl_ghosts_owned(instr_str),
                appl_ghosts_ref(instr_str),
                appl_ghosts_owned(instr_str),
                appl_ghosts_ref(instr_str)
            ]
        })),
        "child" => Ok(MemberInstruction::Child(syn::parse2(input)?)),
        "parent" => Ok(MemberInstruction::Parent(syn::parse2(input)?)),
        "as_type" => Ok(MemberInstruction::As(syn::parse2(input)?)),
        "literal" => Ok(MemberInstruction::Lit(syn::parse2(input)?)),
        "pattern" => Ok(MemberInstruction::Pat(syn::parse2(input)?)),
        "repeat" => Ok(MemberInstruction::Repeat(syn::parse2(input)?)),
        "stop_repeat" => Ok(MemberInstruction::StopRepeat),
        "type_hint" => Ok(MemberInstruction::VariantTypeHint(syn::parse2(input)?)),
        "children" if bark => Ok(MemberInstruction::Misnamed { instr: "children", span: instr.span(), guess_name: "child", own: own_instr }),
        "where_clause" if bark => Ok(MemberInstruction::Misplaced { instr: "where_clause", span: instr.span(), own: own_instr }),
        "allow_unknown" if bark => Ok(MemberInstruction::Misplaced { instr: "allow_unknown", span: instr.span(), own: own_instr }),
        _ if own_instr => Ok(MemberInstruction::UnrecognizedWithError { instr: instr_str.clone(), span: instr.span() }),
        _ => Ok(MemberInstruction::Unrecognized)
    }
}

fn try_parse_type_hint(input: ParseStream) -> Result<TypeHint> {
    if !input.peek(Token![as]){
        return Ok(TypeHint::Unspecified)
    }

    input.parse::<Token![as]>()?;

    let mut _content;
    if input.peek(Brace) {
        braced!(_content in input);
        return Ok(TypeHint::Struct)
    }

    if input.peek(Paren) {
        parenthesized!(_content in input);
        return Ok(TypeHint::Tuple)
    }

    if input.peek(kw::Unit) {
        input.parse::<kw::Unit>()?;
        return Ok(TypeHint::Unit)
    }

    Err(input.error("Only '()', '{}', and 'Unit' are supported type hints."))
}

fn try_parse_container_ident(input: ParseStream, can_be_empty_after: bool) -> Option<TypePath> {
    if peek_container_path(input, can_be_empty_after) {
        let ident = input.parse::<syn::Path>();
        if input.peek(Token![|]) {
            input.parse::<Token![|]>().unwrap();
        }
        return ident.ok().map(|x| x.into());
    }
    None
}

fn try_parse_optional_ident(input: ParseStream) -> Option<Member> {
    if peek_member(input) && input.peek2(Token![,]) {
        let ident = input.parse::<Member>().ok();
        input.parse::<Token![,]>().unwrap();
        return ident;
    }
    if peek_member(input) {
        let fork = input.fork();
        fork.parse::<Member>().unwrap();
        if fork.is_empty() {
            return input.parse::<Member>().ok()
        }
    }
    None
}

fn peek_member(input: ParseStream) -> bool {
    if input.peek(Ident) {
        return true
    }

    let fork = input.fork();
    fork.parse::<syn::Index>().is_ok()
}

fn peek_container_path(input: ParseStream, can_be_empty: bool) -> bool {
    let fork = input.fork();
    match fork.parse::<syn::Path>() {
        Ok(_) => (can_be_empty && fork.is_empty()) || fork.peek(Token![|]),
        Err(_) => false,
    }
}

fn peek_ghost_field_name(input: ParseStream) -> bool {
    peek_member(input) && input.peek2(Token![:])
}

fn try_parse_children(input: ParseStream) -> Result<Punctuated<ChildData, Token![,]>> {
    input.parse_terminated(|x| {
        let child_path: Punctuated<Member, Token![.]> = Punctuated::parse_separated_nonempty(x)?;
        x.parse::<Token![:]>()?;
        let ty = x.parse::<syn::Path>()?;
        Ok(ChildData{
            ty,
            type_hint: try_parse_type_hint(x)?,
            field_path: child_path.clone(),
            field_path_str: child_path.to_token_stream().to_string().chars().filter(|c| !c.is_whitespace()).collect(),
        })
    })
}

fn try_parse_action(input: ParseStream, allow_braceless: bool) -> Result<Option<TokenStream>> {
    if input.is_empty() {
        Ok(None)
    } else if input.peek(Token![@]) || input.peek(Token![~]) {
        return Ok(Some(input.parse()?))
    } else if allow_braceless && !input.peek(Brace) {
        Ok(Some(input.parse()?))
    } else {
        let content;
        braced!(content in input);
        return Ok(Some(content.parse()?))
    }
}

fn try_parse_braced_action(input: ParseStream) -> Result<TokenStream> {
    let content;
    braced!(content in input);

    content.parse::<TokenStream>()
}

fn add_as_type_attrs(input: &syn::Field, attr: AsAttr, attrs: &mut Vec<MemberAttr>) {
    let this_ty = input.ty.to_token_stream();
    let that_ty = attr.tokens;
    attrs.push(MemberAttr { 
        attr: MemberAttrCore { 
            container_ty: attr.container_ty.clone(),
            member: attr.member.clone(), 
            action: Some(quote!(~ as #this_ty)),
        },
        fallible: false,
        original_instr: "as_type".into(),
        applicable_to: [false, false, true, true, false, false]
    });
    attrs.push(MemberAttr { 
        attr: MemberAttrCore { 
            container_ty: attr.container_ty, 
            member: attr.member, 
            action: Some(quote!(~ as #that_ty)),
        },
        fallible: false,
        original_instr: "as_type".into(),
        applicable_to: [true, true, false, false, true, true]
    });
}

fn appl_owned_into(instr: &str) -> bool {
    matches!(instr, "owned_into" | "into" | "map_owned" | "map" | "owned_try_into" | "try_into" | "try_map_owned" | "try_map")
}
fn appl_ref_into(instr: &str) -> bool {
    matches!(instr, "ref_into" | "into" | "map_ref" | "map" | "ref_try_into" | "try_into" | "try_map_ref" | "try_map")
}
fn appl_from_owned(instr: &str) -> bool {
    matches!(instr, "from_owned" | "from" | "map_owned" | "map" | "try_from_owned" | "try_from" | "try_map_owned" | "try_map")
}
fn appl_from_ref(instr: &str) -> bool {
    matches!(instr, "from_ref" | "from" | "map_ref" | "map" | "try_from_ref" | "try_from" | "try_map_ref" | "try_map")
}

fn appl_owned_into_existing(instr: &str) -> bool {
    matches!(instr, "owned_into_existing" | "into_existing" | "owned_try_into_existing" | "try_into_existing")
}

fn appl_ref_into_existing(instr: &str) -> bool {
    matches!(instr, "ref_into_existing" | "into_existing" | "ref_try_into_existing" | "try_into_existing")
}

fn appl_ghosts_owned(instr: &str) -> bool {
    matches!(instr, "ghosts" | "ghosts_owned")
}

fn appl_ghosts_ref(instr: &str) -> bool {
    matches!(instr, "ghosts" | "ghosts_ref")
}

fn appl_ghost_owned(instr: &str) -> bool {
    matches!(instr, "ghost" | "ghost_owned")
}

fn appl_ghost_ref(instr: &str) -> bool {
    matches!(instr, "ghost" | "ghost_ref")
}

fn build_child_path_str(child_path: &Punctuated<Member, Token![.]>) -> Vec<String> {
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