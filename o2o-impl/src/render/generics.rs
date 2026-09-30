use quote::TokenStreamExt;
use crate::render::*;

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub(crate) struct TheseGenerics<'a> {
    #[derivative(Debug(format_with="crate::debug_to_tokens"))]
    pub gens: &'a Generics
}

impl<'a> Render for TheseGenerics<'a> {
    fn render(&self, _: &RenderContext, stream: &mut TokenStream) {
        if !self.gens.params.is_empty() {
            let filtered = self.gens.params.iter().map(|param| match param {
                GenericParam::Type(syn::TypeParam { ident, .. }) =>
                    quote!(#ident),
                GenericParam::Lifetime(l) => {
                    let lifetime = &l.lifetime;
                    quote!(#lifetime)
                }
                GenericParam::Const(syn::ConstParam { ident, .. }) =>
                    quote!(#ident)
            });
            stream.append_all(quote!(<#(#filtered),*>));
        } else {
            stream.append_all(quote!());
        }
    }
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub(crate) struct ThoseGenerics<'a> {
    #[derivative(Debug(format_with="crate::debug_to_tokens"))]
    pub gens: &'a Option<AngleBracketedGenericArguments>
}

impl<'a> Render for ThoseGenerics<'a>  {
    fn render(&self, _: &RenderContext, stream: &mut TokenStream) {
        stream.append_all(self.gens);
    }
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub(crate) struct ImplGenerics {
    #[derivative(Debug(format_with="crate::debug_to_tokens"))]
    pub gens: Generics
}

impl Render for ImplGenerics {
    fn render(&self, _: &RenderContext, stream: &mut TokenStream) {
        if !self.gens.params.is_empty() {
            let filtered = self.gens.params.iter().map(|param| match param {
                GenericParam::Type(syn::TypeParam { ident, colon_token, bounds, .. }) =>
                    quote!(#ident #colon_token #bounds),
                GenericParam::Lifetime(l) =>
                    quote!(#l),
                GenericParam::Const(syn::ConstParam { ident, colon_token, ty, .. }) =>
                    quote!(const #ident #colon_token #ty)
            });

            stream.append_all(quote!(<#(#filtered),*>));
        }
    }
}