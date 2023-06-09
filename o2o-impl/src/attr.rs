use std::hash::Hash;
use std::ops::Index;

use proc_macro2::{TokenStream, Span};
use quote::{ToTokens, quote};
use syn::parse::{ParseStream, Parse};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Brace, Paren};
use syn::{Attribute, Ident, Result, Token, Member, parenthesized, braced, WherePredicate, Error};

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

enum StructInstruction {
    Map(StructAttr),
    Ghost(StructGhostAttr), 
    Where(WhereAttr),
    Children(ChildrenAttr),
    Wrapped(WrappedAttr),
    Unrecognized
}

enum MemberInstruction {
    Map(FieldAttr),
    Ghost(FieldGhostAttr),
    Child(FieldChildAttr),
    Parent(ParentAttr),
    As(AsAttr),
    Repeat(RepeatFor),
    StopRepeat,
    Wrapper(WrapperAttr),
    Unrecognized,
    
}

#[derive(Clone)]
pub(crate) struct TypePath {
    pub span: Span,
    pub path: TokenStream,
    pub path_str: String,
    pub nameless: bool,
}

impl From<syn::Path> for TypePath {
    fn from(value: syn::Path) -> Self {
        TypePath {
            span: value.span(),
            path: value.to_token_stream(),
            path_str: value.to_token_stream().to_string(),
            nameless: false
        }
    }
}

