use crate::model::*;

#[derive(Clone)]
pub(crate) struct LitAttr {
    pub container_ty: Option<TypePath>,
    pub tokens: TokenStream,
}

impl Parse for LitAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        Ok(LitAttr { container_ty, tokens: input.parse()? })
    }
}