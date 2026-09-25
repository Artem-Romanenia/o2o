use crate::model::*;

#[derive(Clone)]
pub(crate) struct AsAttr {
    pub container_ty: Option<TypePath>,
    pub member: Option<Member>,
    pub tokens: TokenStream,
}

impl Parse for AsAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        let (ident, tokens) = if peek_member(input) && input.peek2(Token![,]) {
            let ident = Some(input.parse()?);
            input.parse::<Token![,]>()?;
            let tokens = input.parse()?;
            (ident, tokens)
        } else { (None, input.parse()?) };

        Ok(AsAttr { container_ty, member: ident, tokens })
    }
}