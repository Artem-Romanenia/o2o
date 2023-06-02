use std::{slice::Iter, iter::Peekable, collections::HashSet};

use crate::{ast::{Struct, Field}, attr::{
    Action, ChildData, Kind, StructKindHint, ApplicableAttr, TypePath, 
    GhostData, ChildPath, WrappedAttr, FieldAttrs, WrapperAttr, InitData
}, validate::validate};
use proc_macro2::{TokenStream, Ident, Span};
use syn::{DeriveInput, Result, Error, Data, parse::ParseStream, Token, Member, Index, punctuated::Punctuated};
use quote::{quote, ToTokens};

pub fn derive(node: &DeriveInput) -> Result<TokenStream> {
    match &node.data {
        Data::Struct(data) => {
            let input = Struct::from_syn(node, data)?;
            validate(&input)?;
            Ok(struct_impl(input))
        },
        _ => Err(Error::new_spanned(
            node,
            "#[derive(o2o)] only supports structs.",
        ))
    }
}

struct ImplContext<'a> {
    input: &'a Struct<'a>,
    kind: Kind,
    struct_kind_hint: StructKindHint,
    dst_ty: &'a TokenStream,
    src_ty: &'a TokenStream,
    nameless: bool,
    container_ty: &'a TypePath,
    has_post_init: bool
}

