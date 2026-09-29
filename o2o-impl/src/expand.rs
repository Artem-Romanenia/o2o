use std::{collections::HashMap, iter::Peekable, slice::Iter};

use crate::{model::*, render::*, validate::validate};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

#[cfg(feature = "syn2")]
use syn2 as syn;

use syn::{
    parse_quote, Data, DeriveInput, Error, GenericArgument, GenericParam, Lifetime,
    Member::{Named, Unnamed},
    Result,
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

pub(crate) struct ImplContext<'a> {
    pub input: &'a DataType<'a>,
    pub impl_type: ImplType,
    pub struct_attr: &'a TraitAttrCore,
    pub kind: Kind,
    pub dst_ty: &'a TokenStream,
    pub src_ty: &'a TokenStream,
    pub has_post_init: bool,
    pub fallible: bool,
}

impl<'a> ImplContext<'a> {
    fn to_render_ctx(&'a self) -> RenderContext<'a> {
        RenderContext { 
            impl_type: self.impl_type ,
            kind: self.kind,
            dst_ty: self.dst_ty,
            src_ty: self.src_ty,
            has_post_init: self.has_post_init,
            fallible: self.fallible
        }
    }
}

fn data_type_impl(input: DataType) -> TokenStream {
    let ty = input.get_ident().to_token_stream();
    let attrs = input.get_attrs();

    let impl_type = match input {
        DataType::Struct(_) => ImplType::Struct,
        DataType::Enum(_) => ImplType::Enum,
    };

    let has_post_init = |a: &TraitAttrCore| input.get_members().iter().any(|x|x.get_attrs().has_parameterless_parent_attr(&a.ty));

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
        has_post_init: has_post_init(struct_attr),
        fallible: false,
    })).chain(attrs.iter_for_kind_core(&Kind::OwnedInto, true).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::OwnedInto,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: has_post_init(struct_attr),
        fallible: true,
    })).chain(attrs.iter_for_kind_core(&Kind::RefInto, false).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::RefInto,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: has_post_init(struct_attr),
        fallible: false,
    })).chain(attrs.iter_for_kind_core(&Kind::RefInto, true).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::RefInto,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: has_post_init(struct_attr),
        fallible: true,
    })).chain(attrs.iter_for_kind_core(&Kind::OwnedIntoExisting, false).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::OwnedIntoExisting,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: has_post_init(struct_attr),
        fallible: false,
    })).chain(attrs.iter_for_kind_core(&Kind::OwnedIntoExisting, true).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::OwnedIntoExisting,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: has_post_init(struct_attr),
        fallible: true,
    })).chain(attrs.iter_for_kind_core(&Kind::RefIntoExisting, false).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::RefIntoExisting,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: has_post_init(struct_attr),
        fallible: false,
    })).chain(attrs.iter_for_kind_core(&Kind::RefIntoExisting, true).map(|struct_attr| ImplContext {
        input: &input, impl_type, struct_attr,
        kind: Kind::RefIntoExisting,
        dst_ty: &struct_attr.ty.path,
        src_ty: &ty,
        has_post_init: has_post_init(struct_attr),
        fallible: true,
    })).map(|mut ctx| quote_trait(&input, &mut ctx));

    quote! { #(#impls)* }
}

