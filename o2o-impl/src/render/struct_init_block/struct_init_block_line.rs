use crate::render::*;

#[derive(Debug)]
pub(crate) struct StructInitBlockLine<'a> {
    pub dst_ty: &'a TypePath,
    pub field: &'a Field,
    pub hint: TypeHint,
    pub idx: usize,
    pub parent_child: Option<&'a ParentChildField>,
    pub child_parent_expr: Option<Expression<'a>>
}

impl<'a> Render for StructInitBlockLine<'a> {
    fn render(&self, ctx: &RenderContext) -> TokenStream {
        let f = self.field;
        let hint = self.hint;
        let idx = self.idx;
        let parent_child = self.parent_child;

        let obj = if ctx.impl_type.is_variant() { TokenStream::new() } else {
            match ctx.kind {
                Kind::OwnedInto => quote!(self.),
                Kind::RefInto => quote!(self.),
                Kind::FromOwned => quote!(value.),
                Kind::FromRef => quote!(value.),
                Kind::OwnedIntoExisting => quote!(self.),
                Kind::RefIntoExisting => quote!(self.),
            }
        };

        let member = parent_child.map(|p| &p.this_member)
            .unwrap_or(&f.member);
        let attr = parent_child.map(|p| ApplicableAttr::ParentChildField(p, ctx.kind))
            .or_else(|| ApplicableAttr::get(&f.attrs, &ctx.kind, ctx.fallible, &self.dst_ty));
        let get_field_path = |x: &Member| match f.attrs.child(&self.dst_ty) {
            Some(child_attr) => {
                let ch = child_attr.child_path.child_path.to_token_stream(); 
                quote!(#ch.#x)
            }
            None => x.to_token_stream(),
        };
        let get_child_field_path = |x: &Member| match parent_child {
            Some(p) => {
                let sub_path = &p.sub_path_tokens;
                quote!(#obj #x #sub_path.#member)
            },
            None => quote!(#obj #x)
        };
        let get_from_right_side = |x: &Member| match &self.child_parent_expr {
            Some(expr) => {
                let expr = expr.render(ctx);
                quote!(#expr.#x)
            },
            None => {
                let path = get_field_path(x);
                quote!(#obj #path)
            }
        };

        match (member, attr, &ctx.kind, hint) {
            (Named(ident), None, Kind::OwnedInto | Kind::RefInto, TypeHint::Struct | TypeHint::Unspecified) =>
                if ctx.has_post_init { quote!(obj.#ident = #obj #ident;) } else { quote!(#ident: #obj #ident,) },
            (Named(ident), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Struct | TypeHint::Unspecified) => {
                let field_path = get_field_path(&f.member);
                quote!(other.#field_path = #obj #ident;)
            },
            (Named(ident), None, Kind::OwnedInto | Kind::RefInto, TypeHint::Tuple) => 
                quote!(#obj #ident,),
            (Named(ident), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Tuple) => {
                let index = Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
                quote!(other.#index = #obj #ident;)
            },
            (Named(ident), None, Kind::FromOwned | Kind::FromRef, TypeHint::Struct | TypeHint::Unspecified | TypeHint::Unit) =>
                if f.attrs.has_parent_attr(&self.dst_ty) {
                    match (ctx.kind.is_ref(), ctx.fallible) {
                        (true, true) => quote!(#ident: value.try_into()?,),
                        (true, false) => quote!(#ident: value.into(),),
                        (false, true) => quote!(#ident: (&value).try_into()?,),
                        (false, false) => quote!(#ident: (&value).into(),),
                    }
                } else {
                    let right_side = get_from_right_side(&f.member);
                    quote!(#ident: #right_side,)
                },
            (Named(ident), None, Kind::FromOwned | Kind::FromRef, TypeHint::Tuple) => {
                let index = Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
                let field_path = if ctx.impl_type.is_variant() { get_field_path(&Named(format_ident!("f{}", index))) } else { get_field_path(&index) };
                quote!(#ident: #obj #field_path,)
            },
            (Unnamed(index), None, Kind::OwnedInto | Kind::RefInto, TypeHint::Tuple | TypeHint::Unspecified) =>
                if ctx.has_post_init {
                    let index2 = Unnamed(Index { index: idx as u32, span: Span::call_site() });
                    quote!(obj.#index2 = #obj #index;)
                } else {
                    let index = if ctx.impl_type.is_variant() { format_ident!("f{}", index.index).to_token_stream() } else { index.to_token_stream() };
                    quote!(#obj #index,)
                },
            (Unnamed(index), None, Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Tuple | TypeHint::Unspecified) => {
                let index2 = Unnamed(Index { index: f.idx as u32, span: Span::call_site() });
                quote!(other.#index2 = #obj #index;)
            },
            (Unnamed(index), None, Kind::FromOwned | Kind::FromRef, TypeHint::Tuple | TypeHint::Unspecified | TypeHint::Unit) =>
                if f.attrs.has_parent_attr(&self.dst_ty) {
                    match (ctx.kind.is_ref(), ctx.fallible) {
                        (true, true) => quote!(value.try_into()?,),
                        (true, false) => quote!(value.into(),),
                        (false, true) => quote!((&value).try_into()?,),
                        (false, false) => quote!((&value).into(),),
                    }
                } else {
                    let field_path = if ctx.impl_type.is_variant() { get_field_path(&Named(format_ident!("f{}", index.index))) } else { get_field_path(&f.member) };
                    quote!(#obj #field_path,)
                },
            (Unnamed(_), None, _, TypeHint::Struct) =>
                if f.attrs.has_parent_attr(&self.dst_ty) {
                    match (ctx.kind.is_ref(), ctx.fallible) {
                        (true, true) => quote!(value.try_into()?,),
                        (true, false) => quote!(value.into(),),
                        (false, true) => quote!((&value).try_into()?,),
                        (false, false) => quote!((&value).into(),),
                    }
                } else {
                    unreachable!("6")
                },
            (Named(_), Some(attr), Kind::OwnedInto | Kind::RefInto, TypeHint::Struct | TypeHint::Unspecified) => {
                let field_name = attr.get_field_name_or(&f.member);
                let field_path = get_child_field_path(&f.member);
                let right_side = attr.get_action_or(Some(&field_path), ctx, || quote!(#field_path));
                if ctx.has_post_init { quote!(obj.#field_name = #right_side;) } else { quote!(#field_name: #right_side,) }
            },
            (Named(_), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Struct | TypeHint::Unspecified) => {
                let left_field_path = get_field_path(attr.get_field_name_or(&f.member));
                let right_field_path = get_child_field_path(&f.member);
                let right_side = attr.get_action_or(Some(&right_field_path), ctx, || quote!(#right_field_path));
                quote!(other.#left_field_path = #right_side;)
            },
            (Named(_), Some(attr), Kind::OwnedInto | Kind::RefInto, TypeHint::Tuple) => {
                let right_field_path = get_child_field_path(&f.member);
                let right_side = attr.get_action_or(Some(&right_field_path), ctx, || quote!(#right_field_path));
                quote!(#right_side,)
            },
            (Named(_), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Tuple) => {
                let left_field_path = get_field_path(&Unnamed(Index { index: idx as u32, span: Span::call_site() }));
                let right_field_path = get_child_field_path(&f.member);
                let right_side = attr.get_action_or(Some(&right_field_path), ctx, || quote!(#right_field_path));
                quote!(other.#left_field_path = #right_side;)
            },
            (Named(_), Some(attr), Kind::FromOwned | Kind::FromRef, TypeHint::Struct | TypeHint::Unspecified | TypeHint::Unit) => {
                let right_side = attr.get_stuff(None, get_from_right_side, ctx, || &f.member);
                let idnt = parent_child.map_or(&f.member, |g| &g.this_member);
                quote!(#idnt: #right_side,)
            },
            (Named(ident), Some(attr), Kind::FromOwned | Kind::FromRef, TypeHint::Tuple) => {
                let or = Named(format_ident!("f{}", f.idx));
                let right_side = attr.get_stuff(None, get_from_right_side, ctx, || if ctx.impl_type.is_variant() { &or } else { &f.member });
                quote!(#ident: #right_side,)
            },
            (Unnamed(index), Some(attr), Kind::OwnedInto | Kind::RefInto, TypeHint::Tuple | TypeHint::Unspecified) => {
                let index = if ctx.impl_type.is_variant() { &Member::Named(format_ident!("f{}", index.index)) } else { &f.member };
                let field_path = get_child_field_path(index);
                let right_side = attr.get_action_or(Some(&field_path), ctx, || quote!(#field_path));
                quote!(#right_side,)
            },
            (Unnamed(_), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Tuple | TypeHint::Unspecified) => {
                let left_field_path = get_field_path(attr.get_field_name_or(&f.member));
                let right_field_path = get_child_field_path(&f.member);
                let right_side = attr.get_action_or(Some(&right_field_path), ctx, || quote!(#right_field_path));
                quote!(other.#left_field_path = #right_side;)
            },
            (Unnamed(index), Some(attr), Kind::OwnedInto | Kind::RefInto, TypeHint::Struct) => {
                let field_name = attr.get_ident();
                let field_path = get_child_field_path(&f.member);
                let or = if ctx.impl_type.is_variant() { format_ident!("f{}", index.index).to_token_stream() } else { field_path };
                let right_side = attr.get_action_or(Some(&or), ctx, || quote!(#or));
                if ctx.has_post_init {
                    quote!(obj.#field_name = #right_side;)
                } else {
                    quote!(#field_name: #right_side,)
                }
            },
            (Unnamed(_), Some(attr), Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Struct) => {
                let left_field_path = get_field_path(attr.get_ident());
                let right_field_path = get_child_field_path(&f.member);
                let right_side = attr.get_action_or(Some(&right_field_path), ctx, || quote!(#right_field_path));
                quote!(other.#left_field_path = #right_side;)
            },
            (Unnamed(index), Some(attr), Kind::FromOwned | Kind::FromRef, _) => {
                let or = Named(format_ident!("f{}", index.index));
                let right_side = attr.get_stuff(None, get_from_right_side, ctx, || if ctx.impl_type.is_variant() { &or } else { &f.member });
                quote!(#right_side,)
            },
            (_, _, Kind::OwnedInto | Kind::RefInto | Kind::OwnedIntoExisting | Kind::RefIntoExisting, TypeHint::Unit) => TokenStream::new(),
        }
    }
}