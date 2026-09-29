use crate::render::*;

pub(crate) struct Function<'a> {
    pub attr: Option<Attribute>,
    pub inner_attr: Option<Attribute>,
    pub these_gens: &'a TheseGenerics<'a>,
    pub those_gens: &'a ThoseGenerics<'a>,
    pub body: FunctionBody<'a>,
    pub err_ty: Option<&'a TokenStream>,
    pub r: Option<&'a TokenStream>,
}

impl<'a> Render for Function<'a> {
    fn render(&self, ctx: &RenderContext) -> TokenStream {
        let attr = self.attr.as_ref().map(|x| x.render(ctx));
        let inner_attr = self.inner_attr.as_ref().map(|x| x.render(ctx));
        let these_gens = self.these_gens.render(ctx);
        let those_gens = self.those_gens.render(ctx);
        let src = ctx.src_ty;
        let dst = ctx.dst_ty;
        let r = &self.r;

        let err_ty = self.err_ty;

        let func_def = match (ctx.kind, ctx.fallible) {
            (Kind::FromOwned | Kind::FromRef, false) => quote!(from(value: #r #src #those_gens) -> #dst #these_gens),
            (Kind::FromOwned | Kind::FromRef, true) => quote!(try_from(value: #r #src #those_gens) -> ::core::result::Result<#dst #these_gens, #err_ty>),
            (Kind::OwnedInto | Kind::RefInto, false) => quote!(into(self) -> #dst #those_gens),
            (Kind::OwnedInto | Kind::RefInto, true) => quote!(try_into(self) -> ::core::result::Result<#dst #those_gens, #err_ty>),
            (Kind::OwnedIntoExisting | Kind::RefIntoExisting, false) => quote!(into_existing(self, other: &mut #dst #those_gens)),
            (Kind::OwnedIntoExisting | Kind::RefIntoExisting, true) => quote!(try_into_existing(self, other: &mut #dst #those_gens) -> ::core::result::Result<(), #err_ty>),
        };

        let body = self.body.render(ctx);

        quote! {
            #attr
            fn #func_def {
                #inner_attr
                #body
            }
        }
    }
}