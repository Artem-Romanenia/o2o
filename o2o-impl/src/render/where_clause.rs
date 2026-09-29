use crate::render::*;

pub(crate) struct WhereClause {
    pub where_clause: TokenStream
}

impl Render for WhereClause {
    fn render(&self, _: &RenderContext) -> TokenStream {
        let where_clause = &self.where_clause;
        quote!(where #where_clause)
    }
}