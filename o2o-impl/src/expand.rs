use std::{slice::Iter, iter::Peekable, collections::HashSet};

use crate::{
    ast::{DataType, DataTypeMember, Enum, Field, Struct, Variant}, 
    attr::{ApplicableAttr, ChildData, ChildPath, GhostData, InitData, Kind, TraitAttrCore, DataTypeAttrs, StructKindHint}, validate::validate
};
use proc_macro2::{TokenStream, Span};
use syn::{punctuated::Punctuated, Data, DeriveInput, Error, Generics, Index, Member, Result, Token};
use quote::{format_ident, quote, ToTokens};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    match &node.data {
        Data::Struct(data) => {
            let input = Struct::from_syn(node, data)?;
            let input = DataType::Struct(&input);
            validate(&input)?;
            Ok(struct_impl(input))
        },
        Data::Enum(data) => {
            let input = Enum::from_syn(node, data)?;
            let input = DataType::Enum(&input);
            validate(&input)?;
            Ok(struct_impl(input))
        },
        _ => Err(Error::new_spanned(
            node,
            "#[derive(o2o)] only supports structs and enums.",
        ))
    }
}

#[derive(Clone)]
struct ImplContext<'a> {
    input: &'a DataType<'a>,
    struct_attr: &'a TraitAttrCore,
    kind: Kind,
    dst_ty: &'a TokenStream,
    src_ty: &'a TokenStream,
    has_post_init: bool,
    destructured_src: bool,
}

