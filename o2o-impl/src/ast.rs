use crate::attr::{self, Kind, TypePath};
use crate::attr::{FieldAttrs, StructAttrs};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{Attribute, DataEnum, DataStruct, DeriveInput, Fields, Generics, Ident, Index, Member, Result};

pub(crate) struct Struct<'a> {
    pub attrs: StructAttrs,
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub fields: Vec<Field>,
    pub named_fields: bool,
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

pub(crate) struct Field {
    pub attrs: FieldAttrs,
    pub idx: usize,
    pub member: Member,
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

    fn from_syn(i: usize, field: &'a syn::Field) -> Result<Self> {
        Ok(Field {
            attrs: attr::get_field_attrs(DataTypeMember::Field(field))?,
            idx: i,
            member: field.ident.clone().map(Member::Named).unwrap_or_else(|| {
                Member::Unnamed(Index {
                    index: i as u32,
                    span: field.ty.span(),
                })
            }),
        })
    }
}

pub(crate) struct Enum<'a> {
    pub attrs: StructAttrs,
    pub ident: &'a Ident,
    pub generics: &'a Generics,
    pub variants: Vec<Variant>
}

impl<'a> Enum<'a> {
    pub fn from_syn(node: &'a DeriveInput, data: &'a DataEnum) -> Result<Self> {
        let attrs = attr::get_struct_attrs(&node.attrs)?;
        let variants = Variant::multiple_from_syn(&data.variants)?;
        Ok(Enum {
            attrs,
            ident: &node.ident,
            generics: &node.generics,
            variants
        })
    }
}

pub(crate) struct Variant {
    pub attrs: FieldAttrs,
    pub idx: usize,
    pub fields: Vec<Field>,
    pub named_fields: bool,
}

impl<'a> Variant {
    fn multiple_from_syn(variants: &'a Punctuated<syn::Variant, Comma>) -> Result<Vec<Self>> {
        let mut attrs_to_repeat = None;

        variants
            .iter()
            .enumerate()
            .map(|(i, variant)| {
                let mut field = Variant::from_syn(i, variant)?;

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

    fn from_syn(i: usize, variant: &'a syn::Variant) -> Result<Self> {
        let fields = Field::multiple_from_syn(&variant.fields)?;
        Ok(Variant {
            attrs: attr::get_field_attrs(DataTypeMember::Variant(&variant))?,
            idx: i,
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
            DataType::Struct(s) => &s.ident,
            DataType::Enum(e) => &e.ident
        }
    }

    pub fn get_attrs(&'a self) -> &'a StructAttrs {
        match self {
            DataType::Struct(s) => &s.attrs,
            DataType::Enum(e) => &e.attrs
        }
    }

    pub fn get_field(&'a self, ty: &TypePath) -> Option<&Field> {
        match self {
            DataType::Struct(s) => s.fields.iter().find(|f| 
                f.attrs.field_attr_core(&Kind::OwnedInto, ty)
                    .filter(|&g| g.wrapper).is_some()
            ),
            DataType::Enum(e) => None
        }
    }

    pub fn get_smt(&'a self) -> &Vec<Field> {
        match self {
            DataType::Struct(s) => &s.fields,
            DataType::Enum(e) => &e.attrs
        }
    }
}

pub(crate) enum DataTypeMember<'a> {
    Field(&'a syn::Field),
    Variant(&'a syn::Variant)
}

impl<'a> DataTypeMember<'a> {
    pub fn get_attrs(&'a self) -> &'a Vec<Attribute> {
        match self {
            DataTypeMember::Field(f) => &f.attrs,
            DataTypeMember::Variant(v) => &v.attrs
        }
    }
}