use crate::render::*;

#[derive(Debug)]
pub(crate) struct Attribute {
    pub attr: TokenStream,
    pub inner: bool,
}

impl Render for Attribute {
    fn render(&self, _: &RenderContext) -> TokenStream {
        let attr = &self.attr;
        match self.inner {
            true => quote!(#![ #attr ]),
            false => quote!(#[ #attr ])
        }
    }
}