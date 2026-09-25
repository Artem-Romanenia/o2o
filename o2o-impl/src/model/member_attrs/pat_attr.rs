use crate::model::*;

#[derive(Clone)]
pub(crate) struct PatAttr {
    pub container_ty: Option<TypePath>,
    pub tokens: TokenStream,
}

impl Parse for PatAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        Ok(PatAttr { container_ty, tokens: input.parse()? })
    }
}