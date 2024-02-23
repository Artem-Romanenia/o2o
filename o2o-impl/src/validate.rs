use std::collections::{HashMap, HashSet};
use proc_macro2::Span;
use quote::ToTokens;
use syn::{spanned::Spanned, Result};
use crate::{ast::Struct, attr::{ChildrenAttr, FieldChildAttr, Kind, StructAttrCore, StructAttrs, StructGhostAttr, StructKindHint, TypePath, WhereAttr}};

pub(crate) fn validate(input: &Struct) -> Result<()> {
    let mut errors: HashMap<String, Span> = HashMap::new();

    if input.attrs.attrs.is_empty() {
        errors.insert("At least one trait instruction is expected.".into(), Span::call_site());
    }

    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::FromOwned), &mut errors);
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::FromRef), &mut errors);
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::OwnedInto), &mut errors);
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::RefInto), &mut errors);
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::OwnedIntoExisting), &mut errors);
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::RefIntoExisting), &mut errors);

    let type_paths = input.attrs.attrs.iter()
        .map(|x| &x.attr.ty)
        .collect::<HashSet<_>>();

    validate_ghost_attrs(&Kind::FromOwned, &input.attrs.ghost_attrs, &type_paths, &mut errors);
    validate_ghost_attrs(&Kind::FromRef, &input.attrs.ghost_attrs, &type_paths, &mut errors);
    validate_ghost_attrs(&Kind::OwnedInto, &input.attrs.ghost_attrs, &type_paths, &mut errors);
    validate_ghost_attrs(&Kind::RefInto, &input.attrs.ghost_attrs, &type_paths, &mut errors);
    validate_ghost_attrs(&Kind::OwnedIntoExisting, &input.attrs.ghost_attrs, &type_paths, &mut errors);
    validate_ghost_attrs(&Kind::RefIntoExisting, &input.attrs.ghost_attrs, &type_paths, &mut errors);

    validate_children_attrs(&input.attrs.children_attrs, &type_paths, &mut errors);
    validate_where_attrs(&input.attrs.where_attrs, &type_paths, &mut errors);

    validate_fields(input, &input.attrs, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        let mut root_err = syn::Error::new(Span::call_site(), "Cannot expand o2o macro");

        errors.iter().for_each(|(err, sp)| root_err.combine(syn::Error::new(*sp, err)));

        Err(root_err)
    }
}

fn validate_struct_attrs<'a, I: Iterator<Item = &'a StructAttrCore>>(attrs: I, errors: &mut HashMap<String, Span>) {
    let mut unique_ident = HashSet::new();
    for attr in attrs {
        if !unique_ident.insert(&attr.ty) {
            errors.insert("Ident here must be unique.".into(), attr.ty.span);
        }
    }
}

fn validate_ghost_attrs(kind: &Kind, ghost_attrs: &[StructGhostAttr], type_paths: &HashSet<&TypePath>, errors: &mut HashMap<String, Span>) {
    if ghost_attrs.iter().filter(|x|x.applicable_to[kind] && x.attr.container_ty.is_none()).count() > 1 {
        errors.insert("There can be at most one default #[ghost(...)] instruction.".into(), Span::call_site());
    }

    let mut unique_dedicated_attr_type_path = HashSet::new();

    for ghost_attr in ghost_attrs.iter().filter(|x|x.applicable_to[kind] && x.attr.container_ty.is_some()) {
        let tp = ghost_attr.attr.container_ty.as_ref().unwrap();
        if !type_paths.contains(tp) {
            errors.insert("Type {} doesn't match any type specified in trait instructions.".into(), tp.span);
        }
        if !unique_dedicated_attr_type_path.insert(tp) {
            errors.insert("Dedicated #[ghost(...)] instruction for  type {} is already defined.".into(), tp.span);
        }
    }
}

fn validate_children_attrs(children_attrs: &[ChildrenAttr], type_paths: &HashSet<&TypePath>, errors: &mut HashMap<String, Span>) {
    if children_attrs.iter().filter(|x| x.container_ty.is_none()).count() > 1 {
        errors.insert("There can be at most one default #[children(...)] instruction.".into(), Span::call_site());
    }

    let mut unique_dedicated_attr_type_path = HashSet::new();

    for children_attr in children_attrs.iter() {
        if let Some(tp) = &children_attr.container_ty{
            if !type_paths.contains(tp) {
                errors.insert("Type {} doesn't match any type specified in trait instructions.".into(), tp.span);
            }
            if !unique_dedicated_attr_type_path.insert(tp) {
                errors.insert("Dedicated #[children(...)] instruction for  type {} is already defined.".into(), tp.span);
            }
        }

        let mut unique_field = HashSet::new();

        for child_data in &children_attr.children {
            if !unique_field.insert(child_data) {
                errors.insert("Ident here must be unique.".into(), child_data.field_path.span());
            }
        }
    }
}

fn validate_where_attrs(where_attrs: &[WhereAttr], type_paths: &HashSet<&TypePath>, errors: &mut HashMap<String, Span>) {
    if where_attrs.iter().filter(|x| x.container_ty.is_none()).count() > 1 {
        errors.insert("There can be at most one default #[where_clause(...)] instruction.".into(), Span::call_site());
    }

    let mut unique_dedicated_attr_type_path = HashSet::new();

    for where_attr in where_attrs.iter() {
        if let Some(tp) = &where_attr.container_ty{
            if !type_paths.contains(tp) {
                errors.insert("Type {} doesn't match any type specified in trait instructions.".into(), tp.span);
            }
            if !unique_dedicated_attr_type_path.insert(tp) {
                errors.insert("Dedicated #[where_clause(...)] instruction for  type {} is already defined.".into(), tp.span);
            }
        }
    }
}

