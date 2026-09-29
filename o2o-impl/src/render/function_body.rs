use crate::render::*;

#[derive(Debug)]
pub(crate) struct FunctionBody<'a> {
    pub pre_init: Option<PreInit<'a>>,
    pub main_code_block: &'a dyn Render,
    pub post_init_statements: Vec<PostInitStatement<'a>>
}

impl<'a> Render for FunctionBody<'a> {
    fn render(&self, ctx: &RenderContext) -> TokenStream {
        let pre_init = self.pre_init.as_ref().map(|x| x.render(ctx));
        let init = self.main_code_block.render(ctx);
        let post_init_statements: Vec<_> = self.post_init_statements.iter().map(|s| s.render(ctx)).collect();

        match (ctx.kind.is_into_existing(), ctx.fallible){
            (true, false) => quote!(#pre_init #init #(#post_init_statements)*),
            (true, true) => quote!(#pre_init #init #(#post_init_statements)* Ok(())),
            (false, fallible) => {
                let dst = ctx.dst_ty;
                let ret = if fallible { quote!(Ok(obj)) } else { quote!(obj) };

                if post_init_statements.is_empty() {
                    quote! {
                        #pre_init
                        #init
                    }
                } else {
                    quote! {
                        let mut obj: #dst = Default::default();
                        #init
                        #(#post_init_statements)*
                        #ret
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct PreInit<'a> {
    pub vars: Vec<(Expression<'a>, &'a Ident)>
}

impl<'a> Render for PreInit<'a> {
    fn render(&self, ctx: &RenderContext) -> TokenStream {
       let g = self.vars.iter().map(|(expr, ident)| {
            let a = ident;
            let b = expr.render(ctx);

            quote!(let #a = #b;)
        });
        TokenStream::from_iter(g)
    }
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub(crate) struct PostInitStatement<'a> {
    #[derivative(Debug(format_with="crate::debug_to_tokens"))]
    pub member: &'a Member
}

impl<'a> Render for PostInitStatement<'a> {
    fn render(&self, ctx: &RenderContext) -> TokenStream {
        let member = self.member;

        match (&ctx.kind, ctx.fallible) {
            (Kind::OwnedIntoExisting, false) => quote!(self.#member.into_existing(other);),
            (Kind::RefIntoExisting, false) => quote!((&(self.#member)).into_existing(other);),
            (Kind::OwnedInto, false) => quote!(self.#member.into_existing(&mut obj);),
            (Kind::RefInto, false) => quote!((&(self.#member)).into_existing(&mut obj);),
            (Kind::OwnedIntoExisting, true) => quote!(self.#member.try_into_existing(other)?;),
            (Kind::RefIntoExisting, true) => quote!((&(self.#member)).try_into_existing(other)?;),
            (Kind::OwnedInto, true) => quote!(self.#member.try_into_existing(&mut obj)?;),
            (Kind::RefInto, true) => quote!((&(self.#member)).try_into_existing(&mut obj)?;),
            _ => unreachable!("5"),
        }
    }
}