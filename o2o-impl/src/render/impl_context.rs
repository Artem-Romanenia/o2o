use crate::model::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ImplType {
    Struct,
    Enum,
    Variant,
}

impl ImplType {
    pub(crate) fn is_variant(self) -> bool {
        self == ImplType::Variant
    }
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