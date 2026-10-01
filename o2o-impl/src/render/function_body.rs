use quote::TokenStreamExt;
use crate::render::*;

#[derive(Debug)]
pub(crate) struct FunctionBody<'a> {
    pub pre_init: Option<PreInit<'a>>,
    pub main_code_block: &'a dyn Render,
    pub post_init_statements: Vec<PostInitStatement<'a>>
}

impl<'a> Render for FunctionBody<'a> {
    fn render(&self, ctx: &RenderContext, stream: &mut TokenStream) {
        let pre_init = self.pre_init.as_ref().map(|x| ctx.with(x));
        let init = ctx.with(self.main_code_block);
        let post_init_statements: Vec<_> = self.post_init_statements.iter().map(|s| ctx.with(s)).collect();

        match (ctx.kind.is_into_existing(), ctx.fallible){
            (true, false) => stream.append_all(quote!(#pre_init #init #(#post_init_statements)*)),
            (true, true) => stream.append_all(quote!(#pre_init #init #(#post_init_statements)* Ok(()))),
            (false, fallible) => {
                let dst = ctx.dst_ty;
                let ret = if fallible { quote!(Ok(obj)) } else { quote!(obj) };

                if post_init_statements.is_empty() {
                    stream.append_all(quote! {
                        #pre_init
                        #init
                    });
                } else {
                    stream.append_all(quote! {
                        let mut obj: #dst = Default::default();
                        #init
                        #(#post_init_statements)*
                        #ret
                    });
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
    fn render(&self, ctx: &RenderContext, stream: &mut TokenStream) {
       self.vars.iter().for_each(|(expr, ident)| {
            let a = ident;
            let b = ctx.with(expr);

           stream.append_all(quote!(let #a = #b;))
        });
    }
}

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub(crate) struct PostInitStatement<'a> {
    #[derivative(Debug(format_with="crate::debug_to_tokens"))]
    pub member: &'a Member
}

impl<'a> Render for PostInitStatement<'a> {
    fn render(&self, ctx: &RenderContext, stream: &mut TokenStream) {
        let member = self.member;

        match (&ctx.kind, ctx.fallible) {
            (Kind::OwnedIntoExisting, false) => stream.append_all(quote!(self.#member.into_existing(other);)),
            (Kind::RefIntoExisting, false) => stream.append_all(quote!((&(self.#member)).into_existing(other);)),
            (Kind::OwnedInto, false) => stream.append_all(quote!(self.#member.into_existing(&mut obj);)),
            (Kind::RefInto, false) => stream.append_all(quote!((&(self.#member)).into_existing(&mut obj);)),
            (Kind::OwnedIntoExisting, true) => stream.append_all(quote!(self.#member.try_into_existing(other)?;)),
            (Kind::RefIntoExisting, true) => stream.append_all(quote!((&(self.#member)).try_into_existing(other)?;)),
            (Kind::OwnedInto, true) => stream.append_all(quote!(self.#member.try_into_existing(&mut obj)?;)),
            (Kind::RefInto, true) => stream.append_all(quote!((&(self.#member)).try_into_existing(&mut obj)?;)),
            _ => unreachable!("5"),
        }
    }
}