use crate::attr::{self};
use crate::attr::{FieldAttrs, StructAttrs};
use syn::spanned::Spanned;
use syn::{DataStruct, DeriveInput, Fields, Generics, Ident, Index, Member, Result};

pub(crate) struct Struct<'a> {
    pub attrs: StructAttrs,
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field>,
    pub named_fields: bool,
}

pub(crate) struct Field {
    pub attrs: FieldAttrs,
    pub idx: usize,
    pub member: Member,
}

impl<'a> Struct<'a> {
    pub fn from_syn(node: &'a DeriveInput, data: &'a DataStruct) -> Result<Self> {
        let attrs = attr::get_struct_attrs(&node.attrs)?;
        let fields = Field::multiple_from_syn(&data.fields)?;
        Ok(Struct {
            attrs,
            ident: &node.ident,
            generics: &node.generics,
            fields,
            named_fields: matches!(&data.fields, Fields::Named(_)),
        })
    }
}

impl<'a> Field {
    fn multiple_from_syn(fields: &'a Fields) -> Result<Vec<Self>> {
        let mut attrs_to_repeat = None;

        fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let mut field = Field::from_syn(i, field)?;

                if field.attrs.stop_repeat {
                    attrs_to_repeat = None;
                }

                if field.attrs.repeat.is_some() {
                    if attrs_to_repeat.is_some() && !field.attrs.stop_repeat {
                        panic!("Previous #[repeat] instruction must be terminated with #[stop_repeat]")
                    }

                    attrs_to_repeat = Some(field.attrs.clone());
                } else if let Some(attrs_to_repeat) = &attrs_to_repeat {
                    field.attrs.merge(attrs_to_repeat.clone());
                }

                Ok(field)
            })
            .collect()
    }

    fn from_syn(i: usize, node: &'a syn::Field) -> Result<Self> {
        Ok(Field {
            attrs: attr::get_field_attrs(node)?,
            idx: i,
            member: node.ident.clone().map(Member::Named).unwrap_or_else(|| {
                Member::Unnamed(Index {
                    index: i as u32,
                    span: node.ty.span(),
                })
            }),
        })
    }
}