impl From<TokenStream> for TypePath {
    fn from(value: TokenStream) -> Self {
        TypePath {
            span: value.span(),
            path_str: value.to_string(),
            path: value,
            nameless: true
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

pub(crate) struct StructAttrs {
    pub attrs: Vec<StructAttr>,
    pub ghost_attrs: Vec<StructGhostAttr>,
    pub where_attrs: Vec<WhereAttr>,
    pub children_attrs: Vec<ChildrenAttr>,
    pub wrapped_attrs: Vec<WrappedAttr>,
}

impl<'a> StructAttrs {
    pub(crate) fn iter_for_kind(&'a self, kind: &'a Kind) -> impl Iterator<Item = &StructAttrCore> {
        self.attrs.iter().filter(move |x| x.applicable_to[kind]).map(|x| &x.attr)
    }

    pub(crate) fn ghost_attr(&'a self, container_ty: &'a TypePath, kind: &'a Kind) -> Option<&StructGhostAttrCore> {
        self.ghost_attrs.iter()
            .find(|x| x.applicable_to[kind] && x.attr.container_ty.is_some() && x.attr.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.ghost_attrs.iter().find(|x| x.applicable_to[kind] && x.attr.container_ty.is_none())).map(|x| &x.attr)
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

    pub(crate) fn wrapped_attr(&'a self, container_ty: &TypePath) -> Option<&WrappedAttr>{
        self.wrapped_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.wrapped_attrs.iter().find(|x| x.container_ty.is_none()))
    }
}

type RepeatFor = [bool; 4];
struct RepeatForWrap(RepeatFor);

enum AttrType {
    Attr,
    Child,
    Parent,
    Ghost
}

impl Index<&AttrType> for RepeatFor {
    type Output = bool;

    fn index(&self, index: &AttrType) -> &Self::Output {
        match index {
            AttrType::Attr => &self[0],
            AttrType::Child => &self[1],
            AttrType::Parent => &self[2],
            AttrType::Ghost => &self[3],
        }
    }
}

impl Parse for RepeatForWrap {
    fn parse(input: ParseStream) -> Result<Self> {
        let types: Punctuated<Ident, Token![,]>  = Punctuated::parse_terminated(input)?;
        if types.is_empty() {
            return Ok(RepeatForWrap([true,  true, true, true]))
        }

        let mut repeat: RepeatFor = [false, false, false, false];

        for ty in types {
            let str = ty.to_token_stream().to_string();
            match str.as_str() {
                "map" => repeat[0] = true,
                "child" => repeat[1] = true,
                "parent" => repeat[2] = true,
                "ghost" => repeat[3] = true,
                _ => return Err(Error::new(ty.span(), format!("#[repeat] of instruction type '{}' is not supported. Supported types are: 'map', 'child', 'parent', 'ghost'", str))),
            };
        }

        Ok(RepeatForWrap(repeat))
    }
}

#[derive(Clone, Default)]
pub(crate) struct FieldAttrs {
    pub attrs: Vec<FieldAttr>,
    pub child_attrs: Vec<FieldChildAttr>,
    pub parent_attrs: Vec<ParentAttr>,
    pub ghost_attrs: Vec<FieldGhostAttr>,
    pub repeat: Option<RepeatFor>,
    pub stop_repeat: bool,
}

impl<'a> FieldAttrs {
    pub(crate) fn iter_for_kind(&'a self, kind: &'a Kind) -> impl Iterator<Item = &FieldAttrCore> {
        self.attrs.iter().filter(move |x| x.applicable_to[kind]).map(|x| &x.attr)
    }

    pub(crate) fn applicable_attr(&'a self, kind: &'a Kind, container_ty: &TypePath) -> Option<ApplicableAttr> {
        self.ghost(container_ty, kind)
            .map(ApplicableAttr::Ghost)
            .or_else(|| self.field_attr(kind, container_ty)
                .or_else(|| if kind == &Kind::OwnedIntoExisting { self.field_attr(&Kind::OwnedInto, container_ty) } else { None })
                .or_else(|| if kind == &Kind::RefIntoExisting { self.field_attr(&Kind::RefInto, container_ty) } else { None })
                .map(ApplicableAttr::Field))
    }

    pub(crate) fn child(&'a self, container_ty: &TypePath) -> Option<&FieldChildAttr>{
        self.child_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.child_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn ghost(&'a self, container_ty: &TypePath, kind: &'a Kind) -> Option<&FieldGhostAttrCore>{
        self.ghost_attrs.iter()
            .find(|x| x.applicable_to[kind] && x.attr.container_ty.is_some() && x.attr.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.ghost_attrs.iter().find(|x| x.applicable_to[kind] && x.attr.container_ty.is_none())).map(|x| &x.attr)
    }

    pub(crate) fn has_parent_attr(&'a self, container_ty: &TypePath) -> bool {
        self.parent_attrs.iter()
            .any(|x| x.container_ty.is_none() || x.container_ty.as_ref().unwrap() == container_ty)
    }

    pub(crate) fn field_attr(&'a self, kind: &'a Kind, container_ty: &TypePath) -> Option<&FieldAttrCore>{
        self.iter_for_kind(kind)
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.iter_for_kind(kind).find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn merge(&'a mut self, other: Self) {
        if let Some(repeat) =  other.repeat  {
            if repeat[&AttrType::Attr] {
                self.attrs.extend(other.attrs);
            }
            if repeat[&AttrType::Child] {
                self.child_attrs.extend(other.child_attrs);
            }
            if repeat[&AttrType::Parent] {
                self.parent_attrs.extend(other.parent_attrs);
            }
            if repeat[&AttrType::Ghost] {
                self.ghost_attrs.extend(other.ghost_attrs);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum StructKindHint {
    Struct = 0,
    Tuple = 1,
    Unspecified = 2
}

pub(crate) struct StructAttr {
    pub attr: StructAttrCore,
    pub applicable_to: ApplicableTo,
}

pub(crate) struct StructAttrCore {
    pub ty: TypePath,
    pub struct_kind_hint: StructKindHint,
    pub init_data: Option<Punctuated<InitData, Token![,]>>,
    pub quick_return: Option<Action>
}

impl Parse for StructAttrCore {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty: TypePath = if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let content_stream = content.parse::<TokenStream>()?;
            quote!((#content_stream)).into()
        } else { input.parse::<syn::Path>()?.into() };
        let struct_kind_hint = if ty.nameless { StructKindHint::Tuple } else { try_parse_struct_kind_hint(input)? };
        let init_data: Option<Punctuated<InitData, Token![,]>> = if input.peek(Token![|]) {
            input.parse::<Token![|]>()?;
            Some(Punctuated::parse_separated_nonempty(input)?)
        } else { None };
        let quick_return = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            try_parse_action(input, true)?
        } else { None };

        Ok(StructAttrCore { ty, struct_kind_hint, init_data, quick_return })
    }
}

pub(crate) struct InitData {
    pub ident: Ident,
    _colon: Token![:],
    pub action: Action,
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

pub(crate) struct StructGhostAttr {
    pub attr: StructGhostAttrCore,
    pub applicable_to: ApplicableTo,
}

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

pub(crate) struct GhostData {
    pub child_path: Option<ChildPath>,
    pub ghost_ident: Member,
    pub action: Action,
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
    pub struct_kind_hint: StructKindHint,
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

pub(crate) struct WrappedAttr {
    pub container_ty: Option<TypePath>,
    pub child_path: Option<ChildPath>,
    pub ident: Member,
    pub action: Option<TokenStream>,
}

impl Parse for WrappedAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        let child_path = if !peek_wrapped_field_name(input) {
            let child_path = Some(Punctuated::parse_separated_nonempty(input)?).map(|child_path| {
                let child_path_str = build_child_path_str(&child_path);
                ChildPath { child_path, child_path_str }
            });
            input.parse::<Token![@]>()?;
            child_path
        } else { None };
        Ok(WrappedAttr {
            container_ty,
            child_path,
            ident: try_parse_optional_ident(input).ok_or(input.error("Member path is expected here."))?,
            action: try_parse_wrapper_action(input)?
        })
    }
}

#[derive(Clone)]
pub(crate) struct FieldAttr  {
    pub attr: FieldAttrCore,
    applicable_to: ApplicableTo,
}

#[derive(Clone)]
pub(crate) struct FieldAttrCore {
    pub container_ty: Option<TypePath>,
    pub member: Option<Member>,
    pub action: Option<Action>,
    pub wrapper: bool,
}

impl Parse for FieldAttrCore {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FieldAttrCore {
            container_ty: try_parse_container_ident(input, false),
            member: try_parse_optional_ident(input),
            action: try_parse_action(input, false)?,
            wrapper: false
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
pub(crate) struct FieldGhostAttr {
    pub attr: FieldGhostAttrCore,
    pub applicable_to: ApplicableTo,
}

#[derive(Clone)]
pub(crate) struct FieldGhostAttrCore {
    pub container_ty: Option<TypePath>,
    pub action: Option<Action>,
}

impl Parse for FieldGhostAttrCore {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FieldGhostAttrCore { 
            container_ty: try_parse_container_ident(input, true),
            action: try_parse_action(input, false)?,
        })
    }
}

pub(crate) enum ApplicableAttr<'a> {
    Field(&'a FieldAttrCore),
    Ghost(&'a FieldGhostAttrCore),
}

#[derive(Clone)]
pub(crate) struct FieldChildAttr {
    pub container_ty: Option<TypePath>,
    pub child_path: ChildPath,
}

impl FieldChildAttr {
    pub(crate) fn get_child_path_str(&self, depth: Option<usize>) -> &str {
        self.child_path.get_child_path_str(depth)
    }
}

impl Parse for FieldChildAttr{
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        let child_path: Punctuated<Member, Token![.]> = Punctuated::parse_separated_nonempty(input)?;
        let child_path_str = build_child_path_str(&child_path);
        Ok(FieldChildAttr { 
            container_ty,
            child_path: ChildPath { 
                child_path,
                child_path_str,
            }
        })
    }
}

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

pub(crate) struct WrapperAttr(pub Option<TokenStream>);

impl Parse for WrapperAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(WrapperAttr(try_parse_wrapper_action(input)?))
    }
}

#[derive(Clone)]
pub(crate) enum Action {
    InlineAtExpr(TokenStream),
    InlineTildeExpr(TokenStream),
    InlineExpr(TokenStream),
    Closure(TokenStream),
    ParamlessClosure(TokenStream),
}

pub(crate) fn get_struct_attrs(input: &[Attribute]) -> Result<StructAttrs> {
    let mut instrs: Vec<StructInstruction> = vec![];
    for x in input.iter() {
        if x.path.is_ident("doc"){
            continue;
        } else if x.path.is_ident("o2o") {
            x.parse_args_with(|input: ParseStream| {
                let new_instrs: Punctuated<StructInstruction, Token![,]> = Punctuated::parse_terminated_with(input, |input| {
                    let instr = input.parse::<Ident>()?;
                    let p: OptionalParenthesizedTokenStream = input.parse()?;
                    parse_struct_instruction(&instr, p.content(), true)
                })?;
                instrs.extend(new_instrs.into_iter());
                Ok(())
            })?;
        } else {
            let instr = x.path.get_ident().unwrap();
            let p: OptionalParenthesizedTokenStream = syn::parse2(x.tokens.clone())?;
            instrs.push(parse_struct_instruction(instr, p.content(), false)?);
        }
    }
    let mut attrs: Vec<StructAttr> = vec![];
    let mut ghost_attrs: Vec<StructGhostAttr> = vec![];
    let mut where_attrs: Vec<WhereAttr> = vec![];
    let mut children_attrs: Vec<ChildrenAttr> = vec![];
    let mut wrapped_attrs: Vec<WrappedAttr> = vec![];
    
    for instr in  instrs {
        match instr {
            StructInstruction::Map(attr) => attrs.push(attr),
            StructInstruction::Ghost(attr) => ghost_attrs.push(attr),
            StructInstruction::Where(attr) => where_attrs.push(attr),
            StructInstruction::Children(attr) => children_attrs.push(attr),
            StructInstruction::Wrapped(attr) => wrapped_attrs.push(attr),
            StructInstruction::Unrecognized => (),
        };
    }
    Ok(StructAttrs {attrs, ghost_attrs, where_attrs, children_attrs, wrapped_attrs })
}

pub(crate) fn get_field_attrs(input: &syn::Field) -> Result<FieldAttrs> {
    let mut instrs: Vec<MemberInstruction> = vec![];
    for x in input.attrs.iter() {
        if x.path.is_ident("doc"){
            continue;
        } else if x.path.is_ident("o2o") {
            x.parse_args_with(|input: ParseStream| {
                let new_instrs: Punctuated<MemberInstruction, Token![,]> = Punctuated::parse_terminated_with(input, |input| {
                    let instr = input.parse::<Ident>()?;
                    let p: OptionalParenthesizedTokenStream = input.parse()?;
                    parse_member_instruction(&instr, p.content(), true)
                })?;
                instrs.extend(new_instrs.into_iter());
                Ok(())
            })?;
        } else {
            let instr = x.path.get_ident().unwrap();
            let p: OptionalParenthesizedTokenStream = syn::parse2(x.tokens.clone())?;
            instrs.push(parse_member_instruction(instr, p.content(), false)?);
        }
    }
    let mut child_attrs: Vec<FieldChildAttr> = vec![];
    let mut ghost_attrs: Vec<FieldGhostAttr> = vec![];
    let mut attrs: Vec<FieldAttr> = vec![];
    let mut parent_attrs: Vec<ParentAttr> = vec![];
    let mut repeat = None;
    let mut stop_repeat = false;
    for instr in  instrs {
        match instr {
            MemberInstruction::Map(attr) => attrs.push(attr),
            MemberInstruction::Child(attr) => child_attrs.push(attr),
            MemberInstruction::Ghost(attr) => ghost_attrs.push(attr),
            MemberInstruction::Parent(attr) => parent_attrs.push(attr),
            MemberInstruction::As(attr) => add_as_type_attrs(input, attr, &mut attrs),
            MemberInstruction::Repeat(repeat_for) => repeat = Some(repeat_for),
            MemberInstruction::StopRepeat => stop_repeat = true,
            MemberInstruction::Wrapper(attr) => {
                let container_ty: TypePath = match &input.ty {
                    syn::Type::Path(path) => path.path.clone().into(),
                    _ => panic!("weird")
                };
                add_wrapper_attrs(&Some(container_ty), attr, &mut attrs, false)
            },
            MemberInstruction::Unrecognized => ()
        };
    }
    Ok(FieldAttrs { child_attrs, parent_attrs, attrs, ghost_attrs, repeat, stop_repeat })
}

fn parse_struct_instruction(instr: &Ident, input: TokenStream, bark: bool) -> Result<StructInstruction>
{
    let instr_str = &instr.to_token_stream().to_string();
    match instr_str.as_ref() {
        "owned_into" | "ref_into" | "into" | "from_owned" | "from_ref" | "from" | 
        "map_owned" | "map_ref" | "map" | "owned_into_existing" | "ref_into_existing" | "into_existing" => 
            Ok(StructInstruction::Map(StructAttr { 
                attr: syn::parse2(input)?, 
                applicable_to: [
                    appl_owned_into(instr_str), 
                    appl_ref_into(instr_str), 
                    appl_from_owned(instr_str), 
                    appl_from_ref(instr_str), 
                    appl_owned_into_existing(instr_str), 
                    appl_ref_into_existing(instr_str)
                ]
            })),
        "ghost" | "ghost_ref" | "ghost_owned" => Ok(StructInstruction::Ghost(StructGhostAttr {
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
        "children" => Ok(StructInstruction::Children(syn::parse2(input)?)),
        "where_clause" => Ok(StructInstruction::Where(syn::parse2(input)?)),
        "wrapped" => Ok(StructInstruction::Wrapped(syn::parse2(input)?)),
        _ if bark => Err(Error::new(instr.span(), format_args!("Struct level instruction '{}' is not supported.", instr))),
        _ => Ok(StructInstruction::Unrecognized),
    }
}

fn parse_member_instruction(instr: &Ident, input: TokenStream, bark: bool) -> Result<MemberInstruction> {
    let instr_str = &instr.to_string();
    match instr_str.as_ref() {
        "owned_into" | "ref_into" | "into" | "from_owned" | "from_ref" | "from" | 
        "map_owned" | "map_ref" | "map" | "owned_into_existing" | "ref_into_existing" | "into_existing" => 
            Ok(MemberInstruction::Map(FieldAttr { 
                attr: syn::parse2(input)?, 
                applicable_to: [
                    appl_owned_into(instr_str), 
                    appl_ref_into(instr_str), 
                    appl_from_owned(instr_str), 
                    appl_from_ref(instr_str), 
                    appl_owned_into_existing(instr_str), 
                    appl_ref_into_existing(instr_str)
                ]
            })),
        "ghost" | "ghost_ref" | "ghost_owned" => Ok(MemberInstruction::Ghost(FieldGhostAttr {
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
        "child" => Ok(MemberInstruction::Child(syn::parse2(input)?)),
        "parent" => Ok(MemberInstruction::Parent(syn::parse2(input)?)),
        "as_type" => Ok(MemberInstruction::As(syn::parse2(input)?)),
        "repeat" => {
            let repeat: RepeatForWrap = syn::parse2(input)?;
            Ok(MemberInstruction::Repeat(repeat.0))
        },
        "stop_repeat" => Ok(MemberInstruction::StopRepeat),
        "wrapper" => Ok(MemberInstruction::Wrapper(syn::parse2(input)?)),
        _ if bark => Err(Error::new(instr.span(), format_args!("Member level instruction '{}' is not supported.", instr))),
        _ => Ok(MemberInstruction::Unrecognized)
    }
}

fn try_parse_struct_kind_hint(input: ParseStream) -> Result<StructKindHint> {
    if !input.peek(Token![as]){
        return Ok(StructKindHint::Unspecified)
    }

    input.parse::<Token![as]>()?;

    let mut _content;
    if input.peek(Brace) {
        braced!(_content in input);
        return Ok(StructKindHint::Struct)
    }

    if input.peek(Paren) {
        parenthesized!(_content in input);
        return Ok(StructKindHint::Tuple)
    }

    Err(input.error("Only '()' and '{}' are supported struct kind hints."))
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

fn peek_wrapped_field_name(input: ParseStream) -> bool {
    let fork = input.fork();
    match fork.parse::<Member>() {
        Ok(_) => fork.is_empty() || fork.peek(Token![,]),
        Err(_) => false,
    }
}

fn try_parse_children(input: ParseStream) -> Result<Punctuated<ChildData, Token![,]>> {
    input.parse_terminated(|x| {
        let child_path: Punctuated<Member, Token![.]> = Punctuated::parse_separated_nonempty(x)?;
        x.parse::<Token![:]>()?;
        let ty = x.parse::<syn::Path>()?;
        Ok(ChildData{
            ty,
            struct_kind_hint: try_parse_struct_kind_hint(x)?,
            field_path: child_path.clone(),
            field_path_str: child_path.to_token_stream().to_string().chars().filter(|c| !c.is_whitespace()).collect(),
        })
    })
}

fn try_parse_action(input: ParseStream, allow_braceless: bool) -> Result<Option<Action>> {
    if input.is_empty() {
        Ok(None)
    } else if input.peek(Token![@]) {
        input.parse::<Token![@]>()?;
        return Ok(Some(Action::InlineAtExpr(input.parse()?)))
    } else if input.peek(Token![~]) {
        input.parse::<Token![~]>()?;
        return Ok(Some(Action::InlineTildeExpr(input.parse()?)))
    } else if input.peek(Token![|]) {
        if input.peek2(Token![|]) {
            return Ok(Some(Action::ParamlessClosure(input.parse()?)))
        } else {
            validate_closure(input)?;
            return Ok(Some(Action::Closure(input.parse()?)))
        }
    } else if allow_braceless && !input.peek(Brace) {
        Ok(Some(Action::InlineExpr(input.parse()?)))
    } else {
        let content;
        braced!(content in input);
        return Ok(Some(Action::InlineExpr(content.parse()?)))
    }
}

fn try_parse_braced_action(input: ParseStream) -> Result<Action> {
    let mut tokens = Vec::new();
    let mut paramless = false;
    let inline = !input.peek(Token![|]);
    
    if !inline {
        tokens.push(input.parse::<Token![|]>()?.to_token_stream());
        if input.peek(Ident) {
            tokens.push(input.parse::<Ident>()?.to_token_stream());
        } else if input.peek(Token![_]) {
            tokens.push(input.parse::<Token![_]>()?.to_token_stream());
        } else {
            paramless = true;
        }
        tokens.push(input.parse::<Token![|]>()?.to_token_stream());
    }
    
    let content;
    braced!(content in input);

    if inline {
        if content.peek(Token![@]) {
            content.parse::<Token![@]>()?;
        } else {
            paramless = true;
        }
        
        tokens.push(content.parse::<TokenStream>()?);
    } else {
        let content = content.parse::<TokenStream>()?;
        tokens.push(quote!({#content}));
    }

    let token_stream = TokenStream::from_iter(tokens);

    let cl = match (inline, paramless) {
        (true, true) => Action::InlineExpr(token_stream),
        (true, false) => Action::InlineAtExpr(token_stream),
        (false, true) => Action::ParamlessClosure(token_stream),
        (false, false) => Action::Closure(token_stream)
    };
    Ok(cl)
}

fn try_parse_wrapper_action(input: ParseStream) -> Result<Option<TokenStream>> {
    if input.is_empty() {
        Ok(None)
    } else {
        input.parse::<Token![~]>()?;
        Ok(Some(input.parse()?))
    }
}

fn validate_closure(input: ParseStream) -> Result<()> {
    if !input.peek(Token![|]) || (!input.peek2(Ident) && !input.peek2(Token![_])) || !input.peek3(Token![|]) {
        return Err(input.error("A closure is expected here"))
    }
    Ok(())
}

fn add_as_type_attrs(input: &syn::Field, attr: AsAttr, attrs: &mut Vec<FieldAttr>) {
    let this_ty = input.ty.to_token_stream();
    let that_ty = attr.tokens;
    attrs.push(FieldAttr { 
        attr: FieldAttrCore { 
            container_ty: attr.container_ty.clone(), 
            member: attr.member.clone(), 
            action: Some(Action::InlineTildeExpr(quote!(as #this_ty))),
            wrapper: false,
        }, 
        applicable_to: [false, false, true, true, false, false]
    });
    attrs.push(FieldAttr { 
        attr: FieldAttrCore { 
            container_ty: attr.container_ty, 
            member: attr.member, 
            action: Some(Action::InlineTildeExpr(quote!(as #that_ty))),
            wrapper: false
        }, 
        applicable_to: [true, true, false, false, true, true]
    });
}

pub(crate) fn add_wrapper_attrs (container_ty: &Option<TypePath>, attr: WrapperAttr, attrs: &mut Vec<FieldAttr>, inverse: bool) {
    attrs.push(FieldAttr { 
        attr: FieldAttrCore { 
            container_ty: container_ty.clone(), 
            member: None, 
            action: Some(Action::InlineAtExpr(TokenStream::new())),
            wrapper: true,
        }, 
        applicable_to: [false ^ inverse, attr.0.is_none() & inverse, true ^ inverse, attr.0.is_none() & !inverse, false, false]
    });
    attrs.push(FieldAttr { 
        attr: FieldAttrCore { 
            container_ty: container_ty.clone(), 
            member: None, 
            action: None,
            wrapper: true,
        }, 
        applicable_to: [true ^ inverse, attr.0.is_none() & !inverse, false ^ inverse, attr.0.is_none() & inverse, false, false]
    });

    if let Some(action) = attr.0 {
        attrs.push(FieldAttr { 
            attr: FieldAttrCore { 
                container_ty: container_ty.clone(), 
                member: None, 
                action: Some(Action::InlineAtExpr(action.clone())),
                wrapper: true,
            }, 
            applicable_to: [false, false ^ inverse, false, true ^ inverse, false, false]
        });
        attrs.push(FieldAttr { 
            attr: FieldAttrCore { 
                container_ty: container_ty.clone(), 
                member: None, 
                action: Some(Action::InlineTildeExpr(action)),
                wrapper: true,
            }, 
            applicable_to: [false, true ^ inverse, false, false ^ inverse, false, false]
        });
    }
}

fn appl_owned_into(instr: &str) -> bool {
    matches!(instr, "owned_into" | "into" | "map_owned" | "map")
}
fn appl_ref_into(instr: &str) -> bool {
    matches!(instr, "ref_into" | "into" | "map_ref" | "map")
}
fn appl_from_owned(instr: &str) -> bool {
    matches!(instr, "from_owned" | "from" | "map_owned" | "map")
}
fn appl_from_ref(instr: &str) -> bool {
    matches!(instr, "from_ref" | "from" | "map_ref" | "map")
}

fn appl_owned_into_existing(instr: &str) -> bool {
    matches!(instr, "owned_into_existing" | "into_existing")
}

fn appl_ref_into_existing(instr: &str) -> bool {
    matches!(instr, "ref_into_existing" | "into_existing")
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
            child_path_str.push(format!("{}.{}", child_path_str.last().unwrap_or(&String::from("")), x.to_token_stream()))
        }
    });
    child_path_str
}