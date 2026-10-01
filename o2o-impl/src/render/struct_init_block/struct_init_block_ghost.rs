use quote::TokenStreamExt;
use crate::render::*;

#[derive(Debug)]
pub(crate) struct StructInitBlockGhost<'a> {
    pub child_path: Option<&'a ChildPath>,
    pub ghost_ident: &'a GhostIdent,
    pub expr: Expression<'a>
}

impl<'a> Render for StructInitBlockGhost<'a> {
    fn render(&self, ctx: &RenderContext, stream: &mut TokenStream) {
        let child_path = match &self.child_path {
            Some(ghost_data) => {
                let child_path = ghost_data.child_path.to_token_stream();
                quote!(#child_path.)
            },
            None => TokenStream::new(),
        };
        
        let right_side = ctx.with(&self.expr);
        let ghost_ident = self.ghost_ident.get_ident();

        match (ghost_ident, &ctx.kind) {
            (Named(ident), Kind::OwnedInto | Kind::RefInto) => stream.append_all(quote!(#ident: #right_side,)),
            (Unnamed(_), Kind::OwnedInto | Kind::RefInto) => stream.append_all(quote!(#right_side,)),
            (Named(ident), Kind::OwnedIntoExisting | Kind::RefIntoExisting) => stream.append_all(quote!(other.#child_path #ident = #right_side;)),
            (Unnamed(index), Kind::OwnedIntoExisting | Kind::RefIntoExisting) => stream.append_all(quote!(other.#child_path #index = #right_side;)),
            (_, _) => unreachable!("7"),
        };
    }
}