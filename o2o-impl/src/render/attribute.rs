use crate::render::*;

#[derive(Debug)]
pub(crate) struct Attribute {
    pub attr: TokenStream,
    pub inner: bool,
}

impl Render for Attribute {
    fn render(&self, _: &RenderContext, stream: &mut TokenStream) {
        let attr = &self.attr;
        match self.inner {
            true => stream.extend(quote!(#![ #attr ])),
            false => stream.extend(quote!(#[ #attr ])),
        };
    }
}