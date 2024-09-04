use crate::{
    ast::{DataType, DataTypeMember, Struct, Variant},
    attr::{ChildAttr, ChildrenAttr, DataTypeAttrs, DataTypeInstruction, FallibleKind, GhostsAttr, Kind, MemberAttrs, MemberInstruction, TraitAttr, TraitAttrCore, TypeHint, TypePath, WhereAttr},
};
use proc_macro2::Span;
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use syn::{spanned::Spanned, Result};

pub(crate) fn validate(input: &DataType) -> Result<()> {
    let attrs = input.get_attrs();
    let mut errors: HashMap<String, Span> = HashMap::new();

    if attrs.attrs.is_empty() {
        errors.insert("At least one trait instruction is expected.".into(), Span::call_site());
    }

    validate_error_instrs(input, attrs, &mut errors);

    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::FromOwned, false), false, &mut errors);
    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::FromRef, false), false, &mut errors);
    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::OwnedInto, false), false, &mut errors);
    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::RefInto, false), false, &mut errors);
    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::OwnedIntoExisting, false), false, &mut errors);
    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::RefIntoExisting, false), false, &mut errors);

    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::FromOwned, true), true, &mut errors);
    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::FromRef, true), true, &mut errors);
    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::OwnedInto, true), true, &mut errors);
    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::RefInto, true), true, &mut errors);
    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::OwnedIntoExisting, true), true, &mut errors);
    validate_struct_attrs(attrs.iter_for_kind_core(&Kind::RefIntoExisting, true), true, &mut errors);

    let type_paths = attrs.attrs.iter().map(|x| &x.core.ty).collect::<HashSet<_>>();

    validate_ghost_attrs(&Kind::FromOwned, &attrs.ghosts_attrs, &type_paths, &mut errors);
    validate_ghost_attrs(&Kind::FromRef, &attrs.ghosts_attrs, &type_paths, &mut errors);
    validate_ghost_attrs(&Kind::OwnedInto, &attrs.ghosts_attrs, &type_paths, &mut errors);
    validate_ghost_attrs(&Kind::RefInto, &attrs.ghosts_attrs, &type_paths, &mut errors);
    validate_ghost_attrs(&Kind::OwnedIntoExisting, &attrs.ghosts_attrs, &type_paths, &mut errors);
    validate_ghost_attrs(&Kind::RefIntoExisting, &attrs.ghosts_attrs, &type_paths, &mut errors);

    validate_children_attrs(&attrs.children_attrs, &type_paths, &mut errors);
    validate_where_attrs(&attrs.where_attrs, &type_paths, &mut errors);

    for member in input.get_members() {
        let member_span = member.get_span();
        let member_attrs = member.get_attrs();

        validate_dedicated_member_attrs(&member_attrs.attrs, |x| x.attr.container_ty.as_ref(), None, member_span, &type_paths, &mut errors);
        validate_dedicated_member_attrs(&member_attrs.ghost_attrs, |x| x.attr.container_ty.as_ref(), None, member_span, &type_paths, &mut errors);

        match member {
            DataTypeMember::Field(f) => {
                bark_at_member_attr(&member_attrs.lit_attrs, "literal", |_| f.member.span(), &mut errors);
                bark_at_member_attr(&member_attrs.pat_attrs, "pattern", |_| f.member.span(), &mut errors);
                bark_at_member_attr(&member_attrs.type_hint_attrs, "type_hint", |_| f.member.span(), &mut errors);
                bark_at_member_attr(&member_attrs.ghosts_attrs.iter().filter(|x| x.applicable_to[&Kind::OwnedInto] && x.applicable_to[&Kind::RefInto]).collect(), "ghosts", |_| f.member.span(), &mut errors);
                bark_at_member_attr(&member_attrs.ghosts_attrs.iter().filter(|x| x.applicable_to[&Kind::OwnedInto] && !x.applicable_to[&Kind::RefInto]).collect(), "ghosts_owned", |_| f.member.span(), &mut errors);
                bark_at_member_attr(&member_attrs.ghosts_attrs.iter().filter(|x| !x.applicable_to[&Kind::OwnedInto] && x.applicable_to[&Kind::RefInto]).collect(), "ghosts_ref", |_| f.member.span(), &mut errors);

                validate_dedicated_member_attrs(&member_attrs.parent_attrs, |x| x.container_ty.as_ref(), Some("parent"), member_span, &type_paths, &mut errors);
            }
            DataTypeMember::Variant(v) => {
                bark_at_member_attr(&member_attrs.parent_attrs, "parent", |_| v.ident.span(), &mut errors);

                validate_dedicated_member_attrs(&member_attrs.lit_attrs, |x| x.container_ty.as_ref(), Some("literal"), member_span, &type_paths, &mut errors);
                validate_dedicated_member_attrs(&member_attrs.pat_attrs, |x| x.container_ty.as_ref(), Some("pattern"), member_span, &type_paths, &mut errors);
                validate_dedicated_member_attrs(&member_attrs.type_hint_attrs, |x| x.container_ty.as_ref(), Some("type_hint"), member_span, &type_paths, &mut errors);
            }
        }

        validate_member_error_instrs(input, member_attrs, &mut errors)
    }

    match input {
        DataType::Struct(s) => {
            validate_fields(s, attrs, &type_paths, &mut errors);
        }
        DataType::Enum(e) => {
            for v in &e.variants {
                validate_variant_fields(v, attrs, &type_paths, &mut errors);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let mut root_err = syn::Error::new(Span::call_site(), "Cannot expand o2o macro");

        errors.iter().for_each(|(err, sp)| root_err.combine(syn::Error::new(*sp, err)));

        Err(root_err)
    }
}

fn validate_error_instrs(input: &DataType, attrs: &DataTypeAttrs, errors: &mut HashMap<String, Span>) {
    let postfix = |own: bool| {
        if !own {
            " To turn this message off, use #[o2o(allow_unknown)]"
        } else {
            ""
        }
    };

    for err in &attrs.error_instrs {
        match (input, err) {
            (DataType::Enum(_), DataTypeInstruction::Misnamed { instr: instr @ "child", span, guess_name: _, own }) | (DataType::Enum(_), DataTypeInstruction::Misplaced { instr: instr @ ("parent" | "as_type"), span, own }) => {
                errors.insert(format!("Member instruction '{}' is not applicable to enums.{}", instr, postfix(*own)), *span);
            }
            (_, DataTypeInstruction::Misnamed { instr: _, span, guess_name, own }) => {
                errors.insert(format!("Perhaps you meant '{}'?{}", guess_name, postfix(*own)), *span);
            }
            (_, DataTypeInstruction::Misplaced { instr, span, own }) => {
                errors.insert(format!("Member instruction '{}' should be used on a member.{}", instr, postfix(*own)), *span);
            }
            (_, DataTypeInstruction::UnrecognizedWithError { instr, span }) => {
                errors.insert(format!("Struct instruction '{}' is not supported.", instr), *span);
            }
            _ => unreachable!("13"),
        }
    }
}

fn validate_member_error_instrs(input: &DataType, attrs: &MemberAttrs, errors: &mut HashMap<String, Span>) {
    let postfix = |own: bool| {
        if !own {
            " To turn this message off, use #[o2o(allow_unknown)]"
        } else {
            ""
        }
    };

    for err in &attrs.error_instrs {
        match (input, err) {
            (DataType::Enum(_), MemberInstruction::Misnamed { instr: instr @ "children", span, guess_name: _, own }) => {
                errors.insert(format!("Struct instruction '{}' is not applicable to enums.{}", instr, postfix(*own)), *span);
            }
            (_, MemberInstruction::Misnamed { instr: _, span, guess_name, own }) => {
                errors.insert(format!("Perhaps you meant '{}'?{}", guess_name, postfix(*own)), *span);
            }
            (_, MemberInstruction::Misplaced { instr, span, own }) => {
                errors.insert(format!("Struct instruction '{}' should be used on a struct.{}", instr, postfix(*own)), *span);
            }
            (_, MemberInstruction::UnrecognizedWithError { instr, span }) => {
                errors.insert(format!("Member instruction '{}' is not supported.", instr), *span);
            }
            _ => unreachable!("14"),
        }
    }
}

fn validate_struct_attrs<'a, I: Iterator<Item = &'a TraitAttrCore>>(attrs: I, fallible: bool, errors: &mut HashMap<String, Span>) {
    let mut unique_ident = HashSet::new();
    for attr in attrs {
        if !unique_ident.insert(&attr.ty) {
            errors.insert("Ident here must be unique.".into(), attr.ty.span);
        }

        if fallible && attr.err_ty.is_none() {
            errors.insert("Error type should be specified for fallible instruction.".into(), attr.ty.span);
        }

        if !fallible && attr.err_ty.is_some() {
            errors.insert("Error type should not be specified for infallible instruction.".into(), attr.err_ty.as_ref().unwrap().span);
        }
    }
}

fn validate_ghost_attrs(kind: &Kind, ghost_attrs: &[GhostsAttr], type_paths: &HashSet<&TypePath>, errors: &mut HashMap<String, Span>) {
    if ghost_attrs.iter().filter(|x| x.applicable_to[kind] && x.attr.container_ty.is_none()).count() > 1 {
        errors.insert("There can be at most one default #[ghosts(...)] instruction.".into(), Span::call_site());
    }

    let mut unique_dedicated_attr_type_path = HashSet::new();

    for ghost_attr in ghost_attrs.iter().filter(|x| x.applicable_to[kind] && x.attr.container_ty.is_some()) {
        let tp = ghost_attr.attr.container_ty.as_ref().unwrap();
        if !type_paths.contains(tp) {
            errors.insert(format!("Type '{}' doesn't match any type specified in trait instructions.", tp.path_str), tp.span);
        }
        if !unique_dedicated_attr_type_path.insert(tp) {
            errors.insert(format!("Dedicated #[ghosts(...)] instruction for type {} is already defined.", tp.path_str), tp.span);
        }
    }
}

fn validate_children_attrs(children_attrs: &[ChildrenAttr], type_paths: &HashSet<&TypePath>, errors: &mut HashMap<String, Span>) {
    if children_attrs.iter().filter(|x| x.container_ty.is_none()).count() > 1 {
        errors.insert("There can be at most one default #[children(...)] instruction.".into(), Span::call_site());
    }

    let mut unique_dedicated_attr_type_path = HashSet::new();

    for children_attr in children_attrs.iter() {
        if let Some(tp) = &children_attr.container_ty {
            if !type_paths.contains(tp) {
                errors.insert(format!("Type '{}' doesn't match any type specified in trait instructions.", tp.path_str), tp.span);
            }
            if !unique_dedicated_attr_type_path.insert(tp) {
                errors.insert(format!("Dedicated #[children(...)] instruction for type {} is already defined.", tp.path_str), tp.span);
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
        if let Some(tp) = &where_attr.container_ty {
            if !type_paths.contains(tp) {
                errors.insert(format!("Type '{}' doesn't match any type specified in trait instructions.", tp.path_str), tp.span);
            }
            if !unique_dedicated_attr_type_path.insert(tp) {
                errors.insert(format!("Dedicated #[where_clause(...)] instruction for type {} is already defined.", tp.path_str), tp.span);
            }
        }
    }
}

fn bark_at_member_attr<T, U: Fn(&T) -> Span>(attrs: &Vec<T>, instr_name: &'static str, extract_span: U, errors: &mut HashMap<String, Span>) {
    for attr in attrs {
        errors.insert(format!("Instruction #[{}(...)] is not supported for this member.", instr_name), extract_span(attr));
    }
}

fn validate_dedicated_member_attrs<T, U: Fn(&T) -> Option<&TypePath>>(attrs: &Vec<T>, extract_type_path: U, instr_name: Option<&'static str>, member_span: Span, type_paths: &HashSet<&TypePath>, errors: &mut HashMap<String, Span>) {
    if let Some(inst_name) = instr_name {
        if attrs.iter().filter(|x| extract_type_path(x).is_none()).count() > 1 {
            errors.insert(format!("There can be at most one default #[{}(...)] instruction for a given member.", inst_name), member_span);
        }
    }

    let mut unique_dedicated_attr_type_path = HashSet::new();

    for attr in attrs {
        if let Some(tp) = extract_type_path(attr) {
            if !type_paths.contains(tp) {
                errors.insert(format!("Type '{}' doesn't match any type specified in trait instructions.", tp.path_str), tp.span);
            }
            if let Some(inst_name) = instr_name {
                if !unique_dedicated_attr_type_path.insert(tp) {
                    errors.insert(format!("Dedicated #[{}(...)] instruction for type {} is already defined.", inst_name, tp.path_str), tp.span);
                }
            }
        }
    }
}

fn validate_fields(input: &Struct, data_type_attrs: &DataTypeAttrs, type_paths: &HashSet<&TypePath>, errors: &mut HashMap<String, Span>) {
    let into_type_paths = data_type_attrs
        .iter_for_kind_core(&Kind::OwnedInto, false)
        .chain(data_type_attrs.iter_for_kind_core(&Kind::RefInto, false))
        .chain(data_type_attrs.iter_for_kind_core(&Kind::OwnedInto, true))
        .chain(data_type_attrs.iter_for_kind_core(&Kind::RefInto, true))
        .map(|x| &x.ty)
        .collect::<HashSet<_>>();

    let from_type_paths = data_type_attrs
        .iter_for_kind_core(&Kind::FromOwned, false)
        .chain(data_type_attrs.iter_for_kind_core(&Kind::FromRef, false))
        .chain(data_type_attrs.iter_for_kind_core(&Kind::FromOwned, true))
        .chain(data_type_attrs.iter_for_kind_core(&Kind::FromRef, true))
        .filter(|x| x.update.is_none())
        .map(|x| &x.ty)
        .collect::<HashSet<_>>();

    for field in &input.fields {
        for ghost_attr in field.attrs.ghost_attrs.iter() {
            if ghost_attr.attr.action.is_some() {
                continue;
            }
            match &ghost_attr.attr.container_ty {
                Some(tp) => {
                    if from_type_paths.contains(tp) {
                        errors.insert(format!("Member instruction #[ghost(...)] for member '{}' should provide default value for type {}", field.member.to_token_stream(), tp.path_str), field.member.span());
                    }
                }
                None => {
                    let field_name_str = field.member.to_token_stream().to_string();
                    for tp in from_type_paths.iter() {
                        errors.insert(format!("Member instruction #[ghost(...)] for member '{}' should provide default value for type {}", field_name_str, tp.path_str), field.member.span());
                    }
                }
            }
        }

        if let Some(repeat_attr) = &field.attrs.repeat {
            if repeat_attr.permeate {
                errors.insert("Permeating repeat instruction is only applicable to enum variant fields.".into(), field.member.span());
            }
        }
    }

    for child_attr in input.fields.iter().flat_map(|x| &x.attrs.child_attrs) {
        match &child_attr.container_ty {
            Some(tp) => {
                if !type_paths.contains(tp) {
                    errors.insert(format!("Type '{}' doesn't match any type specified in trait instructions.", tp.path_str), tp.span);
                }
                if into_type_paths.contains(tp) {
                    check_child_errors(child_attr, data_type_attrs, tp, errors)
                }
            }
            None => {
                for tp in into_type_paths.iter() {
                    check_child_errors(child_attr, data_type_attrs, tp, errors)
                }
            }
        }
    }

    if !input.named_fields {
        let data_type_attrs: Vec<(&TraitAttrCore, Kind)> = data_type_attrs
            .iter_for_kind_core(&Kind::OwnedInto, false)
            .map(|x| (x, Kind::OwnedInto))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::RefInto, false).map(|x| (x, Kind::RefInto)))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::OwnedIntoExisting, false).map(|x| (x, Kind::OwnedIntoExisting)))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::RefIntoExisting, false).map(|x| (x, Kind::RefIntoExisting)))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::FromOwned, false).map(|x| (x, Kind::FromOwned)))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::FromRef, false).map(|x| (x, Kind::FromRef)))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::OwnedInto, true).map(|x| (x, Kind::OwnedInto)))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::RefInto, true).map(|x| (x, Kind::RefInto)))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::OwnedIntoExisting, true).map(|x| (x, Kind::OwnedIntoExisting)))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::RefIntoExisting, true).map(|x| (x, Kind::RefIntoExisting)))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::FromOwned, true).map(|x| (x, Kind::FromOwned)))
            .chain(data_type_attrs.iter_for_kind_core(&Kind::FromRef, true).map(|x| (x, Kind::FromRef)))
            .collect();

        for (data_type_attr, kind) in data_type_attrs {
            if data_type_attr.quick_return.is_none() && data_type_attr.type_hint == TypeHint::Struct {
                for field in &input.fields {
                    if field.attrs.ghost(&data_type_attr.ty, &kind).is_some() || field.attrs.has_parent_attr(&data_type_attr.ty) {
                        continue;
                    }

                    if let Some(field_attr) = field.attrs.applicable_field_attr(&kind, false, &data_type_attr.ty) {
                        if kind == Kind::FromOwned || kind == Kind::FromRef {
                            if field_attr.attr.member.is_none() && field_attr.attr.action.is_none() {
                                errors.insert(format!("Member trait instruction #[{}(...)] for member {} should specify corresponding field name of the {} or an action", field_attr.original_instr, field.member.to_token_stream(), data_type_attr.ty.path), field.member.span());
                            }
                        } else if field_attr.attr.member.is_none() {
                            errors.insert(format!("Member trait instruction #[{}(...)] for member {} should specify corresponding field name of the {}", field_attr.original_instr, field.member.to_token_stream(), data_type_attr.ty.path_str), field.member.span());
                        }
                    } else {
                        errors.insert(
                            format!(
                                "Member {} should have member trait instruction with field name{}, that corresponds to #[{}({}...)] trait instruction",
                                field.member.to_token_stream(),
                                if kind == Kind::FromOwned || kind == Kind::FromRef { " or an action" } else { "" },
                                FallibleKind(kind, false),
                                data_type_attr.ty.path_str
                            ),
                            field.member.span(),
                        );
                    }
                }
            }
        }
    }
}

