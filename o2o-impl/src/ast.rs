use crate::attr::{self};
use crate::attr::{StructAttrs, FieldAttrs};
use proc_macro2::Span;
use syn::{
    DataStruct, DeriveInput, Fields, Generics, Ident, Index, Member, Result
};

pub(crate) struct Struct<'a> {
    pub attrs: StructAttrs,
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field>,
    pub named: bool,
}

pub(crate) struct Field {
    pub attrs: FieldAttrs,
    pub idx: usize,
    pub member: Member,
}

impl<'a> Struct<'a> {
    pub fn from_syn(node: &'a DeriveInput, data: &'a DataStruct) -> Result<Self> {
        let attrs = attr::get_struct_attrs(&node.attrs)?;
        let fields = Field::multiple_from_syn(&data.fields, Span::call_site())?;
        Ok(Struct {
            attrs,
            ident: &node.ident,
            generics: &node.generics,
            fields,
            named: matches!(&data.fields, Fields::Named(_))
        })
    }
}

impl<'a> Field {
    fn multiple_from_syn(
        fields: &'a Fields,
        span: Span,
    ) -> Result<Vec<Self>> {
        fields
            .iter()
            .enumerate()
            .map(|(i, field)| Field::from_syn(i, field, span))
            .collect()
    }

    fn from_syn(
        i: usize,
        node: &'a syn::Field,
        span: Span,
    ) -> Result<Self> {
        Ok(Field {
            attrs: attr::get_field_attrs(&node.attrs)?,
            idx: i,
            member: node.ident.clone().map(Member::Named).unwrap_or_else(|| {
                Member::Unnamed(Index {
                    index: i as u32,
                    span,
                })
            }),
        })
    }
}
