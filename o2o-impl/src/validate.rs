use std::collections::HashSet;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{Result, spanned::Spanned};
use crate::{ast::{Struct, Field}, attr::{Kind, StructAttrCore, StructAttrs, FieldChildAttr, TypePath, ChildrenAttr, WhereAttr, StructGhostAttr}};

pub(crate) fn validate(input: &Struct) -> Result<()> {
    if input.attrs.attrs.is_empty() {
        return Err(syn::Error::new(Span::call_site(), "At least one 'map'-like struct level instruction is expected."))
    }

    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::FromOwned))?;
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::FromRef))?;
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::OwnedInto))?;
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::RefInto))?;
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::OwnedIntoExisting))?;
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::RefIntoExisting))?;

    let type_paths = input.attrs.attrs.iter()
        .map(|x| &x.attr.ty)
        .collect::<HashSet<_>>();

    validate_ghost_attrs(&Kind::FromOwned, &input.attrs.ghost_attrs, &type_paths)?;
    validate_ghost_attrs(&Kind::FromRef, &input.attrs.ghost_attrs, &type_paths)?;
    validate_ghost_attrs(&Kind::OwnedInto, &input.attrs.ghost_attrs, &type_paths)?;
    validate_ghost_attrs(&Kind::RefInto, &input.attrs.ghost_attrs, &type_paths)?;
    validate_ghost_attrs(&Kind::OwnedIntoExisting, &input.attrs.ghost_attrs, &type_paths)?;
    validate_ghost_attrs(&Kind::RefIntoExisting, &input.attrs.ghost_attrs, &type_paths)?;

    validate_children_attrs(&input.attrs.children_attrs, &type_paths)?;
    validate_where_attrs(&input.attrs.where_attrs, &type_paths)?;

    validate_fields(&input.fields, &input.attrs)?;
    Ok(())
}

fn validate_struct_attrs<'a, I>(attrs: I) -> Result<()> 
    where I: Iterator<Item = &'a StructAttrCore>
{
    let mut unique_ident = HashSet::new();
    for attr in attrs {
        if !unique_ident.insert(&attr.ty) {
            return Err(syn::Error::new(attr.ty.span, "Ident here must be unique."))
        }
    }
    Ok(())
}

fn validate_ghost_attrs(kind: &Kind, ghost_attrs: &[StructGhostAttr], type_paths: &HashSet<&TypePath>) -> Result<()> {
    if ghost_attrs.iter().filter(|x|x.applicable_to[kind] && x.attr.container_ty.is_none()).count() > 1 {
        return Err(syn::Error::new(Span::call_site(), "There can be at most one default #[ghost(...)] instruction."))
    }

    let mut unique_dedicated_attr_type_path = HashSet::new();

    for ghost_attr in ghost_attrs.iter().filter(|x|x.applicable_to[kind] && x.attr.container_ty.is_some()) {
        let tp = ghost_attr.attr.container_ty.as_ref().unwrap();
        if !type_paths.contains(tp) {
            return Err(syn::Error::new(tp.span, format!("Type {} doesn't match any type specified in 'map'-like struct level instructions.", tp.path_str)))
        }
        if !unique_dedicated_attr_type_path.insert(tp) {
            return Err(syn::Error::new(tp.span, format!("Dedicated #[ghost(...)] instruction for  type {} is already defined.", tp.path_str)))
        }
    }
    Ok(())
}

fn validate_children_attrs(children_attrs: &[ChildrenAttr], type_paths: &HashSet<&TypePath>) -> Result<()> {
    if children_attrs.iter().filter(|x| x.container_ty.is_none()).count() > 1 {
        return Err(syn::Error::new(Span::call_site(), "There can be at most one default #[children(...)] instruction."))
    }

    let mut unique_dedicated_attr_type_path = HashSet::new();

    for children_attr in children_attrs.iter() {
        if let Some(tp) = &children_attr.container_ty{
            if !type_paths.contains(tp) {
                return Err(syn::Error::new(tp.span, format!("Type {} doesn't match any type specified in 'map'-like struct level instructions.", tp.path_str)))
            }
            if !unique_dedicated_attr_type_path.insert(tp) {
                return Err(syn::Error::new(tp.span, format!("Dedicated #[children(...)] instruction for  type {} is already defined.", tp.path_str)))
            }
        }

        let mut unique_field = HashSet::new();

        for child_data in &children_attr.children {
            if !unique_field.insert(child_data) {
                return Err(syn::Error::new(child_data.field_path.span(), "Ident here must be unique."))
            }
        }
    }
    Ok(())
}

