use crate::model::*;

pub(crate) struct WhereAttr {
    pub container_ty: Option<TypePath>,
    pub where_clause: Punctuated<WherePredicate, Token![,]>,
}

impl Parse for WhereAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(WhereAttr {
            container_ty: try_parse_container_ident(input, false),
            where_clause: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}