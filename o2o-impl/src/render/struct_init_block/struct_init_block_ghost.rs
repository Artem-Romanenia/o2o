use crate::render::*;

pub(crate) struct StructInitBlockGhost<'a> {
    pub child_path: Option<&'a ChildPath>,
    pub ghost_ident: &'a GhostIdent,
    pub expr: Expression<'a>
}

impl Render for StructInitBlockGhost<'_> {
    fn render(&self, ctx: &RenderContext) -> TokenStream {
        let child_path = match &self.child_path {
            Some(ghost_data) => {
                let child_path = ghost_data.child_path.to_token_stream();
                quote!(#child_path.)
            },
            None => TokenStream::new(),
        };
        
        let right_side = self.expr.render(ctx);
        let ghost_ident = self.ghost_ident.get_ident();

        match (ghost_ident, &ctx.kind) {
            (Named(ident), Kind::OwnedInto | Kind::RefInto) => quote!(#ident: #right_side,),
            (Unnamed(_), Kind::OwnedInto | Kind::RefInto) => quote!(#right_side,),
            (Named(ident), Kind::OwnedIntoExisting | Kind::RefIntoExisting) => quote!(other.#child_path #ident = #right_side;),
            (Unnamed(index), Kind::OwnedIntoExisting | Kind::RefIntoExisting) => quote!(other.#child_path #index = #right_side;),
            (_, _) => unreachable!("7"),
        }
    }
}