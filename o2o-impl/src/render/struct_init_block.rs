mod struct_init_block_line;
mod struct_init_block_child;
mod struct_init_block_ghost;

pub(crate) use struct_init_block_line::*;
pub(crate) use struct_init_block_child::*;
pub(crate) use struct_init_block_ghost::*;

use crate::render::*;

#[derive(Debug)]
pub(crate) struct StructInitBlock<'a> {
    pub fragments: Vec<StructInitBlockFragment<'a>>,
    pub type_hint: TypeHint,
    pub struct_kind: StructKind,
    pub dst: Option<&'a TokenStream>,
    pub ok_wrap: bool,
}

impl<'a> Render for StructInitBlock<'a> {
    fn render(&self, ctx: &RenderContext) -> TokenStream {
        let fragments = self.fragments.iter().map(|f| f.render(ctx));

        let block = if ctx.has_post_init || ctx.kind.is_into_existing() {
            quote!(#(#fragments)*)
        } else {
            match (&ctx.kind.is_from(), self.type_hint, self.struct_kind) {
                (true, _, StructKind::Struct) => quote!({#(#fragments)*}),
                (true, _, StructKind::Tuple) => quote!((#(#fragments)*)),
                (true, _, StructKind::Unit) => quote!(),
                (false, TypeHint::Struct, _) |
                (false, TypeHint::Unspecified, StructKind::Struct) => quote!({#(#fragments)*}),
                (false, TypeHint::Tuple, _) |
                (false, TypeHint::Unspecified, StructKind::Tuple) => quote!((#(#fragments)*)),
                (false, TypeHint::Unit, _) => quote!(),
                _ => panic!(),
            }
        };

        let block = if let Some(dst) = self.dst {
            quote!(#dst #block)
        } else {
            block
        };

        if self.ok_wrap {
            quote!(Ok(#block))
        } else {
            block
        }
    }
}

#[derive(Debug)]
pub(crate) enum StructInitBlockFragment<'a> {
    Line(StructInitBlockLine<'a>),
    Child(StructInitBlockChild<'a>),
    Ghost(StructInitBlockGhost<'a>),
    Update(Expression<'a>),
}

impl Render for StructInitBlockFragment<'_> {
    fn render(&self, ctx: &RenderContext) -> TokenStream {
        match self {
            StructInitBlockFragment::Line(line) => line.render(ctx),
            StructInitBlockFragment::Child(child) => child.render(ctx),
            StructInitBlockFragment::Ghost(ghost) => ghost.render(ctx),
            StructInitBlockFragment::Update(expr) => {
                let expr = expr.render(ctx);
                quote!(..#expr)
            },
        }
    }
}