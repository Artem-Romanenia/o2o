use quote::TokenStreamExt;
use crate::render::*;

#[derive(Debug)]
pub(crate) struct EnumInit {
    pub temp: TokenStream,
    pub ok_wrap: bool
}

impl Render for EnumInit {
    fn render(&self, _: &RenderContext, stream: &mut TokenStream) {
        let temp = &self.temp;
        
        match self.ok_wrap {
            true => stream.append_all(quote!(Ok(#temp))),
            false =>  stream.append_all(quote!(#temp))
        };
    }
}