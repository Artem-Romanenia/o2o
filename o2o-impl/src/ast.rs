#[cfg(feature = "syn2")]
use syn2 as syn;

use syn::Attribute;

pub(crate) enum SynDataTypeMember<'a> {
    Field(&'a syn::Field),
    Variant(&'a syn::Variant),
}

impl<'a> SynDataTypeMember<'a> {
    pub fn get_attrs(&'a self) -> &'a Vec<Attribute> {
        match self {
            SynDataTypeMember::Field(f) => &f.attrs,
            SynDataTypeMember::Variant(v) => &v.attrs,
        }
    }
}