fn struct_impl(input: DataType) -> TokenStream {
    let ty = input.get_ident().to_token_stream();
    let attrs = input.get_attrs();

    let from_owned_impls = attrs.iter_for_kind(&Kind::FromOwned).map(|struct_attr| {
        let ctx = ImplContext {
            input: &input,
            struct_attr,
            kind: Kind::FromOwned,
            dst_ty: &ty,
            src_ty: &struct_attr.ty.path,
            has_post_init: false,
            destructured_src: false
        };

        let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
        if let Some(quick_return) = &struct_attr.quick_return {
            return quote_from_trait(&ctx, pre_init, quote_action(quick_return, None, &ctx))
        }
        quote_from_trait(&ctx, pre_init, main_code_block(&ctx))
    });

    let from_ref_impls = attrs.iter_for_kind(&Kind::FromRef).map(|struct_attr| {
        let ctx = ImplContext {
            input: &input,
            struct_attr,
            kind: Kind::FromRef,
            dst_ty: &ty,
            src_ty: &struct_attr.ty.path,
            has_post_init: false,
            destructured_src: false
        };

        let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
        if let Some(quick_return) = &struct_attr.quick_return {
            return quote_from_trait(&ctx, pre_init, quote_action(quick_return, None, &ctx))
        }
        quote_from_trait(&ctx, pre_init, main_code_block(&ctx))
    });

    let owned_into_impls = attrs.iter_for_kind(&Kind::OwnedInto).map(|struct_attr| {
        let mut ctx = ImplContext {
            input: &input,
            struct_attr,
            kind: Kind::OwnedInto,
            dst_ty: &struct_attr.ty.path,
            src_ty: &ty,
            has_post_init: false,
            destructured_src: false
        };

        let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
        if let Some(quick_return) = &struct_attr.quick_return {
            return quote_into_trait(&ctx, pre_init, quote_action(quick_return, None, &ctx), None)
        }
        let post_init = struct_post_init(&ctx);
        ctx.has_post_init = post_init.is_some();
        quote_into_trait(&ctx, pre_init, main_code_block(&ctx), post_init)
    });

    let ref_into_impls = attrs.iter_for_kind(&Kind::RefInto).map(|struct_attr| {
        let mut ctx = ImplContext {
            input: &input,
            struct_attr,
            kind: Kind::RefInto,
            dst_ty: &struct_attr.ty.path,
            src_ty: &ty,
            has_post_init: false,
            destructured_src: false
        };

        let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
        if let Some(quick_return) = &struct_attr.quick_return {
            return quote_into_trait(&ctx, pre_init, quote_action(quick_return, None, &ctx), None)
        }
        let post_init = struct_post_init(&ctx);
        ctx.has_post_init = post_init.is_some();
        quote_into_trait(&ctx, pre_init, main_code_block(&ctx), post_init)
    });

    let owned_into_existing_impls = attrs.iter_for_kind(&Kind::OwnedIntoExisting).map(|struct_attr| {
        let mut ctx = ImplContext {
            input: &input,
            struct_attr,
            kind: Kind::OwnedIntoExisting,
            dst_ty: &struct_attr.ty.path,
            src_ty: &ty,
            has_post_init: false,
            destructured_src: false
        };
        let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
        if let Some(quick_return) = &struct_attr.quick_return {
            let action = quote_action(quick_return, None, &ctx);
            return quote_into_existing_trait(&ctx, pre_init, quote!(*other = #action;), None)
        }
        let post_init = struct_post_init(&ctx);
        ctx.has_post_init = post_init.is_some();
        quote_into_existing_trait(&ctx, pre_init, main_code_block(&ctx), post_init)
    });

    let ref_into_existing_impls = attrs.iter_for_kind(&Kind::RefIntoExisting).map(|struct_attr| {
        let mut ctx = ImplContext {
            input: &input,
            struct_attr,
            kind: Kind::RefIntoExisting,
            dst_ty: &struct_attr.ty.path,
            src_ty: &ty,
            has_post_init: false,
            destructured_src: false
        };
        let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
        if let Some(quick_return) = &struct_attr.quick_return {
            let action = quote_action(quick_return, None, &ctx);
            return quote_into_existing_trait(&ctx, pre_init, quote!(*other = #action;), None)
        }
        let post_init = struct_post_init(&ctx);
        ctx.has_post_init = post_init.is_some();
        quote_into_existing_trait(&ctx, pre_init, main_code_block(&ctx), post_init)
    });

    let result = quote! {
        #(#from_owned_impls)*
        #(#from_ref_impls)*
        #(#owned_into_impls)*
        #(#ref_into_impls)*
        #(#owned_into_existing_impls)*
        #(#ref_into_existing_impls)*
    };

    result
}

fn main_code_block(ctx: &ImplContext) -> TokenStream {
    match ctx.input {
        DataType::Struct(_) => {
            let struct_init_block = struct_init_block(ctx);

            match ctx.kind {
                Kind::FromOwned | Kind::FromRef => {
                    let dst = ctx.dst_ty;
                    quote!(#dst #struct_init_block)
                },
                Kind::OwnedInto | Kind::RefInto => {
                    let dst = if ctx.struct_attr.ty.nameless_tuple || ctx.has_post_init { TokenStream::new() } else { ctx.dst_ty.clone() };
                    quote!(#dst #struct_init_block)
                },
                Kind::OwnedIntoExisting | Kind::RefIntoExisting => struct_init_block,
            }
        },
        DataType::Enum(_) => {
            let struct_init_block = struct_init_block(ctx);

            match ctx.kind {
                Kind::FromOwned | Kind::FromRef => {
                    quote!(match value #struct_init_block)
                },
                Kind::OwnedInto | Kind::RefInto => {
                    quote!(match self #struct_init_block)
                },
                Kind::OwnedIntoExisting | Kind::RefIntoExisting => struct_init_block,
            }
        }
    }
}

fn struct_init_block<'a>(ctx: &'a ImplContext) -> TokenStream {
    let attrs = ctx.input.get_attrs();
    let members = ctx.input.get_members();

    let mut current_path = "";
    let mut group_counter =  0;
    let mut unique_paths = HashSet::<&str>::new();

    let mut make_tuple = |path, stuff: FieldData<'a>| {
        if path != current_path {
            group_counter += 1;
            current_path = path;
        }
        (group_counter, path, stuff)
    };

    let mut fields: Vec<(usize, &str, FieldData)> = vec![];

    fields.extend(members.iter()
    .map(|x| {
        let path = x.get_attrs().child(&ctx.struct_attr.ty).map(|x| x.get_child_path_str(None)).unwrap_or("");
        unique_paths.insert(path);
        make_tuple(path, FieldData::Field(*x))
    })
    .collect::<Vec<(usize, &str, FieldData)>>());
    
    fields.extend(attrs.ghost_attrs.iter()
        .flat_map(|x| &x.attr.ghost_data)
        .filter(|x| unique_paths.insert(x.get_child_path_str(None)))
        .map(|x| {
            let path: &str = x.get_child_path_str(None);
            make_tuple(path, FieldData::GhostData(x))
        }));

    fields.sort_by(|(ga, a, _), (gb, b, _)| ga.cmp(gb).then(a.cmp(b)));

    struct_init_block_inner(&mut fields.iter().peekable(), ctx, None)
}

enum FieldData<'a> {
    Field(DataTypeMember<'a>),
    GhostData(&'a GhostData)
}

fn struct_init_block_inner(
    members: &mut Peekable<Iter<(usize, &str, FieldData)>>,
    ctx: &ImplContext,
    field_ctx: Option<(&ChildPath, Option<&ChildData>, usize)>
) -> TokenStream
{
    let next = members.peek();
    if next.is_none() {
        return quote!({})
    }

    let struct_kind_hint = match field_ctx {
        Some(field_ctx) => match field_ctx.1 {
            Some(child_data) => child_data.struct_kind_hint,
            None => ctx.struct_attr.struct_kind_hint
        },
        None => ctx.struct_attr.struct_kind_hint
    };

    let mut fragments: Vec<TokenStream> = vec![];
    let mut idx: usize = 0;

    while let Some((_, key, field_data)) = members.peek() {
        if let Some(field_ctx) = field_ctx {
            if !key.starts_with(field_ctx.0.get_child_path_str(Some(field_ctx.2))) {
                break;
            }
        }

        match field_data {
            FieldData::Field(f) => {
                let attrs = f.get_attrs();
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
                    Some(child_attr) => 
                        match f {
                            DataTypeMember::Field(f) => render_child_fragment(&child_attr.child_path, members, ctx, field_ctx, struct_kind_hint, || render_struct_line(f, ctx, struct_kind_hint, idx)),
                            DataTypeMember::Variant(_) => todo!(),
                        }
                    None => {
                        members.next();
                        match f {
                            DataTypeMember::Field(f) => render_struct_line(f, ctx, struct_kind_hint, idx),
                            DataTypeMember::Variant(v) => render_enum_line(v, ctx, struct_kind_hint, idx),
                        }
                    }
                };
                fragments.push(fragment);
                idx += 1;
            },
            FieldData::GhostData(ghost_data) => {
                let child_path = &ghost_data.child_path.as_ref().unwrap();
                let fragment = render_child_fragment(child_path, members, ctx, field_ctx, struct_kind_hint, TokenStream::new);

                fragments.push(fragment);
                idx += 1;
            }
        }
    }

    if !ctx.kind.is_from() {
        if let Some(ghost_attr) = ctx.input.get_attrs().ghost_attr(&ctx.struct_attr.ty, &ctx.kind) {
            ghost_attr.ghost_data.iter().for_each(|x| {
                match (&x.child_path, field_ctx) {
                    (Some(_), Some(field_ctx)) => {
                        if x.get_child_path_str(None) == field_ctx.0.get_child_path_str(Some(field_ctx.2)) {
                            fragments.push(render_ghost_line(x, ctx))
                        }
                    },
                    (None, None) => fragments.push(render_ghost_line(x, ctx)),
                    _ => ()
                }
            });
        }
    }

    if let Some(update) = &ctx.struct_attr.update {
        let g = quote_action(update, None, ctx);
        fragments.push(quote!(..#g))
    }

    if ctx.has_post_init || ctx.kind.is_into_existing() {
        return quote!(#(#fragments)*)
    }

    match ctx.input {
        DataType::Struct(s) => {
            match (&ctx.kind, struct_kind_hint, s.named_fields) {
                (Kind::FromOwned | Kind::FromRef, _, true) => quote!({#(#fragments)*}),
                (Kind::FromOwned | Kind::FromRef, _, false) => quote!((#(#fragments)*)),
                (_, StructKindHint::Struct, _) => quote!({#(#fragments)*}),
                (_, StructKindHint::Tuple, _) => quote!((#(#fragments)*)),
                (_, StructKindHint::Unspecified, true) => quote!({#(#fragments)*}),
                (_, StructKindHint::Unspecified, false) => quote!((#(#fragments)*)),
            }
        },
        DataType::Enum(_) => {
            quote!({#(#fragments)*})
        },
    }
}

fn variant_destruct_block(ctx: &ImplContext<'_>) -> TokenStream {
    match ctx.input {
        DataType::Struct(s) => {
            if s.named_fields {
                let idents = s.fields.iter().map(|x| {
                    let attr = x.attrs.applicable_attr(&ctx.kind, &ctx.struct_attr.ty);

                    if !ctx.kind.is_from() || attr.is_none() {
                        let ident = &x.member;
                        quote!(#ident ,)
                    } else if let Some(attr) = attr {
                        let ident = attr.get_field_name_or(&x.member);
                        quote!(#ident ,)
                    } else { unreachable!() }
                });

                quote!({#(#idents)*})
            } else {
                let idents = s.fields.iter().map(|x| {
                    let ident = format_ident!("f{}", x.idx);
                    quote!(#ident ,)
                });

                quote!((#(#idents)*))
            }
        },
        DataType::Enum(_) => panic!("weird")
    }
}

fn render_child_fragment<F: Fn() -> TokenStream>(
    child_path: &ChildPath,
    fields: &mut Peekable<Iter<(usize, &str, FieldData)>>,
    ctx: &ImplContext,
    field_ctx: Option<(&ChildPath, Option<&ChildData>, usize)>,
    struct_kind_hint: StructKindHint,
    render_line: F
) -> TokenStream {
    if let Some(field_ctx) = field_ctx {
        if field_ctx.2 < child_path.child_path_str.len() - 1 {
            match ctx.kind {
                Kind::OwnedInto | Kind::RefInto => 
                    render_child(fields, ctx, (child_path, field_ctx.2 + 1), struct_kind_hint),
                Kind::FromOwned | Kind::FromRef => {
                    fields.next();
                    render_line()
                },
                Kind::OwnedIntoExisting | Kind::RefIntoExisting =>
                    render_existing_child(fields, ctx, (child_path, field_ctx.2 + 1)),
            }
        } else {
            fields.next();
            render_line() 
        }
    } else {
        match ctx.kind {
            Kind::OwnedInto | Kind::RefInto => 
                render_child(fields, ctx, (child_path, 0), struct_kind_hint),
            Kind::FromOwned | Kind::FromRef => {
                fields.next();
                render_line()
            },
            Kind::OwnedIntoExisting | Kind::RefIntoExisting =>
                render_existing_child(fields, ctx, (child_path, 0)),
        }
    }
}

fn struct_pre_init(ctx: &ImplContext, init_data: &Option<Punctuated<InitData, Token![,]>>) -> Option<TokenStream> {
    if let Some(init_data) = init_data {
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

fn struct_post_init(ctx: &ImplContext) -> Option<TokenStream> {
    let mut fragments: Vec<TokenStream> = vec![];

    ctx.input.get_members().iter().for_each(|f| {
        if !ctx.kind.is_from() && f.get_attrs().has_parent_attr(&ctx.struct_attr.ty) {
            match f {
                DataTypeMember::Field(f) => fragments.push(render_parent(f, ctx)),
                DataTypeMember::Variant(_) => todo!(),
            }
        }
    });

    if fragments.is_empty() {
        return None
    }
    Some(quote!(#(#fragments)*))
}

fn render_parent(f: &Field, ctx: &ImplContext) -> TokenStream {
    let member = &f.member;
    match &ctx.kind {
        Kind::OwnedIntoExisting => quote!(self.#member.into_existing(other);),
        Kind::RefIntoExisting => quote!((&(self.#member)).into_existing(other);),
        Kind::OwnedInto => quote!(self.#member.into_existing(&mut obj);),
        Kind::RefInto => quote!((&(self.#member)).into_existing(&mut obj);),
        _ => panic!("weird")
    }
}

fn render_child(
    fields: &mut Peekable<Iter<(usize, &str, FieldData)>>,
    ctx: &ImplContext,
    field_ctx: (&ChildPath, usize),
    hint: StructKindHint) -> TokenStream
{
    let input = match ctx.input {
        DataType::Struct(s) => *s,
        _ => panic!("weird")
    };

    let child_path = field_ctx.0;
    let path = child_path.get_child_path_str(Some(field_ctx.1));
    let child_name = child_path.child_path[field_ctx.1].to_token_stream();
    let mut children = input.attrs.children_attr(&ctx.struct_attr.ty).unwrap().children.iter();
    let child_data = children.find(|child_data| child_data.check_match(path)).unwrap();
    let ty = &child_data.ty;
    let init = struct_init_block_inner(fields, ctx, Some((field_ctx.0, Some(child_data), field_ctx.1 )));
    match (input.named_fields, hint) {
        (true, StructKindHint::Struct | StructKindHint::Unspecified) => quote!(#child_name: #ty #init,),
        (true, StructKindHint::Tuple) => quote!(#ty #init,),
        (false, StructKindHint::Tuple | StructKindHint::Unspecified) => quote!(#ty #init,),
        (false, StructKindHint::Struct) => quote!(#child_name: #ty #init,),
    }
}

fn render_existing_child(
    fields: &mut Peekable<Iter<(usize, &str, FieldData)>>,
    ctx: &ImplContext,
    field_ctx: (&ChildPath, usize)
) -> TokenStream
{
    let input = match ctx.input {
        DataType::Struct(s) => *s,
        _ => panic!("weird")
    };

    let child_attr = field_ctx.0;
    let path = child_attr.get_child_path_str(Some(field_ctx.1));
    let children_attr = input.attrs.children_attr(&ctx.struct_attr.ty);
    let child_data = children_attr.and_then(|x| 
        x.children.iter().find(|child_data| child_data.check_match(path)));
    struct_init_block_inner(fields, ctx, Some((field_ctx.0, child_data, field_ctx.1 )))
}

fn render_struct_line(
    f: &Field,
    ctx: &ImplContext, 
    hint: StructKindHint, 
    idx: usize
) -> TokenStream {
    let attr = f.attrs.applicable_attr(&ctx.kind, &ctx.struct_attr.ty);
    let get_field_path = |x: &Member| {
        match f.attrs.child(&ctx.struct_attr.ty) {
            Some(child_attr) => {
                let ch = child_attr.child_path.child_path.to_token_stream();
                quote!(#ch.#x)
            },
            None => x.to_token_stream()
        }
    };

    let obj = if ctx.destructured_src { TokenStream::new() } else {
        match ctx.kind {
            Kind::OwnedInto => quote!(self.),
            Kind::RefInto => quote!(self.),
            Kind::FromOwned => quote!(value.),
            Kind::FromRef => quote!(value.),
            Kind::OwnedIntoExisting => quote!(self.),
            Kind::RefIntoExisting => quote!(self.),
        }
    };

    match (&f.member, attr, &ctx.kind, hint) {
        (syn::Member::Named(ident), None, Kind::OwnedInto | Kind::RefInto, StructKindHint::Struct | StructKindHint::Unspecified) =>
            if ctx.has_post_init { quote!(obj.#ident = #obj #ident;) } else { quote!(#ident: #obj #ident,) },
        (syn::Member::Named(ident), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Struct | StructKindHint::Unspecified) => {
            let field_path = get_field_path(&f.member);
            quote!(other.#field_path = #obj #ident;)
        },
        (syn::Member::Named(ident), None, Kind::OwnedInto | Kind::RefInto, StructKindHint::Tuple) =>
            quote!(#obj #ident,),
        (syn::Member::Named(ident), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Tuple) => {
            let index = Member::Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
            quote!(other.#index = #obj #ident;)
        }
        (syn::Member::Named(ident), None, Kind::FromOwned | Kind::FromRef, StructKindHint::Struct | StructKindHint::Unspecified) =>
            if f.attrs.has_parent_attr(&ctx.struct_attr.ty) {
                if !ctx.kind.is_ref() { quote!(#ident: (&value).into(),) } else { quote!(#ident: value.into(),) }
            } else {
                let field_path = get_field_path(&f.member);
                quote!(#ident: #obj #field_path,)
            },
        (syn::Member::Named(ident), None, Kind::FromOwned | Kind::FromRef, StructKindHint::Tuple) => {
            let index = Member::Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
            let field_path = get_field_path(&index);
            quote!(#ident: #obj #field_path,)
        },
        (syn::Member::Unnamed(index), None, Kind::OwnedInto | Kind::RefInto, StructKindHint::Tuple | StructKindHint::Unspecified) =>
            if ctx.has_post_init { 
                let index2 = Member::Unnamed(Index { index: idx as u32, span: Span::call_site() });
                quote!(obj.#index2 = #obj #index;)
            } else { 
                let index = if ctx.destructured_src { format_ident!("f{}", index.index).to_token_stream() } else { index.to_token_stream() };
                quote!(#obj #index,)
            },
        (syn::Member::Unnamed(index), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Tuple | StructKindHint::Unspecified) => {
            let index2 = Member::Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
            quote!(other.#index2 = #obj #index;)
        }
        (syn::Member::Unnamed(index), None, Kind::FromOwned | Kind::FromRef, StructKindHint::Tuple | StructKindHint::Unspecified) =>
            if f.attrs.has_parent_attr(&ctx.struct_attr.ty) {
                if !ctx.kind.is_ref() { quote!((&value).into(),) } else { quote!(value.into(),) }
            } else {
                let field_path = if ctx.destructured_src { get_field_path(&Member::Named(format_ident!("f{}", index.index))) } else { get_field_path(&f.member) };
                quote!(#obj #field_path,)
            },
        (syn::Member::Unnamed(_), None, _, StructKindHint::Struct) => {
            if f.attrs.has_parent_attr(&ctx.struct_attr.ty) {
                if !ctx.kind.is_ref() { quote!((&value).into(),) } else { quote!(value.into(),) }
            } else {
                panic!("weird")
            }
        },
        (syn::Member::Named(ident), Some(attr), Kind::OwnedInto | Kind::RefInto, StructKindHint::Struct | StructKindHint::Unspecified) => {
            let field_name = attr.get_field_name_or(&f.member);
            let right_side = attr.get_action_or(Some(ident.to_token_stream()), ctx, || quote!(#obj #ident));
            if ctx.has_post_init { quote!(obj.#field_name = #right_side;) } else { quote!(#field_name: #right_side,) }
        },
        (syn::Member::Named(ident), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Struct | StructKindHint::Unspecified) => {
            let field_path = get_field_path(attr.get_field_name_or(&f.member));
            let right_side = attr.get_action_or(Some(ident.to_token_stream()), ctx, || quote!(#obj #ident));
            quote!(other.#field_path = #right_side;)
        },
        (syn::Member::Named(ident), Some(attr), Kind::OwnedInto | Kind::RefInto, StructKindHint::Tuple) => {
            let right_side = attr.get_action_or(Some(get_field_path(&f.member)), ctx, || quote!(#obj #ident));
            quote!(#right_side,)
        },
        (syn::Member::Named(ident), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Tuple) => {
            let field_path = get_field_path(&Member::Unnamed(Index { index: idx as u32, span: Span::call_site() }));
            let right_side = attr.get_action_or(Some(ident.to_token_stream()), ctx, || quote!(#obj #ident));
            quote!(other.#field_path = #right_side;)
        },
        (syn::Member::Named(ident), Some(attr), Kind::FromOwned | Kind::FromRef, _) => {
            let right_side = attr.get_stuff(&obj, get_field_path, ctx, || &f.member);
            quote!(#ident: #right_side,)
        },
        (syn::Member::Unnamed(index), Some(attr), Kind::OwnedInto | Kind::RefInto, StructKindHint::Tuple | StructKindHint::Unspecified) => {
            let index = if ctx.destructured_src { Some(format_ident!("f{}", index.index).to_token_stream()) } else { Some(index.to_token_stream()) };
            let right_side = attr.get_action_or(index.clone(), ctx, || quote!(#obj #index));
            quote!(#right_side,)
        },
        (syn::Member::Unnamed(index), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Tuple | StructKindHint::Unspecified) => {
            let field_path = get_field_path(attr.get_field_name_or(&f.member));
            let right_side = attr.get_action_or(Some(index.to_token_stream()), ctx, || quote!(#obj #index));
            quote!(other.#field_path = #right_side;)
        },
        (syn::Member::Unnamed(index), Some(attr), Kind::OwnedInto | Kind::RefInto, StructKindHint::Struct) => {
            let field_name = attr.get_ident();
            let right_side = attr.get_action_or(Some(index.to_token_stream()), ctx, || quote!(#obj #index));
            if ctx.has_post_init { quote!(obj.#field_name = #right_side;) } else { quote!(#field_name: #right_side,) }
        },
        (syn::Member::Unnamed(index), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Struct) => {
            let field_path = get_field_path(attr.get_ident());
            let right_side = attr.get_action_or(Some(index.to_token_stream()), ctx, || quote!(#obj #index));
            quote!(other.#field_path = #right_side;)
        },
        (syn::Member::Unnamed(index), Some(attr), Kind::FromOwned | Kind::FromRef, _) => {
            let or = Member::Named(format_ident!("f{}", index.index));
            let right_side = attr.get_stuff(&obj, get_field_path, ctx, || if ctx.destructured_src { &or } else { &f.member});
            quote!(#right_side,)
        }
    }
}

fn render_enum_line(
    v: &Variant,
    ctx: &ImplContext, 
    hint: StructKindHint, 
    _idx: usize
) -> TokenStream {
    let attr = v.attrs.applicable_attr(&ctx.kind, &ctx.struct_attr.ty);

    let src = ctx.src_ty;
    let dst = ctx.dst_ty;

    let ident = &v.ident;

    let variant_struct: Struct<'_> = Struct {
        attrs: DataTypeAttrs {
            attrs: ctx.input.get_attrs().attrs.clone(),
            ghost_attrs: vec![],
            where_attrs: vec![],
            children_attrs: vec![],
        },
        ident,
        generics: &Generics {
            lt_token: None,
            params: Punctuated::new(),
            gt_token: None,
            where_clause: None,
        },
        fields: v.fields.clone(),
        named_fields: v.named_fields,
    };

    let new_ctx = ImplContext {
        input: &DataType::Struct(&variant_struct),
        destructured_src: true,
        ..*ctx
    };

    let destr = if variant_struct.fields.is_empty() { TokenStream::new() } else { variant_destruct_block(&new_ctx) };
    let init = if variant_struct.fields.is_empty() { TokenStream::new() } else { struct_init_block(&new_ctx) };

    match (v.named_fields, attr, &ctx.kind, hint) {
        (_, None, _, StructKindHint::Unspecified) => {
            quote!(#src::#ident #destr => #dst::#ident #init,)
        },
        (_, Some(attr), Kind::FromOwned | Kind::FromRef, StructKindHint::Unspecified) => {
            let member = Member::Named(ident.clone());
            let ident2 = attr.get_field_name_or(&member);
            quote!(#src::#ident2 #destr => #dst::#ident #init,)
        },
        (_, Some(attr), Kind::OwnedInto | Kind::RefInto, StructKindHint::Unspecified) => {
            let member = Member::Named(ident.clone());
            let ident2 = attr.get_field_name_or(&member);
            quote!(#src::#ident #destr => #dst::#ident2 #init,)
        },
        _ => todo!()
    }
}

fn render_ghost_line(ghost_data: &GhostData, ctx: &ImplContext) -> TokenStream {
    let ch = match &ghost_data.child_path {
        Some(ghost_data) => {
            let ch = ghost_data.child_path.to_token_stream();
            quote!(#ch.)
        },
        None => TokenStream::new()
    };
    let right_side = quote_action(&ghost_data.action, None, ctx);
    match (&ghost_data.ghost_ident, &ctx.kind) {
        (Member::Named(ident), Kind::OwnedInto | Kind::RefInto) => quote!(#ident: #right_side,),
        (Member::Unnamed(_), Kind::OwnedInto | Kind::RefInto) => quote!(#right_side,),
        (Member::Named(ident), Kind::OwnedIntoExisting | Kind::RefIntoExisting) => quote!(other.#ch #ident = #right_side;),
        (Member::Unnamed(index), Kind::OwnedIntoExisting | Kind::RefIntoExisting) => quote!(other.#ch #index = #right_side;),
        (_, _) => panic!("weird"),
    }
}

fn replace_tilde_or_at_in_expr(input: &TokenStream, ident: Option<&TokenStream>, path: Option<&TokenStream>) -> TokenStream {
    let mut tokens = Vec::new();

    input.clone().into_iter().for_each(|x| {
        let f = match x {
            proc_macro2::TokenTree::Group(group) => {
                let inner = replace_tilde_or_at_in_expr(&group.stream(), ident, path);
                match group.delimiter() {
                    proc_macro2::Delimiter::Parenthesis => quote!(( #inner )),
                    proc_macro2::Delimiter::Brace => quote!({ #inner }),
                    proc_macro2::Delimiter::Bracket => quote!([ #inner ]),
                    proc_macro2::Delimiter::None => quote!(#inner)
                }
            },
            proc_macro2::TokenTree::Punct(punct) => {
                let ch = punct.as_char();

                if ch == '~' {
                    quote!(#path)
                } else if ch == '@' {
                    quote!(#ident)
                } else {
                    quote!(#punct)
                }
            },
            _ => quote!(#x)
        };

        tokens.push(f)
    });

    TokenStream::from_iter(tokens)
}

fn quote_action(action: &TokenStream, field_path: Option<TokenStream>, ctx: &ImplContext) -> TokenStream {
    let ident = match ctx.kind {
        Kind::FromOwned | Kind::FromRef => quote!(value),
        _ => quote!(self),
    };
    let path = if ctx.destructured_src {
        quote!(#field_path)
    } else {
        match ctx.kind {
            Kind::FromOwned | Kind::FromRef => quote!(value.#field_path),
            _ => quote!(self.#field_path),
        }
    };
    replace_tilde_or_at_in_expr(action, Some(&ident), Some(&path))
}

fn quote_from_trait(ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream) -> TokenStream {
    let dst = ctx.dst_ty;
    let src = ctx.src_ty;
    let gens = ctx.input.get_generics().to_token_stream();
    let where_clause  = ctx.input.get_attrs().where_attr(&ctx.struct_attr.ty).map(|x| {
        let where_clause = x.where_clause.to_token_stream();
        quote!(where #where_clause)
    });
    let r = ctx.kind.is_ref().then_some(quote!(&));
    quote! {
        impl #gens std::convert::From<#r #src> for #dst #gens #where_clause {
            fn from(value: #r #src) -> #dst #gens {
                #pre_init
                #init
            }
        }
    }
}

fn quote_into_trait(ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream, post_init: Option<TokenStream>) -> TokenStream {
    let dst = ctx.dst_ty;
    let src = ctx.src_ty;
    let gens = ctx.input.get_generics().to_token_stream();
    let where_clause  = ctx.input.get_attrs().where_attr(&ctx.struct_attr.ty).map(|x|{
        let where_clause = x.where_clause.to_token_stream();
        quote!(where #where_clause)
    });
    let r = ctx.kind.is_ref().then_some(quote!(&));
    match post_init {
        Some(post_init) => quote!{
            impl #gens std::convert::Into<#dst> for #r #src #gens #where_clause {
                fn into(self) -> #dst {
                    let mut obj: #dst = Default::default();
                    #init
                    #post_init
                    obj
                }
            }
        },
        None => quote! {
            impl #gens std::convert::Into<#dst> for #r #src #gens #where_clause {
                fn into(self) -> #dst {
                    #pre_init
                    #init
                }
            }
        }
    }
}

fn quote_into_existing_trait(ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream, post_init: Option<TokenStream>) -> TokenStream {
    let dst = ctx.dst_ty;
    let src = ctx.src_ty;
    let gens = ctx.input.get_generics().to_token_stream();
    let where_clause  = ctx.input.get_attrs().where_attr(&ctx.struct_attr.ty).map(|x|{
        let where_clause = x.where_clause.to_token_stream();
            quote!(where #where_clause)
    });
    let r = ctx.kind.is_ref().then_some(quote!(&));
    quote! {
        impl #gens o2o::traits::IntoExisting<#dst> for #r #src #gens #where_clause {
            fn into_existing(self, other: &mut #dst) {
                #pre_init
                #init
                #post_init
            }
        }
    }
}

impl<'a> ApplicableAttr<'a> {
    fn get_ident(&self) -> &'a Member {
        match self {
            ApplicableAttr::Field(field_attr) => {
                match &field_attr.member {
                    Some(val) => val,
                    None => panic!("weird"),
                }
            },
            ApplicableAttr::Ghost(_) => panic!("weird")
        }
    }

    fn get_field_name_or(&'a self, field: &'a Member) -> &'a Member {
        match self {
            ApplicableAttr::Field(field_attr) => {
                match &field_attr.member {
                    Some(val) => val,
                    None => field,
                }
            },
            ApplicableAttr::Ghost(_) => panic!("weird")
        }
    }

    fn get_action_or<F: Fn() -> TokenStream>(&self, field_path: Option<TokenStream>, ctx: &ImplContext, or: F) -> TokenStream {
        match self {
            ApplicableAttr::Field(field_attr) => {
                match &field_attr.action {
                    Some(val) => quote_action(val, field_path, ctx),
                    None => or()
                }
            },
            ApplicableAttr::Ghost(_) => panic!("weird")
        }
    }

    fn get_stuff<F1: Fn(&Member) -> TokenStream, F2: Fn() -> &'a Member>(&self, obj: &TokenStream, field_path: F1, ctx: &ImplContext, or: F2) -> TokenStream {
        match self {
            ApplicableAttr::Field(field_attr) => {
                match (&field_attr.member, &field_attr.action) {
                    (Some(ident), Some(action)) => {
                        if let Member::Unnamed(index) = ident {
                            if ctx.destructured_src { 
                                let ident = Member::Named(format_ident!("f{}", index.index));
                                quote_action(action, Some(field_path(&ident)), ctx) 
                            } else { quote_action(action, Some(field_path(ident)), ctx)}
                        } else {
                            quote_action(action, Some(field_path(ident)), ctx)
                        }
                        
                    },
                    (Some(ident), None) => {
                        let field_path = field_path(ident);
                        quote!(#obj #field_path)
                    },
                    (None, Some(action)) => quote_action(action, Some(field_path(or())), ctx),
                    _ => panic!("weird")
                }
            },
            ApplicableAttr::Ghost(ghost_attr) => {
                quote_action(ghost_attr.action.as_ref().unwrap(), None, ctx)
            }
        }
    }
}