use std::{collections::HashMap, iter::Peekable, slice::Iter};

use crate::{
    ast::{DataType, DataTypeMember, Enum, Field, Struct, Variant},
    attr::{ApplicableAttr, ChildData, ChildPath, DataTypeAttrs, GhostData, GhostIdent, Kind, MemberAttrCore, ParentChildField, TraitAttrCore, TypeHint},
    validate::validate,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_quote, Data, DeriveInput, Error, GenericArgument, GenericParam, Index, Lifetime,
    Member::{self, Named, Unnamed}, Result
};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    match &node.data {
        Data::Struct(data) => {
            let input = Struct::from_syn(node, data)?;
            let input = DataType::Struct(&input);
            validate(&input)?;
            Ok(data_type_impl(input))
        },
        Data::Enum(data) => {
            let input = Enum::from_syn(node, data)?;
            let input = DataType::Enum(&input);
            validate(&input)?;
            Ok(data_type_impl(input))
        },
        _ => Err(Error::new_spanned(node, "#[derive(o2o)] only supports structs and enums.")),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ImplType {
    Struct,
    Enum,
    Variant,
}

impl ImplType {
    fn is_variant(self) -> bool {
        self == ImplType::Variant
    }
}

struct ImplContext<'a> {
    input: &'a DataType<'a>,
    impl_type: ImplType,
    struct_attr: &'a TraitAttrCore,
    kind: Kind,
    dst_ty: &'a TokenStream,
    src_ty: &'a TokenStream,
    has_post_init: bool,
    fallible: bool,
}

struct ChildRenderContext<'a> {
    pub ty: &'a syn::Path,
    pub type_hint: TypeHint
}

impl<'a> From<&'a ChildData> for ChildRenderContext<'a> {
    fn from(value: &'a ChildData) -> Self {
        ChildRenderContext { ty: &value.ty, type_hint: value.type_hint }
    }
}

struct FieldContainer<'a> {
    gr_idx: usize,
    path: String,
    field_data: FieldData<'a>
}

enum FieldData<'a> {
    Field(&'a Field),
    GhostData(&'a GhostData),
    ParentChildField(&'a Field, &'a ParentChildField),
}

enum VariantData<'a> {
    Variant(&'a Variant),
    GhostData(&'a GhostData),
}

fn data_type_impl(input: DataType) -> TokenStream {
    let ty = input.get_ident().to_token_stream();
    let attrs = input.get_attrs();

    let impl_type = match input {
        DataType::Struct(_) => ImplType::Struct,
        DataType::Enum(_) => ImplType::Enum,
    };

    let impls = std::iter::empty().chain(attrs.iter_for_kind_core(&Kind::FromOwned, false).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::FromOwned,
        dst_ty: &ty,
        src_ty: &struct_attr.ty.path,
        has_post_init: false,
        fallible: false,
    })).chain(attrs.iter_for_kind_core(&Kind::FromOwned, true).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::FromOwned,
        dst_ty: &ty,
        src_ty: &struct_attr.ty.path,
        has_post_init: false,
        fallible: true,
    })).chain(attrs.iter_for_kind_core(&Kind::FromRef, false).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::FromRef,
        dst_ty: &ty,
        src_ty: &struct_attr.ty.path,
        has_post_init: false,
        fallible: false,
    })).chain(attrs.iter_for_kind_core(&Kind::FromRef, true).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::FromRef,
        dst_ty: &ty,
        src_ty: &struct_attr.ty.path,
        has_post_init: false,
        fallible: true,
    })).chain(attrs.iter_for_kind_core(&Kind::OwnedInto, false).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::OwnedInto,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: false,
        fallible: false,
    })).chain(attrs.iter_for_kind_core(&Kind::OwnedInto, true).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::OwnedInto,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: false,
        fallible: true,
    })).chain(attrs.iter_for_kind_core(&Kind::RefInto, false).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::RefInto,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: false,
        fallible: false,
    })).chain(attrs.iter_for_kind_core(&Kind::RefInto, true).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::RefInto,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: false,
        fallible: true,
    })).chain(attrs.iter_for_kind_core(&Kind::OwnedIntoExisting, false).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::OwnedIntoExisting,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: false,
        fallible: false,
    })).chain(attrs.iter_for_kind_core(&Kind::OwnedIntoExisting, true).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::OwnedIntoExisting,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: false,
        fallible: true,
    })).chain(attrs.iter_for_kind_core(&Kind::RefIntoExisting, false).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::RefIntoExisting,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: false,
        fallible: false,
    })).chain(attrs.iter_for_kind_core(&Kind::RefIntoExisting, true).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::RefIntoExisting,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: false,
        fallible: true,
    })).map(|mut ctx| quote_trait(&input, &mut ctx));

    quote! { #(#impls)* }
}

