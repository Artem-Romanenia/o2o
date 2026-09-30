use crate::render::*;
use quote::TokenStreamExt;

#[derive(Debug)]
pub(crate) struct Attribute {
    pub attr: TokenStream,
    pub inner: bool,
}

impl Render for Attribute {
    fn render(&self, _: &RenderContext, stream: &mut TokenStream) {
        let attr = &self.attr;
        match self.inner {
            true => stream.append_all(quote!(#![ #attr ])),
            false => stream.append_all(quote!(#[ #attr ])),
        };
    }
}