fn validate_fields(input: &Struct, struct_attrs: &StructAttrs, errors: &mut HashMap<String, Span>) {
    let into_type_paths = struct_attrs.iter_for_kind(&Kind::OwnedInto)
        .chain(struct_attrs.iter_for_kind(&Kind::RefInto))
        .map(|x| &x.ty)
        .collect::<HashSet<_>>();

    for child_attr in input.fields.iter().flat_map(|x| &x.attrs.child_attrs) {
        match &child_attr.container_ty {
            Some(tp) => check_child_errors(child_attr, struct_attrs, tp, errors),
            None => for tp in into_type_paths.iter(){
                check_child_errors(child_attr, struct_attrs, tp, errors)
            }
        }
    }

    let from_type_paths = struct_attrs.iter_for_kind(&Kind::FromOwned)
        .chain(struct_attrs.iter_for_kind(&Kind::FromRef))
        .map(|x| &x.ty)
        .collect::<HashSet<_>>();

    for field in &input.fields {
        for ghost_attr in field.attrs.ghost_attrs.iter() {
            if ghost_attr.attr.action.is_some() {
                continue;
            }
            match &ghost_attr.attr.container_ty {
                Some(tp) => {
                    if from_type_paths.contains(tp)  {
                        errors.insert(format!("Member instruction #[ghost(...)] for member '{}' should provide default value for type {}", field.member.to_token_stream(), tp.path_str), field.member.span());
                    }
                },
                None => {
                    let field_name_str = field.member.to_token_stream().to_string();
                    for tp in from_type_paths.iter(){
                        errors.insert(format!("Member instruction #[ghost(...)] for member '{}' should provide default value for type {}", field_name_str, tp.path_str), field.member.span());
                    }
                }
            }
        }
    }

    if !input.named_fields {
        let struct_attrs = struct_attrs.iter_for_kind(&Kind::OwnedInto).map(|x| (x, Kind::OwnedInto))
            .chain(struct_attrs.iter_for_kind(&Kind::RefInto).map(|x| (x, Kind::RefInto)))
            .chain(struct_attrs.iter_for_kind(&Kind::OwnedIntoExisting).map(|x| (x, Kind::OwnedIntoExisting)))
            .chain(struct_attrs.iter_for_kind(&Kind::RefIntoExisting).map(|x| (x, Kind::RefIntoExisting)))
            .chain(struct_attrs.iter_for_kind(&Kind::FromOwned).map(|x| (x, Kind::FromOwned)))
            .chain(struct_attrs.iter_for_kind(&Kind::FromRef).map(|x| (x, Kind::FromRef)));

        for (struct_attr, kind) in struct_attrs {
            if input.attrs.wrapped_attr(&struct_attr.ty).is_none() && struct_attr.struct_kind_hint == StructKindHint::Struct {
                for field in &input.fields {
                    if field.attrs.ghost(&struct_attr.ty, &kind).is_some() || field.attrs.has_parent_attr(&struct_attr.ty) {
                        continue;
                    }

                    if let Some(field_attr) = field.attrs.applicable_field_attr(&kind, &struct_attr.ty) {
                        if kind == Kind::FromOwned || kind == Kind::FromRef {
                            if !field_attr.attr.wrapper && field_attr.attr.member.is_none() && field_attr.attr.action.is_none() {
                                errors.insert(format!("Member trait instruction #[{}(...)] for member {} should specify corresponding field name of the {} or an action", field_attr.original_instr, field.member.to_token_stream(), struct_attr.ty.path), field.member.span());
                            }
                        } else if !field_attr.attr.wrapper && field_attr.attr.member.is_none() {
                            errors.insert(format!("Member trait instruction #[{}(...)] for member {} should specify corresponding field name of the {}", field_attr.original_instr, field.member.to_token_stream(), struct_attr.ty.path_str), field.member.span());
                        }
                    } else {
                        errors.insert(format!("Member {} should have member trait instruction with field name{}, that corresponds to #[{}({}...)] trait instruction", field.member.to_token_stream(), if kind == Kind::FromOwned || kind == Kind::FromRef { " or an action" } else { "" }, kind, struct_attr.ty.path_str), field.member.span());
                    }
                }
            }
        }
    }
}

fn check_child_errors(child_attr: &FieldChildAttr, struct_attrs: &StructAttrs, tp: &TypePath, errors: &mut HashMap<String, Span>) {
    let children_attr = struct_attrs.children_attr(tp);
    for (idx, _level) in child_attr.child_path.child_path.iter().enumerate() {
        let path = child_attr.get_child_path_str(Some(idx));
            match children_attr {
                Some(children_attr) => {
                    if !children_attr.children.iter().any(|x| x.check_match(path)) {
                        errors.insert(format!("Missing '{}: [Type Path]' instruction for type {}", path, tp.path_str), tp.span);
                    }
                },
                None => { errors.insert(format!("Missing #[children(...)] instruction for {}", tp.path_str), tp.span); }
            }
    }
}