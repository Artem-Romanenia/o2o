use std::ops::Not;

use crate::model::*;

#[derive(Clone)]
pub(crate) struct MemberAttr {
    pub attr: MemberAttrCore,
    pub fallible: bool,
    pub original_instr: String,
    pub applicable_to: ApplicableTo,
}

#[derive(Clone)]
pub(crate) struct MemberAttrCore {
    pub container_ty: Option<TypePath>,
    pub member: Option<Member>,
    pub action: Option<InlineExpression>,
}

impl Parse for MemberAttrCore {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(MemberAttrCore {
            container_ty: try_parse_container_ident(input, false),
            member: try_parse_optional_ident(input),
            action: input.is_empty().not().then(|| input.parse()).transpose()?,
        })
    }
}