use crate::render::*;

#[derive(Debug)]
pub(crate) struct WhereClause {
    pub where_clause: TokenStream
}

impl Render for WhereClause {
    fn render(&self, _: &RenderContext, stream: &mut TokenStream) {
        let where_clause = &self.where_clause;
        stream.extend(quote!(where #where_clause));
    }
}