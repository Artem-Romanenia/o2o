use crate::attr::{self};
use crate::attr::{MemberAttrs, DataTypeAttrs};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{Attribute, DataEnum, DataStruct, DeriveInput, Fields, Generics, Ident, Index, Member, Result};

pub(crate) struct Struct<'a> {
    pub attrs: DataTypeAttrs,
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field>,
    pub named_fields: bool,
}

impl<'a> Struct<'a> {
    pub fn from_syn(node: &'a DeriveInput, data: &'a DataStruct) -> Result<Self> {
        let (attrs, bark) = attr::get_data_type_attrs(&node.attrs)?;
        let fields = Field::multiple_from_syn(&data.fields, bark)?;
        Ok(Struct {
            attrs,
            ident: &node.ident,
            generics: &node.generics,
            fields,
            named_fields: matches!(&data.fields, Fields::Named(_)),
        })
    }
}

#[derive(Clone)]
pub(crate) struct Field {
    pub attrs: MemberAttrs,
    pub idx: usize,
    pub member: Member,
}

impl<'a> Field {
    fn multiple_from_syn(fields: &'a Fields, bark: bool) -> Result<Vec<Self>> {
        let mut attrs_to_repeat = None;

        fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let mut field = Field::from_syn(i, field, bark)?;

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

    fn from_syn(i: usize, node: &'a syn::Field, bark: bool) -> Result<Self> {
        Ok(Field {
            attrs: attr::get_member_attrs(SynDataTypeMember::Field(node), bark)?,
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

pub(crate) struct Enum<'a> {
    pub attrs: DataTypeAttrs,
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub variants: Vec<Variant>
}

impl<'a> Enum<'a> {
    pub fn from_syn(node: &'a DeriveInput, data: &'a DataEnum) -> Result<Self> {
        let (attrs, bark) = attr::get_data_type_attrs(&node.attrs)?;
        let variants = Variant::multiple_from_syn(&data.variants, bark)?;
        Ok(Enum {
            attrs,
            ident: &node.ident,
            generics: &node.generics,
            variants
        })
    }
}

pub(crate) struct Variant {
    pub attrs: MemberAttrs,
    pub ident: Ident,
    _idx: usize,
    pub fields: Vec<Field>,
    pub named_fields: bool,
}

impl<'a> Variant {
    fn multiple_from_syn(variants: &'a Punctuated<syn::Variant, Comma>, bark: bool) -> Result<Vec<Self>> {
        let mut attrs_to_repeat = None;

        variants
            .iter()
            .enumerate()
            .map(|(i, variant)| {
                let mut field = Variant::from_syn(i, variant, bark)?;

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

    fn from_syn(i: usize, variant: &'a syn::Variant, bark: bool) -> Result<Self> {
        let fields = Field::multiple_from_syn(&variant.fields, bark)?;
        Ok(Variant {
            attrs: attr::get_member_attrs(SynDataTypeMember::Variant(variant), bark)?,
            ident: variant.ident.clone(),
            _idx: i,
            fields,
            named_fields: matches!(&variant.fields, Fields::Named(_)),
        })
    }
}

pub(crate) enum DataType<'a> {
    Struct(&'a Struct<'a>),
    Enum(&'a Enum<'a>)
}

impl<'a> DataType<'a> {
    pub fn get_ident(&'a self) -> &Ident {
        match self {
            DataType::Struct(s) => s.ident,
            DataType::Enum(e) => e.ident
        }
    }

    pub fn get_attrs(&'a self) -> &'a DataTypeAttrs {
        match self {
            DataType::Struct(s) => &s.attrs,
            DataType::Enum(e) => &e.attrs
        }
    }

    pub fn get_members(&'a self) -> Vec<DataTypeMember> {
        match self {
            DataType::Struct(s) => s.fields.iter().map(DataTypeMember::Field).collect(),
            DataType::Enum(e) => e.variants.iter().map(DataTypeMember::Variant).collect()
        }
    }

    pub fn get_generics(&'a self) -> &'a Generics {
        match self {
            DataType::Struct(s) => s.generics,
            DataType::Enum(e) => e.generics
        }
    }
}

pub(crate) enum SynDataTypeMember<'a> {
    Field(&'a syn::Field),
    Variant(&'a syn::Variant)
}

impl<'a> SynDataTypeMember<'a> {
    pub fn get_attrs(&'a self) -> &'a Vec<Attribute> {
        match self {
            SynDataTypeMember::Field(f) => &f.attrs,
            SynDataTypeMember::Variant(v) => &v.attrs
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum DataTypeMember<'a> {
    Field(&'a Field),
    Variant(&'a Variant)
}

impl<'a> DataTypeMember<'a> {
    pub fn get_attrs(&'a self) -> &'a MemberAttrs {
        match self {
            DataTypeMember::Field(f) => &f.attrs,
            DataTypeMember::Variant(v) => &v.attrs
        }
    }
}