fn struct_impl(input: Struct) -> TokenStream {
    let ty = input.ident.to_token_stream();

    let from_owned_impls = input.attrs.iter_for_kind(&Kind::FromOwned).map(|struct_attr| {
        let ctx = ImplContext{
            input: &input,
            kind: Kind::FromOwned,
            struct_kind_hint: struct_attr.struct_kind_hint,
            dst_ty: &ty,
            src_ty: &struct_attr.ty.path,
            nameless: struct_attr.ty.nameless,
            container_ty: &struct_attr.ty,
            has_post_init: false,
        };

        if let Some(wrapped_attr) = input.attrs.wrapped_attr(ctx.container_ty) {
            let f = &wrapped_attr.ident;
            quote_from_trait(&ctx, None, quote!(value.#f), true)
        } else {
            let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
            let init = struct_init_block(&ctx);
            quote_from_trait(&ctx, pre_init, init, false)
        }
    });

    let from_ref_impls = input.attrs.iter_for_kind(&Kind::FromRef).map(|struct_attr| {
        let ctx = ImplContext{
            input: &input,
            kind: Kind::FromRef,
            struct_kind_hint: struct_attr.struct_kind_hint,
            dst_ty: &ty,
            src_ty: &struct_attr.ty.path,
            nameless: struct_attr.ty.nameless,
            container_ty: &struct_attr.ty,
            has_post_init: false,
        };

        if let Some(wrapped_attr) = input.attrs.wrapped_attr(ctx.container_ty) {
            let f = &wrapped_attr.ident;
            let field_path = match &wrapped_attr.action {
                Some(action) =>  quote_action(&Action::InlineTildeExpr(action.clone()), Some(f.to_token_stream()), &ctx),
                None => quote!(value.#f)
            };
            quote_from_trait(&ctx, None, field_path, true)
        } else {
            let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
            let init = struct_init_block(&ctx);
            quote_from_trait(&ctx, pre_init, init, false)
        }
    });

    let owned_into_impls = input.attrs.iter_for_kind(&Kind::OwnedInto).map(|struct_attr| {
        let mut ctx = ImplContext{
            input: &input,
            kind: Kind::OwnedInto,
            struct_kind_hint: struct_attr.struct_kind_hint,
            dst_ty: &struct_attr.ty.path,
            src_ty: &ty,
            nameless: struct_attr.ty.nameless,
            container_ty: &struct_attr.ty,
            has_post_init: false,
        };

        let wrapped_field = input.fields.iter().find(|f| f.attrs.field_attr(&Kind::OwnedInto, &struct_attr.ty).filter(|&g| g.wrapper).is_some());

        if let Some(wrapped_field) = wrapped_field {
            let f = &wrapped_field.member;
            quote_into_trait(&ctx, None, quote!(self.#f), None, true)
        } else {
            let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
            let post_init = struct_post_init(&ctx);
            ctx.has_post_init = post_init.is_some();
            let init = struct_init_block(&ctx);
            quote_into_trait(&ctx, pre_init, init, post_init, false)
        }
    });

    let ref_into_impls = input.attrs.iter_for_kind(&Kind::RefInto).map(|struct_attr| {
        let mut ctx = ImplContext{
            input: &input,
            kind: Kind::RefInto,
            struct_kind_hint: struct_attr.struct_kind_hint,
            dst_ty: &struct_attr.ty.path,
            src_ty: &ty,
            nameless: struct_attr.ty.nameless,
            container_ty: &struct_attr.ty,
            has_post_init: false,
        };

        let wrapped_field = input.fields.iter().find_map(|f|
            f.attrs.field_attr(&Kind::RefInto, &struct_attr.ty)
                .filter(|&g| g.wrapper)
                .map(|x| (f, &x.action))
        );

        if let Some((wrapped_field, action)) = wrapped_field {
            let f = &wrapped_field.member;
            let field_path = match action {
                Some(action) =>  quote_action(action, Some(f.to_token_stream()), &ctx),
                None => quote!(self.#f)
            };
            quote_into_trait(&ctx, None, field_path, None, true)
        } else {
            let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
            let post_init = struct_post_init(&ctx);
            ctx.has_post_init = post_init.is_some();
            let init = struct_init_block(&ctx);
            quote_into_trait(&ctx, pre_init, init, post_init, false)
        }
    });

    let owned_into_existing_impls = input.attrs.iter_for_kind(&Kind::OwnedIntoExisting).map(|struct_attr| {
        let mut ctx = ImplContext{
            input: &input,
            kind: Kind::OwnedIntoExisting,
            struct_kind_hint: struct_attr.struct_kind_hint,
            dst_ty: &struct_attr.ty.path,
            src_ty: &ty,
            nameless: struct_attr.ty.nameless,
            container_ty: &struct_attr.ty,
            has_post_init: false,
        };
        let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
        let post_init = struct_post_init(&ctx);
        ctx.has_post_init = post_init.is_some();
        let init = struct_init_block(&ctx);
        quote_into_existing_trait(&ctx, pre_init, init, post_init)
    });

    let ref_into_existing_impls = input.attrs.iter_for_kind(&Kind::RefIntoExisting).map(|struct_attr| {
        let mut ctx = ImplContext{
            input: &input,
            kind: Kind::RefIntoExisting,
            struct_kind_hint: struct_attr.struct_kind_hint,
            dst_ty: &struct_attr.ty.path,
            src_ty: &ty,
            nameless: struct_attr.ty.nameless,
            container_ty: &struct_attr.ty,
            has_post_init: false,
        };
        let pre_init = struct_pre_init(&ctx, &struct_attr.init_data);
        let post_init = struct_post_init(&ctx);
        ctx.has_post_init = post_init.is_some();
        let init = struct_init_block(&ctx);
        quote_into_existing_trait(&ctx, pre_init, init, post_init)
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

fn struct_init_block<'a>(ctx: &'a ImplContext) -> TokenStream {
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

    let wrapped_field = ctx.input.attrs.wrapped_attr(ctx.container_ty);

    if let Some(wrapped_field) = wrapped_field {
        let child_path = wrapped_field.child_path.as_ref().map(|x| x.get_child_path_str(None)).unwrap_or("");
        fields.push(make_tuple(child_path, FieldData::WrappedData(wrapped_field)))
    } else {
        fields.extend(ctx.input.fields.iter()
        .map(|x| {
            let path = x.attrs.child(ctx.container_ty).map(|x| x.get_child_path_str(None)).unwrap_or("");
            unique_paths.insert(path);
            make_tuple(path, FieldData::Field(x))
        })
        .collect::<Vec<(usize, &str, FieldData)>>());
    }
    
    fields.extend(ctx.input.attrs.ghost_attrs.iter()
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
    Field(&'a Field),
    GhostData(&'a GhostData),
    WrappedData(&'a WrappedAttr)
}

fn struct_init_block_inner(
    fields: &mut Peekable<Iter<(usize, &str, FieldData)>>,
    ctx: &ImplContext,
    field_ctx: Option<(&ChildPath, Option<&ChildData>, usize)>
) -> TokenStream
{
    let next = fields.peek();
    if next.is_none() {
        return quote!({})
    }

    let struct_kind_hint = match field_ctx {
        Some(field_ctx) => match field_ctx.1 {
            Some(child_data) => child_data.struct_kind_hint,
            None => ctx.struct_kind_hint
        },
        None => ctx.struct_kind_hint
    };

    let mut fragments: Vec<TokenStream> = vec![];
    let mut idx: usize = 0;

    while let Some((_, key, field_data)) = fields.peek() {
        if let Some(field_ctx) = field_ctx {
            if !key.starts_with(field_ctx.0.get_child_path_str(Some(field_ctx.2))) {
                break;
            }
        }

        match field_data {
            FieldData::Field(f) => {
                if ctx.kind != Kind::FromOwned && ctx.kind != Kind::FromRef 
                    && (f.attrs.ghost(ctx.container_ty, &ctx.kind).is_some() || f.attrs.has_parent_attr(ctx.container_ty)) 
                {
                    fields.next();
                    continue;
                }

                let fragment = match f.attrs.child(ctx.container_ty) {
                    Some(child_attr) => 
                        render_child_fragment(&child_attr.child_path, fields, ctx, field_ctx, struct_kind_hint, || render_line(f, ctx, struct_kind_hint, idx)),
                    None => {
                        fields.next();
                        render_line(f, ctx, struct_kind_hint, idx)
                    }
                };
                fragments.push(fragment);
                idx += 1;
            },
            FieldData::GhostData(ghost_data) => {
                let child_path = &ghost_data.child_path.as_ref().unwrap();
                let fragment = render_child_fragment(child_path, fields, ctx, field_ctx, struct_kind_hint, TokenStream::new);

                fragments.push(fragment);
                idx += 1;
            }
            FieldData::WrappedData(wrapped_attr) => {
                let mut attrs = vec![];
                crate::attr::add_wrapper_attrs(WrapperAttr {
                    container_ty: wrapped_attr.container_ty.clone(),
                    action: wrapped_attr.action.clone()
                }, &mut attrs, ctx.kind == Kind::OwnedInto || ctx.kind == Kind::RefInto);
                let f = Field {
                    member: wrapped_attr.ident.clone(),
                    attrs: FieldAttrs{ attrs, ..Default::default() },
                    idx: 0
                };
                let fragment = match &wrapped_attr.child_path {
                    Some(child_path) => 
                        render_child_fragment(child_path, fields, ctx, field_ctx, struct_kind_hint, || render_line(&f, ctx, struct_kind_hint, idx)),
                    None => {
                        fields.next();
                        render_line(&f, ctx, struct_kind_hint, idx)
                    }
                };
                fragments.push(fragment);
                idx += 1;
            }
        }
    }

    if ctx.kind != Kind::FromOwned && ctx.kind != Kind::FromRef {
        if let Some(ghost_attr) = ctx.input.attrs.ghost_attr(ctx.container_ty, &ctx.kind) {
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

    match (ctx.has_post_init, field_ctx, &ctx.kind, struct_kind_hint, ctx.input.named) {
        (_, Some(_), Kind::RefIntoExisting | Kind::OwnedIntoExisting, _, _) => quote!(#(#fragments)*),
        (_, None, Kind::RefIntoExisting | Kind::OwnedIntoExisting, _, _) => quote!(#(#fragments)*),
        (false, _, Kind::FromOwned | Kind::FromRef, _, true) => quote!({#(#fragments)*}),
        (false, _, Kind::FromOwned | Kind::FromRef, _, false) => quote!((#(#fragments)*)),
        (false, _, _, StructKindHint::Struct, _) => quote!({#(#fragments)*}),
        (false, _, _, StructKindHint::Tuple, _) => quote!((#(#fragments)*)),
        (false, _, _, StructKindHint::Unspecified, true) => quote!({#(#fragments)*}),
        (true, _, _, StructKindHint::Unspecified, false) => quote!(#(#fragments)*),
        (false, _, _, StructKindHint::Unspecified, false) => quote!((#(#fragments)*)),
        (true, _, _, StructKindHint::Struct, _) => quote!({#(#fragments)* ..Default::default()}),
        (true, _, _, StructKindHint::Tuple, _) => quote!((#(#fragments)* ..Default::default())),
        (true, _, _, StructKindHint::Unspecified, true) => quote!({#(#fragments)* ..Default::default()}),
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

    ctx.input.fields.iter().for_each(|f| {
        if ctx.kind != Kind::FromOwned && ctx.kind != Kind::FromRef && f.attrs.has_parent_attr(ctx.container_ty) {
            fragments.push(render_parent(f, ctx))
        }
    });

    if fragments.is_empty() {
        return None
    }
    Some(quote!(#(#fragments)*))
}

fn render_parent(f: &Field, ctx: &ImplContext) -> TokenStream {
    match (&f.member, &ctx.kind) {
        (syn::Member::Named(ident), Kind::OwnedIntoExisting) => quote!(self.#ident.into_existing(other);),
        (syn::Member::Unnamed(index), Kind::OwnedIntoExisting) => quote!(self.#index.into_existing(other);),
        (syn::Member::Named(ident), Kind::RefIntoExisting) => quote!((&(self.#ident)).into_existing(other);),
        (syn::Member::Unnamed(index), Kind::RefIntoExisting) => quote!((&(self.#index)).into_existing(other);),
        (syn::Member::Named(ident), Kind::OwnedInto) => quote!(self.#ident.into_existing(&mut obj);),
        (syn::Member::Unnamed(index), Kind::OwnedInto) => quote!(self.#index.into_existing(&mut obj);),
        (syn::Member::Named(ident), Kind::RefInto) => quote!((&(self.#ident)).into_existing(&mut obj);),
        (syn::Member::Unnamed(index), Kind::RefInto) => quote!((&(self.#index)).into_existing(&mut obj);),
        _ => panic!("weird")
    }
}

fn render_child(
    fields: &mut Peekable<Iter<(usize, &str, FieldData)>>,
    ctx: &ImplContext,
    field_ctx: (&ChildPath, usize),
    hint: StructKindHint) -> TokenStream
{
    let child_path = field_ctx.0;
    let path = child_path.get_child_path_str(Some(field_ctx.1));
    let child_name = child_path.child_path[field_ctx.1].to_token_stream();
    let mut children = ctx.input.attrs.children_attr(ctx.container_ty).unwrap().children.iter();
    let child_data = children.find(|child_data| child_data.check_match(path)).unwrap();
    let ty = &child_data.ty;
    let init = struct_init_block_inner(fields, ctx, Some((field_ctx.0, Some(child_data), field_ctx.1 )));
    match (ctx.input.named, hint) {
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
    let child_attr = field_ctx.0;
    let path = child_attr.get_child_path_str(Some(field_ctx.1));
    let children_attr = ctx.input.attrs.children_attr(ctx.container_ty);
    let child_data = children_attr.and_then(|x| 
        x.children.iter().find(|child_data| child_data.check_match(path)));
    struct_init_block_inner(fields, ctx, Some((field_ctx.0, child_data, field_ctx.1 )))
}

fn render_line(
    f: &Field,
    ctx: &ImplContext, 
    hint: StructKindHint, 
    idx: usize
) -> TokenStream {
    let attr = f.attrs.applicable_attr(&ctx.kind, ctx.container_ty);
    let get_field_path = |x: &Member| {
        match f.attrs.child(ctx.container_ty) {
            Some(child_attr) => {
                let ch = child_attr.child_path.child_path.to_token_stream();
                quote!(#ch.#x)
            },
            None => x.to_token_stream()
        }
    };

    match (&f.member, attr, &ctx.kind, hint, ctx.has_post_init) {
        (syn::Member::Named(ident), None, Kind::OwnedInto | Kind::RefInto, StructKindHint::Struct | StructKindHint::Unspecified, _) => 
            quote!(#ident: self.#ident,),
        (syn::Member::Named(ident), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Struct | StructKindHint::Unspecified, _) => {
            let field_path = get_field_path(&f.member);
            quote!(other.#field_path = self.#ident;)
        },
        (syn::Member::Named(ident), None, Kind::OwnedInto | Kind::RefInto, StructKindHint::Tuple, _) => 
            quote!(self.#ident,),
        (syn::Member::Named(ident), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Tuple, _) => {
            let index = Member::Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
            quote!(other.#index = self.#ident;)
        }
        (syn::Member::Named(ident), None, Kind::FromOwned | Kind::FromRef, StructKindHint::Struct | StructKindHint::Unspecified, _) => 
            if f.attrs.has_parent_attr(ctx.container_ty) {
                if ctx.kind == Kind::FromOwned {
                    quote!(#ident: (&value).into(),)
                }else {
                    quote!(#ident: value.into(),)
                }
            } else {
                let field_path = get_field_path(&f.member);
                quote!(#ident: value.#field_path,)
            },
        (syn::Member::Named(ident), None, Kind::FromOwned | Kind::FromRef, StructKindHint::Tuple, _) => {
            let index = Member::Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
            let field_path = get_field_path(&index);
            quote!(#ident: value.#field_path,)
        },
        (syn::Member::Unnamed(index), None, Kind::OwnedInto | Kind::RefInto, StructKindHint::Tuple | StructKindHint::Unspecified, false) => 
            quote!(self.#index,),
        (syn::Member::Unnamed(index), None, Kind::OwnedInto | Kind::RefInto, StructKindHint::Tuple | StructKindHint::Unspecified, true) => {
            let index2 = Member::Unnamed(Index { index: idx as u32, span: Span::call_site() });
            quote!(obj.#index2 = self.#index;)
        },
        (syn::Member::Unnamed(index), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Tuple | StructKindHint::Unspecified, _) => {
            let index2 = Member::Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
            quote!(other.#index2 = self.#index;)
        }
        (syn::Member::Unnamed(_), None, Kind::FromOwned | Kind::FromRef, StructKindHint::Tuple | StructKindHint::Unspecified, _) => 
            if f.attrs.has_parent_attr(ctx.container_ty) {
                if ctx.kind == Kind::FromOwned {
                    quote!((&value).into(),)
                }else {
                    quote!(value.into(),)
                }
            } else {
                let field_path = get_field_path(&f.member);
                quote!(value.#field_path,)
            },
        (syn::Member::Unnamed(_), None, _, StructKindHint::Struct, _) => {
            if f.attrs.has_parent_attr(ctx.container_ty) {
                if ctx.kind == Kind::FromOwned {
                    quote!((&value).into(),)
                }else {
                    quote!(value.into(),)
                }
            } else {
                panic!("weird")
            }
        },
        (syn::Member::Named(ident), Some(attr), Kind::OwnedInto | Kind::RefInto, StructKindHint::Struct | StructKindHint::Unspecified, _) => {
            let field_name = attr.get_field_name_or(&f.member);
            let right_side = attr.get_action_or(Some(f.member.to_token_stream()), ctx, || quote!(self.#ident));
            quote!(#field_name: #right_side,)
        },
        (syn::Member::Named(ident), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Struct | StructKindHint::Unspecified, _) => {
            let field_path = get_field_path(attr.get_field_name_or(&f.member));
            let right_side = attr.get_action_or(Some(ident.to_token_stream()), ctx, || quote!(self.#ident));
            quote!(other.#field_path = #right_side;)
        },
        (syn::Member::Named(ident), Some(attr), Kind::OwnedInto | Kind::RefInto, StructKindHint::Tuple, _) => {
            let right_side = attr.get_action_or(Some(get_field_path(&f.member)), ctx, || quote!(self.#ident));
            quote!(#right_side,)
        },
        (syn::Member::Named(ident), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Tuple, _) => {
            let field_path = get_field_path(&Member::Unnamed(Index { index: idx as u32, span: Span::call_site() }));
            let right_side = attr.get_action_or(Some(ident.to_token_stream()), ctx, || quote!(self.#ident));
            quote!(other.#field_path = #right_side;)
        },
        (syn::Member::Named(ident), Some(attr), Kind::FromOwned | Kind::FromRef, _, _) => {
            let right_side = attr.get_stuff(get_field_path, ctx, || &f.member);
            quote!(#ident: #right_side,)
        },
        (syn::Member::Unnamed(index), Some(attr), Kind::OwnedInto | Kind::RefInto, StructKindHint::Tuple | StructKindHint::Unspecified, _) => {
            let right_side = attr.get_action_or(Some(get_field_path(&f.member)), ctx, || quote!(self.#index));
            quote!(#right_side,)
        },
        (syn::Member::Unnamed(index), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Tuple | StructKindHint::Unspecified, _) => {
            let field_path = get_field_path(attr.get_field_name_or(&f.member));
            let right_side = attr.get_action_or(Some(get_field_path(&f.member)), ctx, || quote!(self.#index));
            quote!(other.#field_path = #right_side;)
        },
        (syn::Member::Unnamed(index), Some(attr), Kind::OwnedInto | Kind::RefInto, StructKindHint::Struct, _) => {
            let field_name = attr.get_ident();
            let right_side = attr.get_action_or(Some(get_field_path(&f.member)), ctx, || quote!(self.#index));
            quote!(#field_name: #right_side,)
        },
        (syn::Member::Unnamed(index), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, StructKindHint::Struct, _) => {
            let field_path = get_field_path(attr.get_ident());
            let right_side = attr.get_action_or(Some(get_field_path(&f.member)), ctx, || quote!(self.#index));
            quote!(other.#field_path = #right_side;)
        },
        (syn::Member::Unnamed(_), Some(attr), Kind::FromOwned | Kind::FromRef, _, _) => {
            let right_side = attr.get_stuff(get_field_path, ctx, || &f.member);
            quote!(#right_side,)
        }
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

fn type_closure_param(input: &TokenStream, ctx: &ImplContext) -> TokenStream {
    let mut cl = TokenStream::new();
    syn::parse::Parser::parse2(|x: ParseStream| {
        let mut tokens = Vec::new();
        tokens.push(x.parse::<Token![|]>().unwrap().to_token_stream());
        if x.peek(Token![_]){
            tokens.push(x.parse::<Token![_]>().unwrap().to_token_stream());
        } else {
            tokens.push(x.parse::<Ident>().unwrap().to_token_stream());
        }
        tokens.push(syn::parse_str(":&").unwrap());
        tokens.push(ctx.src_ty.to_token_stream());
        if ctx.kind != Kind::FromOwned && ctx.kind != Kind::FromRef {
            tokens.push(ctx.input.generics.to_token_stream());
        }
        tokens.push(x.parse::<Token![|]>().unwrap().to_token_stream());
        tokens.push(x.parse().unwrap());

        cl = TokenStream::from_iter(tokens.into_iter());
        Ok(())
    }, input.clone()).unwrap();
    cl
}

fn quote_action(action: &Action, field_path: Option<TokenStream>, ctx: &ImplContext) -> TokenStream {
    match action {
        Action::InlineAtExpr(args) => {
            let ident = match ctx.kind {
                Kind::FromOwned | Kind::FromRef => quote!(value),
                _ => quote!(self),
            };
            quote!(#ident #args)
        },
        Action::InlineTildeExpr(args)  => {
            let path = match ctx.kind {
                Kind::FromOwned | Kind::FromRef => quote!(value.#field_path),
                _ => quote!(self.#field_path),
            };
            quote!(#path #args)
        },
        Action::InlineExpr(args) => args.clone(),
        Action::Closure(args) => {
            let ident = match ctx.kind {
                Kind::FromOwned | Kind::FromRef => quote!(value),
                _ => quote!(self),
            };
            let cl = type_closure_param(args, ctx);
            if ctx.kind == Kind::RefInto || ctx.kind == Kind::FromRef {
                quote!((#cl)(#ident))
            } else {
                quote!((#cl)(&#ident))
            }
        },
        Action::ParamlessClosure(args) => {
            quote!((#args)())
        }
    }
}

fn quote_from_trait(ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream, wrapped: bool) -> TokenStream {
    let dst = ctx.dst_ty;
    let src = ctx.src_ty;
    let gens = ctx.input.generics.to_token_stream();
    let where_clause  = match ctx.input.attrs.where_attr(ctx.container_ty){
        Some(where_attr) => {
            let where_clause = where_attr.where_clause.to_token_stream();
            quote!(where #where_clause)
        },
        None => TokenStream::new()
    };
    let r = if ctx.kind == Kind::FromRef {
        quote!(&)
    } else {
        TokenStream::new()
    };
    if wrapped {
        quote! {
            impl #gens std::convert::From<#r #src> for #dst #gens #where_clause {
                fn from(value: #r #src) -> #dst #gens {
                    #init
                }
            }
        }
    } else {
        quote! {
            impl #gens std::convert::From<#r #src> for #dst #gens #where_clause {
                fn from(value: #r #src) -> #dst #gens {
                    #pre_init
                    #dst #init
                }
            }
        }
    }
}

fn quote_into_trait(ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream, post_init: Option<TokenStream>, wrapped: bool) -> TokenStream {
    let dst = ctx.dst_ty;
    let dst2 = if ctx.nameless { TokenStream::new() } else { dst.clone() };
    let src = ctx.src_ty;
    let gens = ctx.input.generics.to_token_stream();
    let where_clause  = match ctx.input.attrs.where_attr(ctx.container_ty){
        Some(where_attr) => {
            let where_clause = where_attr.where_clause.to_token_stream();
            quote!(where #where_clause)
        },
        None => TokenStream::new()
    };
    let r = if ctx.kind == Kind::RefInto {
        quote!(&)
    } else {
        TokenStream::new()
    };
    match (&ctx.input.named, ctx.struct_kind_hint, post_init) {
        (false, StructKindHint::Tuple | StructKindHint::Unspecified, Some(post_init))
        | (true, StructKindHint::Tuple, Some(post_init)) => quote!{
            impl #gens std::convert::Into<#dst> for #r #src #gens #where_clause {
                fn into(self) -> #dst {
                    let mut obj: #dst = Default::default();
                    #init
                    #post_init
                    obj
                }
            }
        },
        (true, StructKindHint::Struct | StructKindHint::Unspecified, Some(post_init))
        |  (false, StructKindHint::Struct, Some(post_init)) => quote!{
            impl #gens std::convert::Into<#dst> for #r #src #gens #where_clause {
                fn into(self) -> #dst {
                    let mut obj = #dst #init;
                    #post_init
                    obj
                }
            }
        },
        (_, _, None) if wrapped => quote! {
            impl #gens std::convert::Into<#dst> for #r #src #gens #where_clause {
                fn into(self) -> #dst {
                    #init
                }
            }
        },
        (_, _, None) => quote! {
            impl #gens std::convert::Into<#dst> for #r #src #gens #where_clause {
                fn into(self) -> #dst {
                    #pre_init
                    #dst2 #init
                }
            }
        }
    }
}

fn quote_into_existing_trait(ctx: &ImplContext, pre_init: Option<TokenStream>, init: TokenStream, post_init: Option<TokenStream>) -> TokenStream {
    let dst = ctx.dst_ty;
    let src = ctx.src_ty;
    let gens = ctx.input.generics.to_token_stream();
    let where_clause  = match ctx.input.attrs.where_attr(ctx.container_ty){
        Some(where_attr) => {
            let where_clause = where_attr.where_clause.to_token_stream();
            quote!(where #where_clause)
        },
        None => TokenStream::new()
    };
    let r = if ctx.kind == Kind::RefIntoExisting {
        quote!(&)
    } else {
        TokenStream::new()
    };
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
                match &field_attr.ident {
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
                match &field_attr.ident {
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

    fn get_stuff<F1: Fn(&Member) -> TokenStream, F2: Fn() -> &'a Member>(&self, field_path: F1, ctx: &ImplContext, or: F2) -> TokenStream {
        match self {
            ApplicableAttr::Field(field_attr) => {
                match (&field_attr.ident, &field_attr.action) {
                    (Some(ident), Some(action)) => quote_action(action, Some(field_path(ident)), ctx),
                    (Some(ident), None) => {
                        let field_path = field_path(ident);
                        quote!(value.#field_path)
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