fn validate_where_attrs(where_attrs: &[WhereAttr], type_paths: &HashSet<&TypePath>) -> Result<()> {
    if where_attrs.iter().filter(|x| x.container_ty.is_none()).count() > 1 {
        return Err(syn::Error::new(Span::call_site(), "There can be at most one default #[where_clause(...)] instruction."))
    }

    let mut unique_dedicated_attr_type_path = HashSet::new();

    for where_attr in where_attrs.iter() {
        if let Some(tp) = &where_attr.container_ty{
            if !type_paths.contains(tp) {
                return Err(syn::Error::new(tp.span, format!("Type {} doesn't match any type specified in 'map'-like struct level instructions.", tp.path_str)))
            }
            if !unique_dedicated_attr_type_path.insert(tp) {
                return Err(syn::Error::new(tp.span, format!("Dedicated #[where_clause(...)] instruction for  type {} is already defined.", tp.path_str)))
            }
        }
    }
    Ok(())
}

fn validate_fields(fields: &[Field], struct_attrs: &StructAttrs) -> Result<()> {
    let into_type_paths = struct_attrs.iter_for_kind(&Kind::OwnedInto)
        .chain(struct_attrs.iter_for_kind(&Kind::RefInto))
        .map(|x| &x.ty)
        .collect::<HashSet<_>>();

    let from_type_paths = struct_attrs.iter_for_kind(&Kind::FromOwned)
        .chain(struct_attrs.iter_for_kind(&Kind::FromRef))
        .map(|x| &x.ty)
        .collect::<HashSet<_>>();

    let mut errors = HashSet::new();

    for child_attr in fields.iter().flat_map(|x| &x.attrs.child_attrs) {
        match &child_attr.container_ty {
            Some(tp) => check_child_errors(child_attr, struct_attrs, tp, &mut errors),
            None => for tp in into_type_paths.iter(){
                check_child_errors(child_attr, struct_attrs, tp, &mut errors)
            }
        }
    }

    for field in fields {
        for ghost_attr in field.attrs.ghost_attrs.iter() {
            if ghost_attr.attr.action.is_some() {
                continue;
            }
            match &ghost_attr.attr.container_ty {
                Some(tp) => {
                    if  from_type_paths.contains(tp)  {
                        errors.insert(format!("Member level instruction #[ghost(...)] should provide default value for field '{}' for type {}", field.member.to_token_stream(), tp.path_str));
                    }
                },
                None => {
                    let field_name_str = field.member.to_token_stream().to_string();
                    for tp in from_type_paths.iter(){
                        errors.insert(format!("Member level instruction #[ghost(...)] should provide default value for field '{}' for type {}", field_name_str, tp.path_str));
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        let errors: Vec<String> = errors.into_iter().collect();
        return Err(syn::Error::new(Span::call_site(), errors.join("\n")))
    }

    Ok(())
}

fn check_child_errors(child_attr: &FieldChildAttr, struct_attrs: &StructAttrs, tp: &TypePath, errors: &mut HashSet<String>) {
    let children_attr = struct_attrs.children_attr(tp);
    for (idx, _level) in child_attr.child_path.child_path.iter().enumerate() {
        let path = child_attr.get_child_path_str(Some(idx));
            match children_attr {
                Some(children_attr) => {
                    if !children_attr.children.iter().any(|x| x.check_match(path)) {
                        errors.insert(format!("Missing '{}: [Type Path]' instruction for type {}", path, tp.path_str));
                    }
                },
                None => { errors.insert(format!("Missing #[children(...)] instruction for {}", tp.path_str)); }
            }
    }
}