fn enum_main_code_block(input: &Enum, ctx: &ImplContext) -> TokenStream {
    let enum_init_block = enum_init_block(input, ctx);

    match ctx.kind {
        Kind::FromOwned | Kind::FromRef => {
            let match_expr = if let Some(ts) = &ctx.struct_attr.match_expr { replace_tilde_or_at_in_expr(&ts.expr.expr, Some(&quote!(value)), None) } else { quote!(value) };
            quote!(match #match_expr #enum_init_block)
        },
        Kind::OwnedInto | Kind::RefInto => {
            let match_expr = if let Some(ts) = &ctx.struct_attr.match_expr { replace_tilde_or_at_in_expr(&ts.expr.expr, Some(&quote!(self)), None) } else { quote!(self) };
            quote!(match #match_expr #enum_init_block)
        },
        Kind::OwnedIntoExisting | Kind::RefIntoExisting => enum_init_block,
    }
}

fn enum_init_block(input: &Enum, ctx: &ImplContext) -> TokenStream {
    let mut fields: Vec<VariantData> = vec![];

    fields.extend(input.variants.iter()
        .map(VariantData::Variant).collect::<Vec<VariantData>>());

    fields.extend(input.attrs.ghosts_attrs.iter()
        .flat_map(|x| &x.attr.ghost_data)
        .map(VariantData::GhostData));

    enum_init_block_inner(&mut fields.iter().peekable(), ctx)
}

fn enum_init_block_inner(members: &mut Peekable<Iter<VariantData>>, ctx: &ImplContext) -> TokenStream {
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
            },
            VariantData::GhostData(ghost_data) => {
                members.next();
                let fragment = render_enum_ghost_line(ghost_data, ctx);
                fragments.push(fragment);
            },
        }
    }

    if let Some(default_case) = &ctx.struct_attr.default_case {
        let g = render_action(&default_case.expr.expr, None, &ctx.to_render_ctx());
        fragments.push(quote!(_ #g))
    }

    quote!({#(#fragments)*})
}

fn variant_destruct_block(input: &Struct, ctx: &ImplContext) -> TokenStream {
    let (mut idents, type_hint) = match (input.struct_kind.is_struct(), ctx.kind, ctx.struct_attr.type_hint) {
        (true, Kind::OwnedInto | Kind::RefInto | Kind::OwnedIntoExisting | Kind::RefIntoExisting, _) | 
        (true, _, TypeHint::Struct | TypeHint::Unspecified) | 
        (false, Kind::FromOwned | Kind::FromRef, TypeHint::Struct) => (
            input.fields.iter().filter(|x| !ctx.kind.is_from() || x.attrs.ghost(&ctx.struct_attr.ty, &ctx.kind).is_none())
                .map(|x| {
                    let attr = ApplicableAttr::get(&x.attrs, &ctx.kind, ctx.fallible, &ctx.struct_attr.ty);

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

fn render_enum_line(v: &Variant, ctx: &ImplContext) -> TokenStream {
    let attr = ApplicableAttr::get(&v.attrs, &ctx.kind, ctx.fallible, &ctx.struct_attr.ty);
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
        struct_kind: v.variant_kind,
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
        let block = if (!new_ctx.kind.is_from() && new_ctx.struct_attr.type_hint == TypeHint::Unit) || (new_ctx.kind.is_from() && variant_struct.struct_kind.is_unit()) {
            StructInitBlock {
                fragments: vec![],
                type_hint: TypeHint::Unit,
                struct_kind: StructKind::Unit,
                dst: None,
                ok_wrap: false,
            }
        } else {
            StructInitBlock {
                fragments: struct_init_block_fragments(&variant_struct, &new_ctx),
                type_hint: new_ctx.struct_attr.type_hint,
                struct_kind: variant_struct.struct_kind,
                dst: None,
                ok_wrap: false,
            }
        };
        block.render(&new_ctx.to_render_ctx())
    };

    match (v.variant_kind.is_struct(), attr, lit, pat, &ctx.kind) {
        (_, None, None, None, _) => {
            quote!(#src::#ident #destr => #dst::#ident #init,)
        },
        (_, Some(attr), None, None, Kind::FromOwned | Kind::FromRef) => {
            let member = Named(ident.clone());
            let right_side = attr.get_action_or(Some(&quote!(#ident)), &ctx.to_render_ctx(), || quote!(#dst::#ident #init));
            let ident2 = attr.get_field_name_or(&member);
            quote!(#src::#ident2 #destr => #right_side,)
        },
        (_, Some(attr), None, None, Kind::OwnedInto | Kind::RefInto) => {
            let member = Named(ident.clone());
            let right_side = attr.get_stuff(Some(&quote!(#dst::)), |x| quote!(#x #init), &ctx.to_render_ctx(), || &member);
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
            let right_side = attr.get_action_or(None, &ctx.to_render_ctx(), TokenStream::new);
            quote!(#src::#ident #destr => #right_side,)
        },
        _ => todo!(),
    }
}

fn render_enum_ghost_line(ghost_data: &GhostData, ctx: &ImplContext) -> TokenStream {
    let src = ctx.src_ty;
    let right_side = render_action(&ghost_data.action.expr, None, &ctx.to_render_ctx());

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
        },
    }
}

struct QuoteTraitParams<'a> {
    pub attr: Option<&'a TokenStream>,
    pub impl_attr: Option<&'a TokenStream>,
    pub inner_attr: Option<&'a TokenStream>,
    pub impl_gens: syn::Generics,
    pub where_clause: Option<TokenStream>,
    pub r: Option<TokenStream>,
}

fn get_quote_trait_params<'a>(input: &DataType, ctx: &'a ImplContext) -> QuoteTraitParams<'a> {
    let mut impl_gens = input.get_generics().clone();

    let these_lts: Vec<&Lifetime> = input.get_generics().params.iter()
        .filter_map(|g| match g {
            GenericParam::Lifetime(l) => Some(&l.lifetime),
            _ => None,
        }).collect();

    let those_lts: Vec<&Lifetime> = ctx.struct_attr.ty.generics.as_ref()
        .map(|g| {
            g.args.iter().filter_map(|g| match g {
                GenericArgument::Lifetime(l) => Some(l),
                _ => None,
            }).collect()
        }).unwrap_or_default();

    let ref_lts = ctx.kind.is_ref().then_some(if ctx.kind.is_from() { these_lts } else { those_lts.clone() }).unwrap_or_default();

    for lt in those_lts {
        let missing_lt = impl_gens.params.iter().all(|param| if let GenericParam::Lifetime(param) = param { &param.lifetime != lt } else { false });

        if missing_lt {
            let gen = GenericArgument::Lifetime(lt.clone());
            impl_gens.params.push(parse_quote!(#gen));
        }
    }

    if !ref_lts.is_empty() {
        let ind = impl_gens.params.iter().take_while(|p| matches!(p, GenericParam::Lifetime(_))).count();
        impl_gens.params.insert(ind, parse_quote!('o2o: #( #ref_lts )+*));
    }

    QuoteTraitParams {
        attr: ctx.struct_attr.attribute.as_ref(),
        impl_attr: ctx.struct_attr.impl_attribute.as_ref(),
        inner_attr: ctx.struct_attr.inner_attribute.as_ref(),
        impl_gens,
        where_clause: input.get_attrs().where_attr(&ctx.struct_attr.ty).map(|x| x.where_clause.to_token_stream()),
        r: ctx.kind.is_ref().then_some(if ref_lts.is_empty() { quote!(&) } else { quote!(&'o2o) }),
    }
}

fn quote_trait(input: &DataType, ctx: &mut ImplContext) -> TokenStream {
    let params = get_quote_trait_params(input, ctx);

    let err_ty = ctx.struct_attr.err_ty.as_ref().map(|x| &x.path);

    let these_gens = TheseGenerics { gens: input.get_generics() };
    let those_gens = ThoseGenerics { gens: &ctx.struct_attr.ty.generics };

    let main_code_block: Box<dyn Render> = match (&ctx.struct_attr.quick_return, ctx.input) {
        (Some(quick_return), _) => Box::new(Expression::new(&quick_return.expr.expr)),
        (_, DataType::Struct(s)) => {
            Box::new(StructInitBlock {
                fragments: struct_init_block_fragments(s, ctx),
                type_hint: ctx.struct_attr.type_hint,
                struct_kind: s.struct_kind,
                dst: (ctx.kind.is_from() || (!ctx.kind.is_into_existing() && !ctx.struct_attr.ty.nameless_tuple && !ctx.has_post_init)).then_some(ctx.dst_ty),
                ok_wrap: ctx.fallible && !ctx.kind.is_into_existing() && !ctx.has_post_init,
            })
        },
        (_, DataType::Enum(e)) => Box::new(EnumInit {
            temp: enum_main_code_block(e, ctx),
            ok_wrap: ctx.fallible && !ctx.kind.is_into_existing() && !ctx.has_post_init,
        }),
    };

    Implementation {
        impl_attr: params.impl_attr.cloned().map(|attr| Attribute { attr, inner: false }),
        err_ty,
        impl_gens: ImplGenerics { gens: params.impl_gens.clone() },
        these_gens: &these_gens,
        those_gens: &those_gens,
        where_clause: params.where_clause.clone().map(|where_clause| WhereClause { where_clause }),
        r: params.r.as_ref(),
        function: Function {
            attr: params.attr.cloned().map(|attr| Attribute { attr, inner: false }),
            inner_attr: params.inner_attr.cloned().map(|attr| Attribute { attr, inner: true }),
            these_gens: &these_gens,
            those_gens: &those_gens,
            body: FunctionBody {
                pre_init: ctx.struct_attr.init_data.as_ref().map(|init_data| PreInit {
                    vars: init_data.iter().map(|x| (Expression::new(&x.action.expr), &x.ident)).collect(),
                }),
                main_code_block: main_code_block.as_ref(),
                post_init_statements: get_post_init_statements(ctx),
            },
            err_ty,
            r: params.r.as_ref(),
        },
    }
    .render(&ctx.to_render_ctx())
}

fn struct_init_block_fragments<'a>(input: &'a Struct, ctx: &'a ImplContext) -> Vec<StructInitBlockFragment<'a>> {
    let mut group_paths = HashMap::<String, usize>::new();
    group_paths.insert("".into(), 0);

    let mut make_tuple = |path: String, field_data: FieldData<'a>| {
        if group_paths.contains_key(&path) {
            let gr_idx = *group_paths.get(&path).unwrap();
            (FieldContainer { gr_idx, path, field_data }, false)
        } else {
            group_paths.insert(path.clone(), group_paths.len());
            (FieldContainer { gr_idx: group_paths.len() - 1, path, field_data }, true)
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
            let (cnt, new_gr) = make_tuple(x.get_child_path_str(None).into(), FieldData::GhostData(x));
            new_gr.then_some(cnt)
        }));

    fields.sort_by(|a, b| a.gr_idx.cmp(&b.gr_idx));

    struct_init_block_fragments_inner(&mut fields.iter().peekable(), ctx, None)
}

fn struct_init_block_fragments_inner<'a>(
    members: &mut Peekable<Iter<FieldContainer<'a>>>,
    ctx: &'a ImplContext,
    field_ctx: Option<(&ChildPath, Option<&ChildParentData>, usize)>
) -> Vec<StructInitBlockFragment<'a>>
{
    let type_hint = ctx.struct_attr.type_hint;
    let type_hint = field_ctx.map_or(type_hint, |x| x.1.map_or(type_hint, |x| x.type_hint));

    let mut fragments = vec![];
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
                    Some(child_attr) => build_child_fragment(&child_attr.child_path, Some((f, idx)), members, ctx, field_ctx.map(|x| x.2), type_hint),
                    None => {
                        members.next();
                        vec![StructInitBlockFragment::Line(StructInitBlockLine { field: f, hint: type_hint, idx, parent_child: None, dst_ty: &ctx.struct_attr.ty, child_parent_expr: None })]
                    },
                };
                fragments.extend(fragment);
                idx += 1;
            },
            FieldData::GhostData(g) => {
                let child_path = &g.child_path.as_ref().unwrap();
                let fragment = build_child_fragment(child_path, None, members, ctx, field_ctx.map(|x| x.2), type_hint);
                fragments.extend(fragment);
                idx += 1;
            },
            FieldData::ParentChildField(f, p) => {
                let type_hint = if type_hint == TypeHint::Unspecified {
                    if ctx.input.struct_kind().is_struct() { TypeHint::Struct } else { TypeHint::Tuple }
                } else {
                    type_hint
                };
                let fragment = build_parent_child_fragment(f, p, members, p.named_fields(), ctx, field_ctx.map(|x| x.2), || StructInitBlockFragment::Line(StructInitBlockLine { field: f, hint: type_hint, idx, parent_child: Some(p), dst_ty: &ctx.struct_attr.ty, child_parent_expr: None }));
                fragments.push(fragment);
                idx += 1;
            },
        }
    }

    if !ctx.kind.is_from() {
        if let Some(ghost_attr) = ctx.input.get_attrs().ghosts_attr(&ctx.struct_attr.ty, &ctx.kind) {
            ghost_attr.ghost_data.iter().for_each(|x| match (&x.child_path, field_ctx) {
                (Some(_), Some(field_ctx)) => {
                    if x.get_child_path_str(None) == field_ctx.0.get_child_path_str(Some(field_ctx.2)) {
                        fragments.push(StructInitBlockFragment::Ghost(StructInitBlockGhost { child_path: x.child_path.as_ref(), ghost_ident: &x.ghost_ident, expr: Expression::new(&x.action.expr) }))
                    }
                },
                (None, None) => fragments.push(StructInitBlockFragment::Ghost(StructInitBlockGhost { child_path: x.child_path.as_ref(), ghost_ident: &x.ghost_ident, expr: Expression::new(&x.action.expr) })),
                _ => (),
            });
        }
    }

    if let Some(update) = &ctx.struct_attr.update {
        fragments.push(StructInitBlockFragment::Update(Expression::new(&update.expr.expr)))
    }

    fragments
}

fn build_child_fragment<'a>(
    child_path: &ChildPath,
    field: Option<(&'a Field, usize)>,
    fields: &mut Peekable<Iter<FieldContainer<'a>>>,
    ctx: &'a ImplContext,
    depth: Option<usize>,
    type_hint: TypeHint,
) -> Vec<StructInitBlockFragment<'a>>
{
    let get_line = ||{
        field.map(|(f, idx)| StructInitBlockLine {
            field: f, 
            hint: type_hint, 
            idx, 
            parent_child: None, 
            dst_ty: &ctx.struct_attr.ty,
            child_parent_expr: get_child_parent_expression(f, ctx)
        })
    };

    if depth.is_none() || depth.unwrap() < child_path.child_path_str.len() - 1 {
        let new_depth = depth.map_or(0, |x| x + 1);
        match ctx.kind {
            Kind::OwnedInto | Kind::RefInto => {
                let mut child_parents = ctx.input.get_attrs().child_parents_attr(&ctx.struct_attr.ty).unwrap().child_parents.iter();
                let child_data = child_parents.find(|child_data| child_data.check_match(child_path.get_child_path_str(Some(new_depth)))).unwrap();

                vec![build_child(child_data, fields, ctx.input.struct_kind(), ctx, (child_path, new_depth), type_hint)]
            },
            Kind::OwnedIntoExisting | Kind::RefIntoExisting => build_existing_child(fields, ctx, (child_path, new_depth)),
            Kind::FromOwned | Kind::FromRef => {
                fields.next();
                get_line().map(|x| StructInitBlockFragment::Line(x)).into_iter().collect()
            },
        }
    } else {
        fields.next();
        get_line().map(|x| StructInitBlockFragment::Line(x)).into_iter().collect()
    }
}

fn get_child_parent_expression<'a>(field: &'a Field, ctx: &'a ImplContext) -> Option<Expression<'a>> {
    field.attrs.child(&ctx.struct_attr.ty)
        .map(|child_attr| {
            let mut child_parent_attrs_found = false;

            let mut acc = Expression::new_owned(quote!(value));

            child_attr.child_path.child_path.iter().enumerate().for_each(|(idx, m)| {
                let child_parent_data_action = ctx.input.get_attrs()
                    .child_parents_attr(&ctx.struct_attr.ty)
                    .map(|child_parent_attr|
                        child_parent_attr.child_parents.iter()
                        .find(|h| h.check_match(&child_attr.child_path.child_path_str[idx]))).flatten()
                    .map(|xx| xx.actions.iter().find(|y| y.is_applicable(&ctx.kind)).map(|x| &x.action)).flatten();
                if let Some(action) = child_parent_data_action {
                    child_parent_attrs_found = true;
                    let mut old_acc = std::mem::replace(&mut acc, Expression::empty());
                    old_acc.postfix = Some(quote!(.#m));
                    acc = Expression::new_with_tilde(&action.expr, Box::new(old_acc));
                }
            });

            child_parent_attrs_found.then(|| acc)
        }).flatten()
}

fn build_parent_child_fragment<'a, F: FnOnce() -> StructInitBlockFragment<'a>>(
    field: &'a Field,
    parent_child_field: &'a ParentChildField,
    fields: &mut Peekable<Iter<FieldContainer<'a>>>,
    named_fields: bool,
    ctx: &'a ImplContext,
    depth: Option<usize>,
    build_line: F,
) -> StructInitBlockFragment<'a> {
    if depth.is_none() || depth.unwrap() < parent_child_field.sub_path.len() {
        let new_depth = depth.map_or(0, |x| x + 1);
        if ctx.kind.is_from() {
            let ty = if let Some(depth) = depth { parent_child_field.sub_path[depth].path.as_ref().unwrap() } else { field.ty.as_ref().unwrap() };
            let child_path = ChildPath::new(field.member.clone(), parent_child_field.sub_path.iter().map(|x| x.mem.clone()));
            StructInitBlockFragment::Child(StructInitBlockChild {
                block: StructInitBlock {
                    fragments: struct_init_block_fragments_inner(fields, ctx, Some((&child_path, None, new_depth))),
                    type_hint: ctx.struct_attr.type_hint,
                    struct_kind: if named_fields { StructKind::Struct } else { StructKind::Tuple },
                    dst: None,
                    ok_wrap: false,
                },
                name: child_path.child_path[new_depth].clone(),
                ty,
                action: None,
                struct_kind: ctx.input.struct_kind(),
                parent_hint: if ctx.input.struct_kind().is_struct() { TypeHint::Struct } else { TypeHint::Tuple },
            })
        } else {
            fields.next();
            build_line()
        }
    } else {
        fields.next();
        build_line()
    }
}

fn build_child<'a>(
    child_data: &'a ChildParentData,
    fields: &mut Peekable<Iter<FieldContainer<'a>>>,
    struct_kind: StructKind,
    ctx: &'a ImplContext,
    field_ctx: (&ChildPath, usize),
    hint: TypeHint,
) -> StructInitBlockFragment<'a> 
{
    let child_path = field_ctx.0;
    let child_name = child_path.child_path[field_ctx.1].clone();
    let ty = &child_data.ty;
    let action = child_data.actions.iter().find(|action| action.is_applicable(&ctx.kind));

    StructInitBlockFragment::Child(StructInitBlockChild {
        block: StructInitBlock {
            fragments: struct_init_block_fragments_inner(fields, ctx, Some((field_ctx.0, Some(child_data), field_ctx.1))),
            type_hint: child_data.type_hint,
            struct_kind,
            dst: None,
            ok_wrap: false,
        },
        name: child_name,
        ty,
        action,
        struct_kind: ctx.input.struct_kind(),
        parent_hint: hint,
    })
}

fn build_existing_child<'a>(
    fields: &mut Peekable<Iter<FieldContainer<'a>>>,
    ctx: &'a ImplContext,
    field_ctx: (&ChildPath, usize)
) -> Vec<StructInitBlockFragment<'a>>
{
    let child_attr = field_ctx.0;
    let path = child_attr.get_child_path_str(Some(field_ctx.1));
    let child_parents_attr = ctx.input.get_attrs().child_parents_attr(&ctx.struct_attr.ty);
    let child_data = child_parents_attr.and_then(|x| x.child_parents.iter().find(|child_data| child_data.check_match(path)));

    struct_init_block_fragments_inner(fields, ctx, Some((field_ctx.0, child_data, field_ctx.1)))
}

fn get_post_init_statements<'a>(ctx: &'a ImplContext) -> Vec<PostInitStatement<'a>> {
    if ctx.kind.is_from() {
        vec![]
    } else {
        ctx.input.get_members().iter()
            .filter(|mem| mem.get_attrs().has_parameterless_parent_attr(&ctx.struct_attr.ty))
            .map(|mem| match mem {
                DataTypeMember::Field(f) => PostInitStatement { member: &f.member },
                DataTypeMember::Variant(_) => todo!(),
            }).collect()
    }
}