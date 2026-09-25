use std::ops::Not;

use crate::model::*;

#[derive(Clone)]
pub(crate) struct GhostAttr {
    pub attr: FieldGhostAttrCore,
    pub applicable_to: ApplicableTo,
}

#[derive(Clone)]
pub(crate) struct FieldGhostAttrCore {
    pub container_ty: Option<TypePath>,
    pub action: Option<InlineExpression>,
}

impl Parse for FieldGhostAttrCore {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FieldGhostAttrCore {
            container_ty: try_parse_container_ident(input, true),
            action: input.is_empty().not().then(|| input.parse()).transpose()?,
        })
    }
}