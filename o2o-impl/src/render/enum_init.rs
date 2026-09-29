use crate::render::*;

#[derive(Debug)]
pub(crate) struct EnumInit {
    pub temp: TokenStream,
    pub ok_wrap: bool
}

impl Render for EnumInit {
    fn render(&self, _: &RenderContext) -> TokenStream {
        let temp = &self.temp;
        
        match self.ok_wrap {
            true => quote!(Ok(#temp)),
            false =>  quote!(#temp)
        }
    }
}