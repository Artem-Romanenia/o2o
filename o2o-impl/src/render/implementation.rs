use crate::render::*;

#[derive(Debug)]
pub(crate) struct Implementation<'a> {
    pub impl_attr: Option<Attribute>,
    pub err_ty: Option<&'a TokenStream>,
    pub impl_gens: ImplGenerics,
    pub these_gens: &'a TheseGenerics<'a>,
    pub those_gens: &'a ThoseGenerics<'a>,
    pub where_clause: Option<WhereClause>,
    pub r: Option<&'a TokenStream>,
    pub function: Function<'a>
}

impl<'a> Render for Implementation<'a> {
    fn render(&self, ctx: &RenderContext, stream: &mut TokenStream) {
        let impl_attr = self.impl_attr.as_ref().map(|x| ctx.with(x));
        let impl_gens = ctx.with(&self.impl_gens);
        let these_gens = ctx.with(self.these_gens);
        let those_gens = ctx.with(self.those_gens);
        let where_clause = self.where_clause.as_ref().map(|x| ctx.with(x));
        let src = ctx.src_ty;
        let dst = ctx.dst_ty;
        let r = &self.r;

        let trait_name = match (ctx.kind, ctx.fallible) {
            (Kind::FromOwned | Kind::FromRef, false) => quote!(::core::convert::From),
            (Kind::FromOwned | Kind::FromRef, true) => quote!(::core::convert::TryFrom),
            (Kind::OwnedInto | Kind::RefInto, false) => quote!(::core::convert::Into),
            (Kind::OwnedInto | Kind::RefInto, true) => quote!(::core::convert::TryInto),
            (Kind::OwnedIntoExisting | Kind::RefIntoExisting, false) => quote!(o2o::traits::IntoExisting),
            (Kind::OwnedIntoExisting | Kind::RefIntoExisting, true) => quote!(o2o::traits::TryIntoExisting),
        };

        let trait_def = match (ctx.kind.is_ref(), ctx.kind.is_from()) {
            (false, false) => quote!(#trait_name <#dst #those_gens> for #src #these_gens),
            (true, false) => quote!(#trait_name <#dst #those_gens> for #r #src #these_gens),
            (false, true) => quote!(#trait_name <#src #those_gens> for #dst #these_gens),
            (true, true) => quote!(#trait_name <#r #src #those_gens> for #dst #these_gens),
        };

        let err_ty = &self.err_ty.as_ref().map(|x| quote!(type Error = #x;));

        let function = ctx.with(&self.function);
        stream.extend(quote! {
            #impl_attr
            impl #impl_gens #trait_def #where_clause {
                #err_ty
                #function
            }
        });
    }
}