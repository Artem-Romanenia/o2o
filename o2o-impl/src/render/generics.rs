use crate::render::*;

pub(crate) struct TheseGenerics<'a> {
    pub gens: &'a Generics
}

impl<'a> Render for TheseGenerics<'a> {
    fn render(&self, _: &RenderContext) -> TokenStream {
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
            quote!(<#(#filtered),*>)
        } else {
            quote!()
        }
    }
}

pub(crate) struct ThoseGenerics<'a> {
    pub gens: &'a Option<AngleBracketedGenericArguments>
}

impl<'a> Render for ThoseGenerics<'a>  {
    fn render(&self, _: &RenderContext) -> TokenStream {
        self.gens.to_token_stream()
    }
}

pub(crate) struct ImplGenerics {
    pub gens: Generics
}

impl Render for ImplGenerics {
    fn render(&self, _: &RenderContext) -> TokenStream {
        if !self.gens.params.is_empty() {
            let filtered = self.gens.params.iter().map(|param| match param {
                GenericParam::Type(syn::TypeParam { ident, colon_token, bounds, .. }) =>
                    quote!(#ident #colon_token #bounds),
                GenericParam::Lifetime(l) =>
                    quote!(#l),
                GenericParam::Const(syn::ConstParam { ident, colon_token, ty, .. }) =>
                    quote!(const #ident #colon_token #ty)
            });

            quote!(<#(#filtered),*>)
        } else {
            quote!()
        }
    }
}