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
            } else {
                None
            }
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
    PanicDebugInfo, 
}

enum MemberInstruction {
    Map(FieldAttr),
    Ghost(GhostAttr),
    Child(FieldChildAttr), 
    Parent(ParentAttr), 
}

pub(crate) struct TypePath {
    pub span: Span,
    pub path: TokenStream,
    pub path_str: String
}

impl From<syn::Path> for TypePath {
    fn from(value: syn::Path) -> Self {
        TypePath {
            span: value.span(),
            path: value.to_token_stream(),
            path_str: value.to_token_stream().to_string()
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

#[derive(PartialEq)]
pub(crate) enum Kind {
    OwnedInto,
    RefInto,
    FromOwned,
    FromRef,
    OwnedIntoExisting,
    RefIntoExisting
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
    pub panic_debug_info: bool,
}

impl<'a> StructAttrs {
    pub(crate) fn iter_for_kind(&'a self, kind: &'a Kind) -> impl Iterator<Item = &MapStructAttr> {
        self.attrs.iter().filter(move |x| x.applicable_to[kind]).map(|x| &x.attr)
    }

    pub(crate) fn ghost_attr(&'a self, container_ty: &'a TypePath) -> Option<&StructGhostAttr> {
        self.ghost_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.ghost_attrs.iter().find(|x| x.container_ty.is_none()))
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

pub(crate) struct FieldAttrs {
    pub attrs: Vec<FieldAttr>,
    pub child_attrs: Vec<FieldChildAttr>,
    pub parent_attrs: Vec<ParentAttr>,
    pub ghost_attrs: Vec<GhostAttr>,
}

impl<'a> FieldAttrs {
    pub(crate) fn iter_for_kind(&'a self, kind: &'a Kind) -> impl Iterator<Item = &MapFieldAttr> {
        self.attrs.iter().filter(move |x| x.applicable_to[kind]).map(|x| &x.attr)
    }

    pub(crate) fn applicable_attr(&'a self, kind: &'a Kind, container_ty: &TypePath) -> Option<ApplicableAttr> {
        self.ghost(container_ty)
            .map(ApplicableAttr::Ghost)
            .or_else(|| self.field_attr(kind, container_ty)
                .or_else(|| if kind == &Kind::OwnedIntoExisting { self.field_attr(&Kind::OwnedInto, container_ty) } else { None })
                .or_else(|| if kind == &Kind::RefIntoExisting { self.field_attr(&Kind::RefInto, container_ty) } else { None })
                .map(ApplicableAttr::Field))
    }

    pub(crate) fn child(&'a self, ident: &TypePath) -> Option<&FieldChildAttr>{
        self.child_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == ident)
            .or_else(|| self.child_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn ghost(&'a self, ident: &TypePath) -> Option<&GhostAttr>{
        self.ghost_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == ident)
            .or_else(|| self.ghost_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn has_parent_attr(&'a self, ident: &TypePath) -> bool {
        self.parent_attrs.iter()
            .any(|x| x.container_ty.is_none() || x.container_ty.as_ref().unwrap() == ident)
    }

    pub(crate) fn field_attr(&'a self, kind: &'a Kind, ident: &TypePath) -> Option<&MapFieldAttr>{
        self.iter_for_kind(kind)
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == ident)
            .or_else(|| self.iter_for_kind(kind).find(|x| x.container_ty.is_none()))
    }
}

#[derive(Clone, Copy)]
pub(crate) enum StructKindHint {
    Struct = 0,
    Tuple = 1,
    Unspecified = 2
}

pub(crate) struct StructAttr {
    pub attr: MapStructAttr,
    pub applicable_to: ApplicableTo,
}

pub(crate) struct MapStructAttr {
    pub ty: TypePath,
    pub struct_kind_hint: StructKindHint,
}

impl Parse  for MapStructAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(MapStructAttr { 
            ty: input.parse::<syn::Path>()?.into(),
            struct_kind_hint: try_parse_struct_kind_hint(input)?,
        })
    }
}

pub(crate) struct StructGhostAttr {
    pub container_ty: Option<TypePath>,
    pub ghost_data: Punctuated<GhostData, Token![,]>,
}

impl Parse for StructGhostAttr{
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(StructGhostAttr { 
            container_ty: try_parse_container_ident(input, false),
            ghost_data: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

pub(crate) struct GhostData {
    pub ghost_ident: Member,
    pub action: Action,
}

impl Parse for GhostData {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(GhostData {
            ghost_ident: try_parse_ident(input)?,
            action: Action::Closure(parse_braced_closure(input)?)
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

pub(crate) struct FieldAttr  {
    pub attr: MapFieldAttr,
    applicable_to: ApplicableTo,
}

pub(crate) struct MapFieldAttr {
    pub container_ty: Option<TypePath>,
    pub ident: Option<Member>,
    pub action: Option<Action>,
}

impl Parse for MapFieldAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(MapFieldAttr {
            container_ty: try_parse_container_ident(input, false),
            ident: try_parse_optional_ident(input),
            action: try_parse_action(input)?,
        })
    }
}

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

pub(crate) struct GhostAttr {
    pub container_ty: Option<TypePath>,
    pub action: Option<Action>,
}

impl Parse for GhostAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(GhostAttr { 
            container_ty: try_parse_container_ident(input, true),
            action: try_parse_action(input)?,
        })
    }
}

pub(crate) enum ApplicableAttr<'a> {
    Field(&'a MapFieldAttr),
    Ghost(&'a GhostAttr),
}

pub(crate) struct FieldChildAttr {
    pub container_ty: Option<TypePath>,
    pub field_path: Punctuated<Member, Token![.]>,
    field_path_str: Vec<String>,
}

impl FieldChildAttr {
    pub(crate) fn get_field_path_str(&self, depth: Option<usize>) -> &str {
        match depth {
            None => &self.field_path_str[self.field_path_str.len() - 1],
            Some(depth) => &self.field_path_str[depth],
        }
    }
}

impl Parse for FieldChildAttr{
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        let child_path: Punctuated<Member, Token![.]> = Punctuated::parse_separated_nonempty(input)?;
        let field_path = child_path.clone();
        let mut field_path_str = vec![];
        child_path.iter().for_each(|x| {
            if field_path_str.is_empty() {
                field_path_str.push(x.to_token_stream().to_string())
            } else {
                field_path_str.push(format!("{}.{}", field_path_str.last().unwrap_or(&String::from("")), x.to_token_stream()))
            }
        });
        Ok(FieldChildAttr { 
            container_ty,
            field_path,
            field_path_str,
        })
    }
}

pub(crate) enum Action {
    InlineExpr(TokenStream),
    Closure(TokenStream),
}

pub(crate) fn get_struct_attrs(input: &[Attribute]) -> Result<StructAttrs> {
    let mut instrs: Vec<StructInstruction> = vec![];
    for x in input.iter() {
        if x.path.is_ident("o2o") {
            x.parse_args_with(|input: ParseStream| {
                let new_instrs: Punctuated<StructInstruction, Token![,]> = Punctuated::parse_terminated_with(input, |input| {
                    let instr = input.parse::<Ident>()?;
                    let p: OptionalParenthesizedTokenStream = input.parse()?;
                    parse_struct_instruction(&instr, p.content())
                })?;
                instrs.extend(new_instrs.into_iter());
                Ok(())
            })?;
        } else {
            let instr = x.path.get_ident().unwrap();
            let p: OptionalParenthesizedTokenStream = syn::parse2(x.tokens.clone())?;
            instrs.push(parse_struct_instruction(instr, p.content())?);
        }
    }
    let mut ghost_attrs: Vec<StructGhostAttr> = vec![];
    let mut where_attrs: Vec<WhereAttr> = vec![];
    let mut children_attrs: Vec<ChildrenAttr> = vec![];
    let mut attrs: Vec<StructAttr> = vec![];
    let mut panic_debug_info = false;
    for instr in  instrs {
        match instr {
            StructInstruction::Map(attr) => attrs.push(attr),
            StructInstruction::Ghost(attr) => ghost_attrs.push(attr),
            StructInstruction::Where(attr) => where_attrs.push(attr),
            StructInstruction::Children(attr) => children_attrs.push(attr),
            StructInstruction::PanicDebugInfo => panic_debug_info = true
        };
    }
    Ok(StructAttrs {attrs, ghost_attrs, where_attrs, children_attrs, panic_debug_info })
}

pub(crate) fn get_field_attrs(input: &[Attribute]) -> Result<FieldAttrs> {
    let mut instrs: Vec<MemberInstruction> = vec![];
    for x in input.iter() {
        if x.path.is_ident("o2o") {
            x.parse_args_with(|input: ParseStream| {
                let new_instrs: Punctuated<MemberInstruction, Token![,]> = Punctuated::parse_terminated_with(input, |input| {
                    let instr = input.parse::<Ident>()?;
                    let p: OptionalParenthesizedTokenStream = input.parse()?;
                    parse_member_instruction(&instr, p.content())
                })?;
                instrs.extend(new_instrs.into_iter());
                Ok(())
            })?;
        } else {
            let instr = x.path.get_ident().unwrap();
            let p: OptionalParenthesizedTokenStream = syn::parse2(x.tokens.clone())?;
            instrs.push(parse_member_instruction(instr, p.content())?);
        }
    }
    let mut child_attrs: Vec<FieldChildAttr> = vec![];
    let mut ghost_attrs: Vec<GhostAttr> = vec![];
    let mut attrs: Vec<FieldAttr> = vec![];
    let mut parent_attrs: Vec<ParentAttr> = vec![];
    for instr in  instrs {
        match instr {
            MemberInstruction::Map(attr) => attrs.push(attr),
            MemberInstruction::Child(attr) => child_attrs.push(attr),
            MemberInstruction::Ghost(attr) => ghost_attrs.push(attr),
            MemberInstruction::Parent(attr) => parent_attrs.push(attr)
        };
    }
    Ok(FieldAttrs { child_attrs, parent_attrs, attrs, ghost_attrs })
}

fn parse_struct_instruction(instr: &Ident, input: TokenStream) -> Result<StructInstruction>
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
        "ghost" => Ok(StructInstruction::Ghost(syn::parse2(input)?)),
        "children" => Ok(StructInstruction::Children(syn::parse2(input)?)),
        "where_clause" => Ok(StructInstruction::Where(syn::parse2(input)?)),
        "panic_debug_info" => Ok(StructInstruction::PanicDebugInfo),
        _ => Err(Error::new(instr.span(), format_args!("Struct level instruction '{}' is not supported.", instr)))
    }
}

fn parse_member_instruction(instr: &Ident, input: TokenStream) -> Result<MemberInstruction> {
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
        "ghost" => Ok(MemberInstruction::Ghost(syn::parse2(input)?)),
        "child" => Ok(MemberInstruction::Child(syn::parse2(input)?)),
        "parent" => Ok(MemberInstruction::Parent(syn::parse2(input)?)),
        _ => Err(Error::new(instr.span(), format_args!("Member level instruction '{}' is not supported.", instr)))
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

fn try_parse_container_ident(input: ParseStream, can_be_empty: bool) -> Option<TypePath> {
    if peek_container_path(input, can_be_empty) {
        let ident = input.parse::<syn::Path>();
        if input.peek(Token![|]) {
            input.parse::<Token![|]>().unwrap();
        }
        return ident.ok().map(|x| x.into());
    }
    None
}

fn try_parse_ident(input: ParseStream) -> Result<Member> {
    let ident = input.parse::<Member>()?;
    input.parse::<Token![:]>()?;

    Ok(ident)
}

fn try_parse_optional_ident(input: ParseStream) -> Option<Member> {
    if (input.peek(Ident) || peek_index(input)) && input.peek2(Token![,]) {
        let ident = input.parse::<Member>().ok();
        input.parse::<Token![,]>().unwrap();
        return ident;
    }
    if input.peek(Ident) || peek_index(input) {
        let fork = input.fork();
        fork.parse::<Member>().unwrap();
        if fork.is_empty() {
            return input.parse::<Member>().ok()
        }
    }
    None
}

fn peek_index(input: ParseStream) -> bool {
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

fn try_parse_action(input: ParseStream) -> Result<Option<Action>> {
    if input.peek(Token![|]) {
        validate_closure(input)?;
        return Ok(Some(Action::Closure(input.parse()?)))
    } else if !input.is_empty() {
        return Ok(Some(Action::InlineExpr(input.parse()?)))
    }
    Ok(None)
}

fn parse_braced_closure(input: ParseStream) -> Result<TokenStream> {
    validate_closure(input)?;

    let mut tokens = Vec::new();
    
    tokens.push(input.parse::<Token![|]>()?.to_token_stream());
    if input.peek(Ident) {
        tokens.push(input.parse::<Ident>()?.to_token_stream());
    } else {
        tokens.push(input.parse::<Token![_]>()?.to_token_stream());
    }
    tokens.push(input.parse::<Token![|]>()?.to_token_stream());
    let content;
    braced!(content in input);
    let content = content.parse::<TokenStream>()?;

    tokens.push(quote!({#content}));

    Ok(TokenStream::from_iter(tokens.into_iter()))
}

fn validate_closure(input: ParseStream) -> Result<()> {
    if !input.peek(Token![|]) || (!input.peek2(Ident) && !input.peek2(Token![_])) || !input.peek3(Token![|]) {
        return Err(input.error("A closure is expected here"))
    }
    Ok(())
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