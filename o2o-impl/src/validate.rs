use std::collections::HashSet;
use syn::{Result, spanned::Spanned};
use crate::{ast::Struct, attr::{StructAttr, FieldAttr, FieldChildAttr, Kind, MapStructAttr, MapFieldAttr}};

pub(crate) fn validate(input: &Struct) -> Result<()> {
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::OwnedInto))?;
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::RefInto))?;
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::FromOwned))?;
    validate_struct_attrs(input.attrs.iter_for_kind(&Kind::FromRef))?;
    for x in &input.fields {
        validate_field_attrs(x.attrs.iter_for_kind(&Kind::OwnedInto), &x.attrs.child_attrs, input.attrs.iter_for_kind(&Kind::OwnedInto))?;
        validate_field_attrs(x.attrs.iter_for_kind(&Kind::RefInto), &x.attrs.child_attrs, input.attrs.iter_for_kind(&Kind::RefInto))?;
        validate_field_attrs(x.attrs.iter_for_kind(&Kind::FromOwned), &x.attrs.child_attrs, input.attrs.iter_for_kind(&Kind::FromOwned))?;
        validate_field_attrs(x.attrs.iter_for_kind(&Kind::FromRef), &x.attrs.child_attrs, input.attrs.iter_for_kind(&Kind::FromRef))?;
    }
    Ok(())
}

fn validate_struct_attrs<'a, I>(attrs: I) -> Result<()> 
    where I: Iterator<Item = &'a MapStructAttr>
{
    let mut unique_ident = HashSet::new();
    for attr in attrs {
        if !unique_ident.insert(attr.ty.path.to_string()) {
            return Err(syn::Error::new(attr.ty.span, "Ident here must be unique"))
        }
        let mut unique_child_ident = HashSet::new();
        for child in &attr.children {
            if !unique_child_ident.insert(child) {
                return Err(
                    syn::Error::new(child.get_child_path().span(), "Child field name here must be unique")
                )
            }
        }
    }
    Ok(())
}

fn validate_field_attrs<'a, IS, IF>(
    attrs: IF, 
    child_attr: &'a Vec<FieldChildAttr>, 
    struct_attrs: IS) -> Result<()> 
    where IS: Iterator<Item = &'a MapStructAttr>,
          IF: Iterator<Item = &'a MapFieldAttr>
{
    Ok(())
}
