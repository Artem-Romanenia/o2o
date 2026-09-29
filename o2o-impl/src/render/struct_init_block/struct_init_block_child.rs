use crate::render::*;

pub(crate) struct StructInitBlockChild<'a> {
    pub block: StructInitBlock<'a>,
    pub name: Member,
    pub ty: &'a syn::Path,
    pub action: Option<&'a ChildParentAction>,
    pub struct_kind: StructKind,
    pub parent_hint: TypeHint,
}

impl<'a> std::fmt::Debug for StructInitBlockChild<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StructInitBlockChild").field("block", &self.block).field("name", &self.name.to_token_stream().to_string()).field("ty", &self.ty.to_token_stream().to_string()).field("action", &self.action).field("struct_kind", &self.struct_kind).field("parent_hint", &self.parent_hint).finish()
    }
}

impl<'a> Render for StructInitBlockChild<'a> {
    fn render(&self, ctx: &RenderContext) -> TokenStream {
        let block = self.block.render(ctx);
        let child_name = &self.name;
        let ty = self.ty;
        let type_initialization = if let Some(action) = self.action {
            replace_tilde_or_at_in_expr(&action.action.expr, Some(&quote!(#ty #block)), None)
        } else {
            quote!(#ty #block)
        };

        match (self.struct_kind, self.parent_hint) {
            (StructKind::Tuple, TypeHint::Struct) |
            (StructKind::Struct, TypeHint::Struct | TypeHint::Unspecified) => quote!(#child_name: #type_initialization,),
            (StructKind::Struct, TypeHint::Tuple) |
            (StructKind::Tuple, TypeHint::Tuple | TypeHint::Unspecified) => quote!(#type_initialization,),
            (_, _) => unreachable!("15"),
        }
    }
}