fn validate_variant_fields(input: &Variant, data_type_attrs: &DataTypeAttrs, _type_paths: &HashSet<&TypePath>, errors: &mut HashMap<String, Span>) {
    if !input.named_fields {
        let data_type_attrs: Vec<(&TraitAttr, Kind)> = data_type_attrs
            .iter_for_kind(&Kind::OwnedInto, false)
            .map(|x| (x, Kind::OwnedInto))
            .chain(data_type_attrs.iter_for_kind(&Kind::RefInto, false).map(|x| (x, Kind::RefInto)))
            .chain(data_type_attrs.iter_for_kind(&Kind::OwnedIntoExisting, false).map(|x| (x, Kind::OwnedIntoExisting)))
            .chain(data_type_attrs.iter_for_kind(&Kind::RefIntoExisting, false).map(|x| (x, Kind::RefIntoExisting)))
            .chain(data_type_attrs.iter_for_kind(&Kind::FromOwned, false).map(|x| (x, Kind::FromOwned)))
            .chain(data_type_attrs.iter_for_kind(&Kind::FromRef, false).map(|x| (x, Kind::FromRef)))
            .chain(data_type_attrs.iter_for_kind(&Kind::OwnedInto, true).map(|x| (x, Kind::OwnedInto)))
            .chain(data_type_attrs.iter_for_kind(&Kind::RefInto, true).map(|x| (x, Kind::RefInto)))
            .chain(data_type_attrs.iter_for_kind(&Kind::OwnedIntoExisting, true).map(|x| (x, Kind::OwnedIntoExisting)))
            .chain(data_type_attrs.iter_for_kind(&Kind::RefIntoExisting, true).map(|x| (x, Kind::RefIntoExisting)))
            .chain(data_type_attrs.iter_for_kind(&Kind::FromOwned, true).map(|x| (x, Kind::FromOwned)))
            .chain(data_type_attrs.iter_for_kind(&Kind::FromRef, true).map(|x| (x, Kind::FromRef)))
            .collect();

        for (data_type_attr, kind) in data_type_attrs {
            if data_type_attr.core.quick_return.is_none() && input.attrs.type_hint(&data_type_attr.core.ty).map_or(TypeHint::Unspecified, |x| x.type_hint) == TypeHint::Struct {
                for field in &input.fields {
                    if field.attrs.ghost(&data_type_attr.core.ty, &kind).is_some() || field.attrs.has_parent_attr(&data_type_attr.core.ty) {
                        continue;
                    }

                    if let Some(field_attr) = field.attrs.applicable_field_attr(&kind, false, &data_type_attr.core.ty) {
                        if kind == Kind::FromOwned || kind == Kind::FromRef {
                            if field_attr.attr.member.is_none() && field_attr.attr.action.is_none() {
                                errors.insert(format!("Member trait instruction #[{}(...)] for member {} should specify corresponding field name of the {} or an action", field_attr.original_instr, field.member.to_token_stream(), data_type_attr.core.ty.path), field.member.span());
                            }
                        } else if field_attr.attr.member.is_none() {
                            errors.insert(format!("Member trait instruction #[{}(...)] for member {} should specify corresponding field name of the {}", field_attr.original_instr, field.member.to_token_stream(), data_type_attr.core.ty.path_str), field.member.span());
                        }
                    } else {
                        errors.insert(
                            format!(
                                "Member {} of a variant {} should have member trait instruction with field name{}, that corresponds to #[{}({}...)] trait instruction",
                                field.member.to_token_stream(),
                                input.ident,
                                if kind == Kind::FromOwned || kind == Kind::FromRef { " or an action" } else { "" },
                                FallibleKind(kind, data_type_attr.fallible),
                                data_type_attr.core.ty.path_str
                            ),
                            field.member.span(),
                        );
                    }
                }
            }
        }
    }
}

fn check_child_errors(child_attr: &ChildAttr, struct_attrs: &DataTypeAttrs, tp: &TypePath, errors: &mut HashMap<String, Span>) {
    let children_attr = struct_attrs.children_attr(tp);
    for (idx, _level) in child_attr.child_path.child_path.iter().enumerate() {
        let path = child_attr.get_child_path_str(Some(idx));
        match children_attr {
            Some(children_attr) => {
                if !children_attr.children.iter().any(|x| x.check_match(path)) {
                    errors.insert(format!("Missing '{}: [Type Path]' instruction for type {}", path, tp.path_str), tp.span);
                }
            }
            None => {
                errors.insert(format!("Missing #[children(...)] instruction for {}", tp.path_str), tp.span);
            }
        }
    }
}