fn main_code_block(ctx: &ImplContext) -> TokenStream {
    if let Some(quick_return) = &ctx.struct_attr.quick_return {
        //TODO: Consider removing quick returns for into_existing because they are confusing
        if ctx.kind.is_into_existing() {
            let action = quote_action(quick_return, None, ctx);
            return quote!(*other = #action;);
        }
        return quote_action(quick_return, None, ctx);
    }

    match ctx.input {
        DataType::Struct(s) => struct_main_code_block(s, ctx),
        DataType::Enum(e) => enum_main_code_block(e, ctx),
    }
}

fn main_code_block_ok(ctx: &ImplContext) -> TokenStream {
    if let Some(quick_return) = &ctx.struct_attr.quick_return {
        //TODO: Consider removing quick returns for into_existing because they are confusing
        if ctx.kind.is_into_existing() {
            let action = quote_action(quick_return, None, ctx);
            return quote!(*other = #action;);
        }
        return quote_action(quick_return, None, ctx);
    }

    let inner = match ctx.input {
        DataType::Struct(s) => struct_main_code_block(s, ctx),
        DataType::Enum(e) => enum_main_code_block(e, ctx),
    };

    if ctx.has_post_init {
        inner
    } else {
        quote!(Ok(#inner))
    }
}

fn struct_main_code_block(input: &Struct, ctx: &ImplContext) -> TokenStream {
    let struct_init_block = struct_init_block(input, ctx);

    match ctx.kind {
        Kind::FromOwned | Kind::FromRef => {
            let dst = ctx.dst_ty;
            quote!(#dst #struct_init_block)
        },
        Kind::OwnedInto | Kind::RefInto => {
            let dst = if ctx.struct_attr.ty.nameless_tuple || ctx.has_post_init {
                TokenStream::new()
            } else {
                ctx.dst_ty.clone()
            };
            quote!(#dst #struct_init_block)
        },
        Kind::OwnedIntoExisting | Kind::RefIntoExisting => struct_init_block,
    }
}

fn enum_main_code_block(input: &Enum, ctx: &ImplContext) -> TokenStream {
    let enum_init_block = enum_init_block(input, ctx);

    match ctx.kind {
        Kind::FromOwned | Kind::FromRef => {
            quote!(match value #enum_init_block)
        },
        Kind::OwnedInto | Kind::RefInto => {
            quote!(match self #enum_init_block)
        },
        Kind::OwnedIntoExisting | Kind::RefIntoExisting => enum_init_block,
    }
}

fn struct_init_block<'a>(input: &'a Struct, ctx: &ImplContext) -> TokenStream {
    if (!ctx.kind.is_from() && ctx.struct_attr.type_hint == TypeHint::Unit) || (ctx.kind.is_from() && input.unit) {
        return TokenStream::new();
    }

    let mut group_paths = HashMap::<String, usize>::new();
    group_paths.insert("".into(), 0);

    let mut make_tuple = |path: String, field_data: FieldData<'a>| {
        if group_paths.contains_key(&path) {
            let gr_idx = *group_paths.get(&path).unwrap();
            (FieldContainer { gr_idx, path, field_data }, false)
        } else {
            group_paths.insert(path.clone(), group_paths.len());
            (FieldContainer { gr_idx: group_paths.len() - 1, path, field_data}, true)
        }
    };

    let mut fields: Vec<FieldContainer> = vec![];

    fields.extend(input.fields.iter()
        .flat_map(|x| {
            let fields: Vec<FieldContainer> = if let Some(p) = x.attrs.parameterized_parent_attr(&ctx.struct_attr.ty).map(|a| a.child_fields.as_ref().unwrap()) {
                p.iter().map(|p| make_tuple(format!("{}{}", &x.member_str, &p.sub_path_tokens.to_string().replace(' ', "")), FieldData::ParentChildField(x, p)).0).collect()
            } else {
                let path = x.attrs.child(&ctx.struct_attr.ty).map(|x| x.get_child_path_str(None)).unwrap_or(&x.member_str);
                vec![make_tuple(path.to_string(), FieldData::Field(x)).0]
            };
            fields.into_iter()
        }));

    fields.extend(input.attrs.ghosts_attrs.iter()
        .flat_map(|x| &x.attr.ghost_data)
        .filter_map(|x| {
            let res = make_tuple(x.get_child_path_str(None).into(), FieldData::GhostData(x));
            res.1.then_some(res.0)
        }));

    fields.sort_by(|a, b| a.gr_idx.cmp(&b.gr_idx));

    struct_init_block_inner(&mut fields.iter().peekable(), input.named_fields, ctx, None)
}

fn struct_init_block_inner(
    members: &mut Peekable<Iter<FieldContainer>>,
    named_fields: bool, 
    ctx: &ImplContext,
    field_ctx: Option<(&ChildPath, Option<&ChildRenderContext>, usize)>
) -> TokenStream
{
    let type_hint = ctx.struct_attr.type_hint;
    let type_hint = field_ctx.map_or(type_hint, |x|x.1.map_or(type_hint, |x|x.type_hint));

    let mut fragments: Vec<TokenStream> = vec![];
    let mut idx: usize = 0;

    while let Some(FieldContainer { path, field_data, .. }) = members.peek() {
        if let Some(field_ctx) = field_ctx {
            let p = field_ctx.0.get_child_path_str(Some(field_ctx.2));
            if path != p && !path.starts_with(format!("{p}.").as_str()) {
                break;
            }
        }

        match field_data {
            FieldData::Field(f) => {
                let attrs = &f.attrs;
                if !ctx.kind.is_from() && (attrs.ghost(&ctx.struct_attr.ty, &ctx.kind).is_some() || attrs.has_parent_attr(&ctx.struct_attr.ty)) {
                    members.next();
                    continue;
                }

                if ctx.kind.is_from() {
                    if let Some(ghost_attr) = attrs.ghost(&ctx.struct_attr.ty, &ctx.kind) {
                        if ghost_attr.action.is_none() {
                            members.next();
                            continue;
                        }
                    }
                }

                let fragment = match attrs.child(&ctx.struct_attr.ty) {
                    Some(child_attr) => render_child_fragment(&child_attr.child_path, members, ctx, field_ctx.map(|x|x.2), type_hint, || render_struct_line(f, ctx, type_hint, idx, None)),
                    None => {
                        members.next();
                        render_struct_line(f, ctx, type_hint, idx, None)
                    }
                };
                fragments.push(fragment);
                idx += 1;
            },
            FieldData::GhostData(g) => {
                let child_path = &g.child_path.as_ref().unwrap();
                let fragment = render_child_fragment(child_path, members, ctx, field_ctx.map(|x|x.2), type_hint, TokenStream::new);
                fragments.push(fragment);
                idx += 1;
            },
            FieldData::ParentChildField(f, p) => {
                let type_hint = if type_hint == TypeHint::Unspecified { if ctx.input.named_fields() {TypeHint::Struct} else {TypeHint::Tuple} } else { type_hint };
                let fragment = render_parent_child_fragment(f, p, members, p.named_fields(), ctx, field_ctx.map(|x|x.2), || render_struct_line(f, ctx, type_hint, idx, Some(p)));
                fragments.push(fragment);
                idx += 1;
            }
        }
    }

    if !ctx.kind.is_from() {
        if let Some(ghost_attr) = ctx.input.get_attrs().ghosts_attr(&ctx.struct_attr.ty, &ctx.kind) {
            ghost_attr.ghost_data.iter().for_each(|x| match (&x.child_path, field_ctx) {
                (Some(_), Some(field_ctx)) => {
                    if x.get_child_path_str(None) == field_ctx.0.get_child_path_str(Some(field_ctx.2)) {
                        fragments.push(render_ghost_line(x, ctx))
                    }
                }
                (None, None) => fragments.push(render_ghost_line(x, ctx)),
                _ => (),
            });
        }
    }

    if let Some(update) = &ctx.struct_attr.update {
        let a = quote_action(update, None, ctx);
        fragments.push(quote!(..#a))
    }

    if ctx.has_post_init || ctx.kind.is_into_existing() {
        return quote!(#(#fragments)*);
    }

    match (&ctx.kind, type_hint, named_fields) {
        (Kind::FromOwned | Kind::FromRef, _, true) => quote!({#(#fragments)*}),
        (Kind::FromOwned | Kind::FromRef, _, false) => quote!((#(#fragments)*)),
        (_, TypeHint::Struct, _) => quote!({#(#fragments)*}),
        (_, TypeHint::Tuple, _) => quote!((#(#fragments)*)),
        (_, TypeHint::Unspecified, true) => quote!({#(#fragments)*}),
        (_, TypeHint::Unspecified, false) => quote!((#(#fragments)*)),
        (_, TypeHint::Unit, _) => unreachable!("2"),
    }
}

fn enum_init_block(input: &Enum, ctx: &ImplContext) -> TokenStream {
    let mut fields: Vec<VariantData> = vec![];

    fields.extend(input.variants.iter()
        .map(VariantData::Variant).collect::<Vec<VariantData>>());
    
    fields.extend(input.attrs.ghosts_attrs.iter()
        .flat_map(|x| &x.attr.ghost_data)
        .map(VariantData::GhostData));

    enum_init_block_inner(&mut fields.iter().peekable(), input, ctx)
}

fn enum_init_block_inner(members: &mut Peekable<Iter<VariantData>>, input: &Enum, ctx: &ImplContext) -> TokenStream {
    let mut fragments: Vec<TokenStream> = vec![];

    while let Some(member_data) = members.peek() {
        match member_data {
            VariantData::Variant(v) => {
                let attrs = &v.attrs;
                if ctx.kind.is_from() && attrs.ghost(&ctx.struct_attr.ty, &ctx.kind).is_some() {
                    members.next();
                    continue;
                }

                if !ctx.kind.is_from() {
                    if let Some(ghost_attr) = attrs.ghost(&ctx.struct_attr.ty, &ctx.kind) {
                        if ghost_attr.action.is_none() {
                            members.next();
                            continue;
                        }
                    }
                }

                members.next();
                let fragment = render_enum_line(v, ctx);
                fragments.push(fragment);
            }
            VariantData::GhostData(ghost_data) => {
                members.next();
                let fragment = render_enum_ghost_line(ghost_data, ctx);
                fragments.push(fragment);
            }
        }
    }

    if let Some(default_case) = &ctx.struct_attr.default_case {
        if ctx.kind.is_from() && (input.variants.iter().any(|v| v.attrs.lit(&ctx.struct_attr.ty).is_some() || v.attrs.pat(&ctx.struct_attr.ty).is_some()) || input.attrs.ghosts_attr(&ctx.struct_attr.ty, &ctx.kind).is_some())
            || !ctx.kind.is_from() && input.variants.iter().any(|v| v.attrs.ghost(&ctx.struct_attr.ty, &ctx.kind).is_some())
        {
            let g = quote_action(default_case, None, ctx);
            fragments.push(quote!(_ #g))
        }
    }

    quote!({#(#fragments)*})
}

fn variant_destruct_block(input: &Struct, ctx: &ImplContext) -> TokenStream {
    let (mut idents, type_hint) = match (input.named_fields, ctx.kind, ctx.struct_attr.type_hint) {
        (true, Kind::OwnedInto | Kind::RefInto | Kind::OwnedIntoExisting | Kind::RefIntoExisting, _) | 
        (true, _, TypeHint::Struct | TypeHint::Unspecified) | 
        (false, Kind::FromOwned | Kind::FromRef, TypeHint::Struct) => (
            input.fields.iter().filter(|x| !ctx.kind.is_from() || x.attrs.ghost(&ctx.struct_attr.ty, &ctx.kind).is_none())
                .map(|x| {
                    let attr = x.attrs.applicable_attr(&ctx.kind, ctx.fallible, &ctx.struct_attr.ty);

                    if !ctx.kind.is_from() || attr.is_none() {
                        let ident = &x.member;
                        quote!(#ident ,)
                    } else if let Some(attr) = attr {
                        let ident = attr.get_field_name_or(&x.member);
                        quote!(#ident ,)
                    } else { unreachable!("3") }
                }).collect(),
            TypeHint::Struct,
        ),
        (_, Kind::FromOwned | Kind::FromRef, TypeHint::Unit) => (vec![], TypeHint::Unit),
        _ => (
            input.fields.iter().filter(|x| !ctx.kind.is_from() || x.attrs.ghost(&ctx.struct_attr.ty, &ctx.kind).is_none())
                .map(|x| {
                    let ident = format_ident!("f{}", x.idx);
                    quote!(#ident ,)
                }).collect(),
            TypeHint::Tuple,
        ),
    };

    if ctx.kind.is_from() {
        idents.extend(input.attrs.ghosts_attrs.iter().flat_map(|x| &x.attr.ghost_data).map(|x| {
            let ghost_ident = x.ghost_ident.get_ident();
            let ident = match ghost_ident {
                Named(ident) => ident.to_token_stream(),
                Unnamed(index) => format_ident!("f{}", index.index).to_token_stream(),
            };
            quote!(#ident ,)
        }));
    }

    match type_hint {
        TypeHint::Struct => quote!({#(#idents)*}),
        TypeHint::Tuple => quote!((#(#idents)*)),
        TypeHint::Unit => TokenStream::new(),
        _ => unreachable!("4"),
    }
}

fn render_child_fragment<F: Fn() -> TokenStream>(
    child_path: &ChildPath,
    fields: &mut Peekable<Iter<FieldContainer>>,
    ctx: &ImplContext,
    depth: Option<usize>,
    type_hint: TypeHint,
    render_line: F
) -> TokenStream
{
    if depth.is_none() || depth.unwrap() < child_path.child_path_str.len() - 1 {
        let new_depth = depth.map_or(0, |x|x+1);
        match ctx.kind {
            Kind::OwnedInto | Kind::RefInto => {
                let mut children = ctx.input.get_attrs().children_attr(&ctx.struct_attr.ty).unwrap().children.iter();
                let child_data = children.find(|child_data| child_data.check_match(child_path.get_child_path_str(Some(new_depth)))).unwrap();
                
                render_child(&child_data.into(), fields, ctx.input.named_fields(), ctx, (child_path, new_depth), type_hint)
            },
            Kind::OwnedIntoExisting | Kind::RefIntoExisting => render_existing_child(fields, ctx.input.named_fields(), ctx, (child_path, new_depth)),
            Kind::FromOwned | Kind::FromRef => {
                fields.next();
                render_line()
            }
        }
    } else {
        fields.next();
        render_line()
    }
}

fn render_parent_child_fragment<F: Fn() -> TokenStream>(
    field: &Field,
    parent_child_field: &ParentChildField,
    fields: &mut Peekable<Iter<FieldContainer>>,
    named_fields: bool,
    ctx: &ImplContext,
    depth: Option<usize>,
    render_line: F
) -> TokenStream
{
    if depth.is_none() || depth.unwrap() < parent_child_field.sub_path.len() {
        let new_depth = depth.map_or(0, |x|x+1);
        if ctx.kind.is_from() {
            let ty = if let Some(depth) = depth { parent_child_field.sub_path[depth].1.as_ref().unwrap() } else { field.ty.as_ref().unwrap() };
            let child_data = ChildRenderContext { ty, type_hint: ctx.struct_attr.type_hint };
            let child_path = ChildPath::new(field.member.clone(), parent_child_field.sub_path.iter().map(|x|x.0.clone()));
            render_child(&child_data, fields, named_fields, ctx, (&child_path, new_depth), if ctx.input.named_fields() {TypeHint::Struct} else {TypeHint::Tuple})
        } else {
            fields.next();
            render_line()
        }
    } else {
        fields.next();
        render_line()
    }
}

fn struct_pre_init(ctx: &ImplContext) -> Option<TokenStream> {
    if let Some(init_data) = &ctx.struct_attr.init_data {
        let g = init_data.iter().map(|x| {
            let a = &x.ident;
            let b = quote_action(&x.action, None, ctx);

            quote!(let #a = #b;)
        });
        Some(TokenStream::from_iter(g))
    } else {
        None
    }
}

fn struct_post_init(input: &DataType, ctx: &ImplContext) -> Option<TokenStream> {
    let mut fragments: Vec<TokenStream> = vec![];

    input.get_members().iter().for_each(|f| {
        if !ctx.kind.is_from() && f.get_attrs().has_parameterless_parent_attr(&ctx.struct_attr.ty) {
            match f {
                DataTypeMember::Field(f) => fragments.push(render_parent(f, ctx)),
                DataTypeMember::Variant(_) => todo!(),
            }
        }
    });

    if fragments.is_empty() {
        return None;
    }
    Some(quote!(#(#fragments)*))
}

fn render_parent(f: &Field, ctx: &ImplContext) -> TokenStream {
    let member = &f.member;
    match (&ctx.kind, ctx.fallible) {
        (Kind::OwnedIntoExisting, false) => quote!(self.#member.into_existing(other);),
        (Kind::RefIntoExisting, false) => quote!((&(self.#member)).into_existing(other);),
        (Kind::OwnedInto, false) => quote!(self.#member.into_existing(&mut obj);),
        (Kind::RefInto, false) => quote!((&(self.#member)).into_existing(&mut obj);),
        (Kind::OwnedIntoExisting, true) => quote!(self.#member.try_into_existing(other)?;),
        (Kind::RefIntoExisting, true) => quote!((&(self.#member)).try_into_existing(other)?;),
        (Kind::OwnedInto, true) => quote!(self.#member.try_into_existing(&mut obj)?;),
        (Kind::RefInto, true) => quote!((&(self.#member)).try_into_existing(&mut obj)?;),
        _ => unreachable!("5"),
    }
}

fn render_child(
    child_data: &ChildRenderContext,
    fields: &mut Peekable<Iter<FieldContainer>>,
    named_fields: bool,
    ctx: &ImplContext,
    field_ctx: (&ChildPath, usize),
    hint: TypeHint) -> TokenStream
{
    let child_path = field_ctx.0;
    let child_name = child_path.child_path[field_ctx.1].to_token_stream();
    let ty = &child_data.ty;
    let init = struct_init_block_inner(fields, named_fields, ctx, Some((field_ctx.0, Some(child_data), field_ctx.1)));
    match (ctx.input.named_fields(), hint) {
        (true, TypeHint::Struct | TypeHint::Unspecified) => quote!(#child_name: #ty #init,),
        (true, TypeHint::Tuple) => quote!(#ty #init,),
        (false, TypeHint::Tuple | TypeHint::Unspecified) => quote!(#ty #init,),
        (false, TypeHint::Struct) => quote!(#child_name: #ty #init,),
        (_, TypeHint::Unit) => unreachable!("15"),
    }
}

fn render_existing_child(
    fields: &mut Peekable<Iter<FieldContainer>>,
    named_fields: bool,
    ctx: &ImplContext,
    field_ctx: (&ChildPath, usize)
) -> TokenStream
{
    let child_attr = field_ctx.0;
    let path = child_attr.get_child_path_str(Some(field_ctx.1));
    let children_attr = ctx.input.get_attrs().children_attr(&ctx.struct_attr.ty);
    let child_data = children_attr.and_then(|x| x.children.iter().find(|child_data| child_data.check_match(path)));
    struct_init_block_inner(fields, named_fields, ctx, Some((field_ctx.0, child_data.map(|x|x.into()).as_ref(), field_ctx.1)))
}

fn render_struct_line(
    f: &Field,
    ctx: &ImplContext,
    hint: TypeHint,
    idx: usize,
    parent_child: Option<&ParentChildField>
) -> TokenStream
{
    let member = parent_child.map(|p| &p.this_member)
        .unwrap_or(&f.member);
    let attr = parent_child.map(|p| ApplicableAttr::ParentChildField(p, ctx.kind))
        .or_else(|| f.attrs.applicable_attr(&ctx.kind, ctx.fallible, &ctx.struct_attr.ty));
    let get_field_path = |x: &Member| match f.attrs.child(&ctx.struct_attr.ty) {
        Some(child_attr) => {
            let ch = child_attr.child_path.child_path.to_token_stream();
            quote!(#ch.#x)
        }
        None => x.to_token_stream(),
    };
    let get_child_field_path = |x: &Member| match parent_child {
        Some(p) => {
            let sub_path = &p.sub_path_tokens;
            quote!(#x #sub_path.#member)
        },
        None => x.to_token_stream()
    };

    let obj = if ctx.impl_type.is_variant() { TokenStream::new() } else {
        match ctx.kind {
            Kind::OwnedInto => quote!(self.),
            Kind::RefInto => quote!(self.),
            Kind::FromOwned => quote!(value.),
            Kind::FromRef => quote!(value.),
            Kind::OwnedIntoExisting => quote!(self.),
            Kind::RefIntoExisting => quote!(self.),
        }
    };

    match (member, attr, &ctx.kind, hint) {
        (Named(ident), None, Kind::OwnedInto | Kind::RefInto, TypeHint::Struct | TypeHint::Unspecified) =>
            if ctx.has_post_init { quote!(obj.#ident = #obj #ident;) } else { quote!(#ident: #obj #ident,) },
        (Named(ident), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Struct | TypeHint::Unspecified) => {
            let field_path = get_field_path(&f.member);
            quote!(other.#field_path = #obj #ident;)
        },
        (Named(ident), None, Kind::OwnedInto | Kind::RefInto, TypeHint::Tuple) => 
            quote!(#obj #ident,),
        (Named(ident), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Tuple) => {
            let index = Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
            quote!(other.#index = #obj #ident;)
        },
        (Named(ident), None, Kind::FromOwned | Kind::FromRef, TypeHint::Struct | TypeHint::Unspecified | TypeHint::Unit) =>
            if f.attrs.has_parent_attr(&ctx.struct_attr.ty) {
                match (ctx.kind.is_ref(), ctx.fallible) {
                    (true, true) => quote!(#ident: value.try_into()?,),
                    (true, false) => quote!(#ident: value.into(),),
                    (false, true) => quote!(#ident: (&value).try_into()?,),
                    (false, false) => quote!(#ident: (&value).into(),),
                }
            } else {
                let field_path = get_field_path(&f.member);
                quote!(#ident: #obj #field_path,)
            },
        (Named(ident), None, Kind::FromOwned | Kind::FromRef, TypeHint::Tuple) => {
            let index = Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
            let field_path = if ctx.impl_type.is_variant() { get_field_path(&Named(format_ident!("f{}", index))) } else { get_field_path(&index) };
            quote!(#ident: #obj #field_path,)
        },
        (Unnamed(index), None, Kind::OwnedInto | Kind::RefInto, TypeHint::Tuple | TypeHint::Unspecified) =>
            if ctx.has_post_init {
                let index2 = Unnamed(Index { index: idx as u32, span: Span::call_site() });
                quote!(obj.#index2 = #obj #index;)
            } else {
                let index = if ctx.impl_type.is_variant() { format_ident!("f{}", index.index).to_token_stream() } else { index.to_token_stream() };
                quote!(#obj #index,)
            },
        (Unnamed(index), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Tuple | TypeHint::Unspecified) => {
            let index2 = Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
            quote!(other.#index2 = #obj #index;)
        },
        (Unnamed(index), None, Kind::FromOwned | Kind::FromRef, TypeHint::Tuple | TypeHint::Unspecified | TypeHint::Unit) =>
            if f.attrs.has_parent_attr(&ctx.struct_attr.ty) {
                match (ctx.kind.is_ref(), ctx.fallible) {
                    (true, true) => quote!(value.try_into()?,),
                    (true, false) => quote!(value.into(),),
                    (false, true) => quote!((&value).try_into()?,),
                    (false, false) => quote!((&value).into(),),
                }
            } else {
                let field_path = if ctx.impl_type.is_variant() { get_field_path(&Named(format_ident!("f{}", index.index))) } else { get_field_path(&f.member) };
                quote!(#obj #field_path,)
            },
        (Unnamed(_), None, _, TypeHint::Struct) =>
            if f.attrs.has_parent_attr(&ctx.struct_attr.ty) {
                match (ctx.kind.is_ref(), ctx.fallible) {
                    (true, true) => quote!(value.try_into()?,),
                    (true, false) => quote!(value.into(),),
                    (false, true) => quote!((&value).try_into()?,),
                    (false, false) => quote!((&value).into(),),
                }
            } else {
                unreachable!("6")
            },
        (Named(_), Some(attr), Kind::OwnedInto | Kind::RefInto, TypeHint::Struct | TypeHint::Unspecified) => {
            let field_name = attr.get_field_name_or(&f.member);
            let field_path = get_child_field_path(&f.member);
            let right_side = attr.get_action_or(Some(&field_path), ctx, || quote!(#obj #field_path));
            if ctx.has_post_init { quote!(obj.#field_name = #right_side;) } else { quote!(#field_name: #right_side,) }
        },
        (Named(_), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Struct | TypeHint::Unspecified) => {
            let left_field_path = get_field_path(attr.get_field_name_or(&f.member));
            let right_field_path = get_child_field_path(&f.member);
            let right_side = attr.get_action_or(Some(&right_field_path), ctx, || quote!(#obj #right_field_path));
            quote!(other.#left_field_path = #right_side;)
        },
        (Named(_), Some(attr), Kind::OwnedInto | Kind::RefInto, TypeHint::Tuple) => {
            let right_field_path = get_child_field_path(&f.member);
            let right_side = attr.get_action_or(Some(&right_field_path), ctx, || quote!(#obj #right_field_path));
            quote!(#right_side,)
        },
        (Named(_), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Tuple) => {
            let left_field_path = get_field_path(&Unnamed(Index { index: idx as u32, span: Span::call_site() }));
            let right_field_path = get_child_field_path(&f.member);
            let right_side = attr.get_action_or(Some(&right_field_path), ctx, || quote!(#obj #right_field_path));
            quote!(other.#left_field_path = #right_side;)
        },
        (Named(_), Some(attr), Kind::FromOwned | Kind::FromRef, TypeHint::Struct | TypeHint::Unspecified | TypeHint::Unit) => {
            let right_side = attr.get_stuff(&obj, get_field_path, ctx, || &f.member);
            let idnt = parent_child.map_or(&f.member, |g| &g.this_member);
            quote!(#idnt: #right_side,)
        },
        (Named(ident), Some(attr), Kind::FromOwned | Kind::FromRef, TypeHint::Tuple) => {
            let or = Named(format_ident!("f{}", f.idx));
            let right_side = attr.get_stuff(&obj, get_field_path, ctx, || if ctx.impl_type.is_variant() { &or } else { &f.member });
            quote!(#ident: #right_side,)
        },
        (Unnamed(index), Some(attr), Kind::OwnedInto | Kind::RefInto, TypeHint::Tuple | TypeHint::Unspecified) => {
            let index = if ctx.impl_type.is_variant() { &Member::Named(format_ident!("f{}", index.index)) } else { &f.member };
            let field_path = get_child_field_path(index);
            let right_side = attr.get_action_or(Some(&field_path), ctx, || quote!(#obj #field_path));
            quote!(#right_side,)
        },
        (Unnamed(_), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Tuple | TypeHint::Unspecified) => {
            let left_field_path = get_field_path(attr.get_field_name_or(&f.member));
            let right_field_path = get_child_field_path(&f.member);
            let right_side = attr.get_action_or(Some(&right_field_path), ctx, || quote!(#obj #right_field_path));
            quote!(other.#left_field_path = #right_side;)
        },
        (Unnamed(index), Some(attr), Kind::OwnedInto | Kind::RefInto, TypeHint::Struct) => {
            let field_name = attr.get_ident();
            let field_path = get_child_field_path(&f.member);
            let or = if ctx.impl_type.is_variant() { format_ident!("f{}", index.index).to_token_stream() } else { field_path };
            let right_side = attr.get_action_or(Some(&or), ctx, || quote!(#obj #or));
            if ctx.has_post_init {
                quote!(obj.#field_name = #right_side;)
            } else {
                quote!(#field_name: #right_side,)
            }
        },
        (Unnamed(_), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Struct) => {
            let left_field_path = get_field_path(attr.get_ident());
            let right_field_path = get_child_field_path(&f.member);
            let right_side = attr.get_action_or(Some(&right_field_path), ctx, || quote!(#obj #right_field_path));
            quote!(other.#left_field_path = #right_side;)
        },
        (Unnamed(index), Some(attr), Kind::FromOwned | Kind::FromRef, _) => {
            let or = Named(format_ident!("f{}", index.index));
            let right_side = attr.get_stuff(&obj, get_field_path, ctx, || if ctx.impl_type.is_variant() { &or } else { &f.member });
            quote!(#right_side,)
        },
        (_, _, Kind::OwnedInto | Kind::RefInto | Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Unit) => TokenStream::new(),
    }
}

fn render_enum_line(v: &Variant, ctx: &ImplContext) -> TokenStream {
    let attr = v.attrs.applicable_attr(&ctx.kind, ctx.fallible, &ctx.struct_attr.ty);
    let lit = v.attrs.lit(&ctx.struct_attr.ty);
    let pat = v.attrs.pat(&ctx.struct_attr.ty);
    let var = v.attrs.type_hint(&ctx.struct_attr.ty);

    let src = ctx.src_ty;
    let dst = ctx.dst_ty;

    let ident = &v.ident;

    let variant_struct: Struct<'_> = Struct {
        attrs: DataTypeAttrs { ghosts_attrs: v.attrs.ghosts_attrs.clone(), ..Default::default() },
        ident,
        generics: &Default::default(),
        fields: v.fields.clone(),
        named_fields: v.named_fields,
        unit: v.unit,
    };

    let mut struct_attr = ctx.struct_attr.clone();
    let type_hint = var.map_or(TypeHint::Unspecified, |x| x.type_hint);
    struct_attr.type_hint = type_hint;

    let new_ctx = ImplContext {
        input: &DataType::Struct(&variant_struct),
        struct_attr: &struct_attr,
        impl_type: ImplType::Variant,
        ..*ctx
    };

    let empty_fields = variant_struct.fields.is_empty();
    let destr = if empty_fields && (!new_ctx.kind.is_from() || type_hint.maybe(TypeHint::Unit)) {
        TokenStream::new()
    } else if empty_fields && new_ctx.kind.is_from() && type_hint == TypeHint::Tuple {
        quote!((..))
    } else if empty_fields && new_ctx.kind.is_from() && type_hint == TypeHint::Struct {
        quote!({ .. })
    } else {
        variant_destruct_block(&variant_struct, &new_ctx)
    };

    let init = if attr.as_ref().is_some_and(|x| x.has_action()) || empty_fields && type_hint.maybe(TypeHint::Unit) {
        TokenStream::new()
    } else {
        struct_init_block(&variant_struct, &new_ctx)
    };

    match (v.named_fields, attr, lit, pat, &ctx.kind) {
        (_, None, None, None, _) => {
            quote!(#src::#ident #destr => #dst::#ident #init,)
        },
        (_, Some(attr), None, None, Kind::FromOwned | Kind::FromRef) => {
            let member = Named(ident.clone());
            let right_side = attr.get_action_or(Some(&quote!(#ident)), ctx, || quote!(#dst::#ident #init));
            let ident2 = attr.get_field_name_or(&member);
            quote!(#src::#ident2 #destr => #right_side,)
        },
        (_, Some(attr), None, None, Kind::OwnedInto | Kind::RefInto) => {
            let member = Named(ident.clone());
            let right_side = attr.get_stuff(&quote!(#dst::), |x| quote!(#x #init), ctx, || &member);
            quote!(#src::#ident #destr => #right_side,)
        },
        (_, None, Some(lit), None, Kind::FromOwned | Kind::FromRef) => {
            let left_side = &lit.tokens;
            quote!(#left_side => #dst::#ident #init,)
        },
        (_, None, Some(lit), None, Kind::OwnedInto | Kind::RefInto) => {
            let right_side = &lit.tokens;
            quote!(#src::#ident #destr => #right_side,)
        },
        (_, None, None, Some(pat), Kind::FromOwned | Kind::FromRef) => {
            let left_side = &pat.tokens;
            quote!(#left_side => #dst::#ident #init,)
        },
        (_, Some(attr), None, Some(_), Kind::OwnedInto | Kind::RefInto) => {
            let right_side = attr.get_action_or(None, ctx, TokenStream::new);
            quote!(#src::#ident #destr => #right_side,)
        }
        _ => todo!(),
    }
}

fn render_ghost_line(ghost_data: &GhostData, ctx: &ImplContext) -> TokenStream {
    let ch = match &ghost_data.child_path {
        Some(ghost_data) => {
            let ch = ghost_data.child_path.to_token_stream();
            quote!(#ch.)
        }
        None => TokenStream::new(),
    };
    let right_side = quote_action(&ghost_data.action, None, ctx);
    let ghost_ident = &ghost_data.ghost_ident.get_ident();
    match (ghost_ident, &ctx.kind) {
        (Named(ident), Kind::OwnedInto | Kind::RefInto) => quote!(#ident: #right_side,),
        (Unnamed(_), Kind::OwnedInto | Kind::RefInto) => quote!(#right_side,),
        (Named(ident), Kind::OwnedIntoExisting | Kind::RefIntoExisting) => quote!(other.#ch #ident = #right_side;),
        (Unnamed(index), Kind::OwnedIntoExisting | Kind::RefIntoExisting) => quote!(other.#ch #index = #right_side;),
        (_, _) => unreachable!("7"),
    }
}

fn render_enum_ghost_line(ghost_data: &GhostData, ctx: &ImplContext) -> TokenStream {
    let src = ctx.src_ty;
    let right_side = quote_action(&ghost_data.action, None, ctx);

    match &ghost_data.ghost_ident {
        GhostIdent::Member(ghost_ident) => match (ghost_ident, ctx.kind.is_from()) {
            (Unnamed(_), _) => unreachable!("17"),
            (Named(ident), true) => quote!(#src::#ident => #right_side,),
            (_, false) => TokenStream::new(),
        },
        GhostIdent::Destruction(destr) => {
            if ctx.kind.is_from() {
                quote!(#src::#destr => #right_side,)
            } else {
                TokenStream::new()
            }
        }
    }
}

fn replace_tilde_or_at_in_expr(input: &TokenStream, at_tokens: Option<&TokenStream>, tilde_tokens: Option<&TokenStream>) -> TokenStream {
    let mut tokens = Vec::new();

    input.clone().into_iter().for_each(|x| {
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

fn quote_action(action: &TokenStream, tilde_postfix: Option<&TokenStream>, ctx: &ImplContext) -> TokenStream {
    let dst = ctx.dst_ty;
    let ident = match ctx.kind {
        Kind::FromOwned | Kind::FromRef => quote!(value),
        _ => quote!(self),
    };
    let path = match ctx.impl_type {
        ImplType::Struct => quote!(#ident.#tilde_postfix),
        ImplType::Enum => quote!(#dst::#tilde_postfix),
        ImplType::Variant => quote!(#tilde_postfix),
    };
    replace_tilde_or_at_in_expr(action, Some(&ident), Some(&path))
}

struct QuoteTraitParams<'a> {
    pub attr: Option<&'a TokenStream>,
    pub impl_attr: Option<&'a TokenStream>,
    pub inner_attr: Option<&'a TokenStream>,
    pub dst: &'a TokenStream,
    pub src: &'a TokenStream,
    pub these_gens: TokenStream,
    pub those_gens: TokenStream,
    pub impl_gens: TokenStream,
    pub where_clause: Option<TokenStream>,
    pub r: Option<TokenStream>,
}

fn get_quote_trait_params<'a>(input: &DataType, ctx: &'a ImplContext) -> QuoteTraitParams<'a> {
    let mut impl_gens = input.get_generics().clone();

    let these_lts: Vec<&Lifetime> = input.get_generics().params.iter().filter_map(|g| match g {
        GenericParam::Lifetime(l) => Some(&l.lifetime),
        _ => None
    }).collect();

    let those_lts: Vec<&Lifetime> = ctx.struct_attr.ty.generics.as_ref().map(|g| g.args.iter().filter_map(|g| match g {
        GenericArgument::Lifetime(l) => Some(l),
        _ => None
    }).collect()).unwrap_or_default();

    let ref_lts = ctx.kind.is_ref().then_some(
        if ctx.kind.is_from() { these_lts } else { those_lts.clone() }
    ).unwrap_or_default();

    for lt in those_lts {
        let missing_lt = impl_gens.params.iter().all(|param| {
            if let GenericParam::Lifetime(param) = param {
                &param.lifetime != lt
            } else { false }
        });

        if missing_lt {
            let gen = GenericArgument::Lifetime(lt.clone());
            impl_gens.params.push(parse_quote!(#gen));
        }
    }

    if !ref_lts.is_empty() {
        impl_gens.params.push(parse_quote!('o2o: #( #ref_lts )+*));
    }

    QuoteTraitParams { 
        attr: ctx.struct_attr.attribute.as_ref(), 
        impl_attr: ctx.struct_attr.impl_attribute.as_ref(), 
        inner_attr: ctx.struct_attr.inner_attribute.as_ref(), 
        dst: ctx.dst_ty, 
        src: ctx.src_ty, 
        these_gens: input.get_generics().to_token_stream(),
        those_gens: ctx.struct_attr.ty.generics.to_token_stream(),
        impl_gens: impl_gens.to_token_stream(), 
        where_clause: input.get_attrs().where_attr(&ctx.struct_attr.ty).map(|x| {
            let where_clause = &x.where_clause;
            quote!(where #where_clause)
        }), 
        r: ctx.kind.is_ref().then_some(if ref_lts.is_empty() { quote!(&) } else { quote!(&'o2o) }) 
    }
}

fn quote_trait(input: &DataType, ctx: &mut ImplContext) -> TokenStream {
    let pre_init = struct_pre_init(ctx);
    let post_init = if ctx.kind.is_from() { None } else {
        struct_post_init(input, ctx)
    };
    ctx.has_post_init = post_init.is_some();

    match (ctx.kind, ctx.fallible) {
        (Kind::FromOwned, false) | (Kind::FromRef, false) => quote_from_trait(input, ctx, pre_init, main_code_block(ctx)),
        (Kind::FromOwned, true) | (Kind::FromRef, true) => quote_try_from_trait(input, ctx, pre_init, main_code_block_ok(ctx)),
        (Kind::OwnedInto, false) | (Kind::RefInto, false) => quote_into_trait(input, ctx, pre_init, main_code_block(ctx), post_init),
        (Kind::OwnedInto, true) | (Kind::RefInto, true) => quote_try_into_trait(input, ctx, pre_init, main_code_block_ok(ctx), post_init),
        (Kind::OwnedIntoExisting, false) | (Kind::RefIntoExisting, false) => quote_into_existing_trait(input, ctx, pre_init, main_code_block(ctx), post_init),
        (Kind::OwnedIntoExisting, true) | (Kind::RefIntoExisting, true) => quote_try_into_existing_trait(input, ctx, pre_init, main_code_block(ctx), post_init),
    }
}

fn quote_from_trait(input: &DataType, ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream) -> TokenStream {
    let QuoteTraitParams { attr, impl_attr, inner_attr, dst, src, these_gens, those_gens, impl_gens, where_clause, r } = get_quote_trait_params(input, ctx);
    quote! {
        #impl_attr
        impl #impl_gens ::core::convert::From<#r #src #those_gens> for #dst #these_gens #where_clause {
            #attr
            fn from(value: #r #src #those_gens) -> #dst #these_gens {
                #inner_attr
                #pre_init
                #init
            }
        }
    }
}

fn quote_try_from_trait(input: &DataType, ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream) -> TokenStream {
    let QuoteTraitParams { attr, impl_attr, inner_attr, dst, src, these_gens, those_gens, impl_gens, where_clause, r } = get_quote_trait_params(input, ctx);
    let err_ty = &ctx.struct_attr.err_ty.as_ref().unwrap().path;
    quote! {
        #impl_attr
        impl #impl_gens ::core::convert::TryFrom<#r #src #those_gens> for #dst #these_gens #where_clause {
            type Error = #err_ty;
            #attr
            fn try_from(value: #r #src #those_gens) -> Result<#dst #these_gens, #err_ty> {
                #inner_attr
                #pre_init
                #init
            }
        }
    }
}

fn quote_into_trait(input: &DataType, ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream, post_init: Option<TokenStream>) -> TokenStream {
    let QuoteTraitParams { attr, impl_attr, inner_attr, dst, src, these_gens, those_gens, impl_gens, where_clause, r } = get_quote_trait_params(input, ctx);

    let body = match post_init {
        Some(post_init) => quote! {
            let mut obj: #dst = Default::default();
            #init
            #post_init
            obj
        },
        None => quote! {
            #pre_init
            #init
        },
    };

    quote! {
        #impl_attr
        impl #impl_gens ::core::convert::Into<#dst #those_gens> for #r #src #these_gens #where_clause {
            #attr
            fn into(self) -> #dst #those_gens {
                #inner_attr
                #body
            }
        }
    }
}

fn quote_try_into_trait(input: &DataType, ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream, post_init: Option<TokenStream>) -> TokenStream {
    let QuoteTraitParams { attr, impl_attr, inner_attr, dst, src, these_gens, those_gens, impl_gens, where_clause, r } = get_quote_trait_params(input, ctx);
    let err_ty = &ctx.struct_attr.err_ty.as_ref().unwrap().path;

    let body = match post_init {
        Some(post_init) => quote! {
            let mut obj: #dst = Default::default();
            #init
            #post_init
            Ok(obj)
        },
        None => quote! {
            #pre_init
            #init
        },
    };

    quote! {
        #impl_attr
        impl #impl_gens ::core::convert::TryInto<#dst #those_gens> for #r #src #these_gens #where_clause {
            type Error = #err_ty;
            #attr
            fn try_into(self) -> Result<#dst #those_gens, #err_ty> {
                #inner_attr
                #body
            }
        }
    }
}

fn quote_into_existing_trait(input: &DataType, ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream, post_init: Option<TokenStream>) -> TokenStream {
    let QuoteTraitParams { attr, impl_attr, inner_attr, dst, src, these_gens, those_gens, impl_gens, where_clause, r } = get_quote_trait_params(input, ctx);
    quote! {
        #impl_attr
        impl #impl_gens o2o::traits::IntoExisting<#dst #those_gens> for #r #src #these_gens #where_clause {
            #attr
            fn into_existing(self, other: &mut #dst #those_gens) {
                #inner_attr
                #pre_init
                #init
                #post_init
            }
        }
    }
}

fn quote_try_into_existing_trait(input: &DataType, ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream, post_init: Option<TokenStream>) -> TokenStream {
    let QuoteTraitParams { attr, impl_attr, inner_attr, dst, src, these_gens, those_gens, impl_gens, where_clause, r } = get_quote_trait_params(input, ctx);
    let err_ty = &ctx.struct_attr.err_ty.as_ref().unwrap().path;
    quote! {
        #impl_attr
        impl #impl_gens o2o::traits::TryIntoExisting<#dst #those_gens> for #r #src #these_gens #where_clause {
            type Error = #err_ty;
            #attr
            fn try_into_existing(self, other: &mut #dst #those_gens) -> Result<(), #err_ty> {
                #inner_attr
                #pre_init
                #init
                #post_init
                Ok(())
            }
        }
    }
}

impl<'a> ApplicableAttr<'a> {
    fn get_ident(&'a self) -> &'a Member {
        match self {
            ApplicableAttr::Field(MemberAttrCore { member, .. }) => match member {
                Some(val) => val,
                None => unreachable!("8"),
            },
            ApplicableAttr::ParentChildField(p, kind) => {
                let attr = p.get_for_kind(kind);
                match attr.as_ref() {
                    Some(attr) => match attr.that_member.as_ref() {
                        Some(val) => val,
                        None => unreachable!("18"),
                    },
                    None => unreachable!("19")
                }
            }
            ApplicableAttr::Ghost(_) => unreachable!("9"),
        }
    }

    fn has_action(&self) -> bool {
        match self {
            ApplicableAttr::Field(f) => f.action.is_some(),
            ApplicableAttr::Ghost(g) => g.action.is_some(),
            ApplicableAttr::ParentChildField(p, kind) => p.get_for_kind(kind).is_some_and(|x| x.action.is_some()),
        }
    }

    fn get_field_name_or(&'a self, field: &'a Member) -> &'a Member {
        match self {
            ApplicableAttr::Field(MemberAttrCore { member, .. }) => match member {
                Some(val) => val,
                None => field,
            },
            ApplicableAttr::Ghost(_) => unreachable!("10"),
            ApplicableAttr::ParentChildField(p, kind) => {
                let attr = p.get_for_kind(kind);

                match attr.as_ref() {
                    Some(attr) =>  match attr.that_member.as_ref() {
                        Some(val) => val,
                        None => &p.this_member,
                    },
                    None => &p.this_member
                }
            }
        }
    }

    fn get_action_or<F: Fn() -> TokenStream>(&self, field_path: Option<&TokenStream>, ctx: &ImplContext, or: F) -> TokenStream {
        match self {
            ApplicableAttr::Field(MemberAttrCore { action, .. }) => match action {
                Some(val) => quote_action(val, field_path, ctx),
                None => or(),
            },
            ApplicableAttr::ParentChildField(p, kind) => {
                let attr = p.get_for_kind(kind);

                match attr.as_ref() {
                    Some(attr) => match attr.action.as_ref() {
                        Some(val) => quote_action(val, field_path, ctx),
                        None => or()
                    },
                    None => or()
                }
            }
            ApplicableAttr::Ghost(_) => unreachable!("11"),
        }
    }

    fn get_stuff<F1: Fn(&Member) -> TokenStream, F2: Fn() -> &'a Member>(&self, obj: &TokenStream, field_path: F1, ctx: &ImplContext, or: F2) -> TokenStream {
        let get_stuff = |member: &Option<Member>, action: &Option<TokenStream>| {
            match (member, action) {
                (Some(ident), Some(action)) => if let Unnamed(index) = ident {
                        if ctx.impl_type.is_variant() {
                            let ident = Named(format_ident!("f{}", index.index));
                            quote_action(action, Some(&field_path(&ident)), ctx)
                        } else {
                            quote_action(action, Some(&field_path(ident)), ctx)
                        }
                    } else {
                        quote_action(action, Some(&field_path(ident)), ctx)
                    },
                (Some(ident), None) => {
                    let field_path = field_path(ident);
                    quote!(#obj #field_path)
                }
                (None, Some(action)) => quote_action(action, Some(&field_path(or())), ctx),
                _ => unreachable!("12"),
            }
        };
        match self {
            ApplicableAttr::Field(MemberAttrCore { member, action, .. }) => get_stuff(member, action),
            ApplicableAttr::ParentChildField(p, kind) => {
                let attr = p.get_for_kind(kind);

                if attr.is_some_and(|x|x.that_member.is_some()) {
                    let attr = attr.unwrap();
                    get_stuff(&attr.that_member, &attr.action)
                } else {
                    get_stuff(&Some(p.this_member.clone()), attr.map_or(&None, |x| &x.action))
                }
            },
            ApplicableAttr::Ghost(ghost_attr) => quote_action(ghost_attr.action.as_ref().unwrap(), None, ctx),
        }
    }
}
