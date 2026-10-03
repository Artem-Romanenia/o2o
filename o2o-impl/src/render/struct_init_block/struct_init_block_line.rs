use proc_macro2::{Delimiter, Group, Literal, Punct, Spacing};
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


// ---- token emitters --------------------------------------------------------

fn punct(stream: &mut TokenStream, c: char) {
    Punct::new(c, Spacing::Alone).to_tokens(stream);
}

fn word(stream: &mut TokenStream, w: &str) {
    Ident::new(w, Span::call_site()).to_tokens(stream);
}

/// `w.`
fn dot(stream: &mut TokenStream, w: &str) {
    word(stream, w);
    punct(stream, '.');
}

fn index(stream: &mut TokenStream, i: usize) {
    Literal::usize_unsuffixed(i).to_tokens(stream);
}

fn f_ident(ident: usize) -> Ident {
    format_ident!("f{}", ident)
}

/// Only for APIs that insist on an owned `TokenStream`.
fn collect(f: impl FnOnce(&mut TokenStream)) -> TokenStream {
    let mut s = TokenStream::new();
    f(&mut s);
    s
}

/// `lhs = rhs;`
fn assign(
    stream: &mut TokenStream,
    lhs: impl FnOnce(&mut TokenStream),
    rhs: impl FnOnce(&mut TokenStream),
) {
    lhs(stream);
    punct(stream, '=');
    rhs(stream);
    punct(stream, ';');
}

/// `name: rhs,`  (or just `rhs,` when `name` is `None`)
fn init(
    stream: &mut TokenStream,
    name: Option<&dyn ToTokens>,
    rhs: impl FnOnce(&mut TokenStream),
) {
    if let Some(n) = name {
        n.to_tokens(stream);
        punct(stream, ':');
    }
    rhs(stream);
    punct(stream, ',');
}

/// `self.` / `value.` / nothing for variants
fn obj(ctx: &RenderContext, s: &mut TokenStream) {
    if ctx.impl_type.is_variant() {
        return;
    }
    match ctx.kind {
        Kind::FromOwned | Kind::FromRef => dot(s, "value"),
        _ => dot(s, "self"),
    }
}

/// `value.into()` / `value.try_into()?` / `(&value).into()` / `(&value).try_into()?`
fn parent_conv(ctx: &RenderContext, stream: &mut TokenStream) {
    if ctx.kind.is_ref() {
        word(stream, "value");
    } else {
        let inner = collect(|g| {
            punct(g, '&');
            word(g, "value");
        });
        Group::new(Delimiter::Parenthesis, inner).to_tokens(stream);
    }
    punct(stream, '.');
    word(stream, if ctx.fallible { "try_into" } else { "into" });
    Group::new(Delimiter::Parenthesis, TokenStream::new()).to_tokens(stream);
    if ctx.fallible {
        punct(stream, '?');
    }
}

// ---- path emitters (former closures) ---------------------------------------

impl<'a> StructInitBlockLine<'a> {
    /// `child_path.x` or `x`
    fn field_path(&self, x: &Member, s: &mut TokenStream) {
        if let Some(child_attr) = self.field.attrs.child(self.dst_ty) {
            child_attr.child_path.child_path.to_tokens(s);
            punct(s, '.');
        }
        x.to_tokens(s);
    }

    /// `#obj x sub_path.member` or `#obj x`
    fn child_field_path(&self, ctx: &RenderContext, x: &Member, stream: &mut TokenStream) {
        obj(ctx, stream);
        x.to_tokens(stream);
        if let Some(p) = self.parent_child {
            p.sub_path_tokens.to_tokens(stream);
            punct(stream, '.');
            p.this_member.to_tokens(stream);
        }
    }

    /// `expr.x` or `#obj field_path(x)`
    fn from_right_side(&self, ctx: &RenderContext, x: &Member, stream: &mut TokenStream) {
        match &self.child_parent_expr {
            Some(expr) => {
                ctx.with(expr).to_tokens(stream);
                punct(stream, '.');
                x.to_tokens(stream);
            }
            None => {
                obj(ctx, stream);
                self.field_path(x, stream);
            }
        }
    }

    fn has_parent(&self) -> bool {
        self.field.attrs.has_parent_attr(self.dst_ty)
    }
}

// ---- render ----------------------------------------------------------------

impl<'a> Render for StructInitBlockLine<'a> {
    fn render(&self, ctx: &RenderContext, stream: &mut TokenStream) {
        use Kind::*;
        use TypeHint::*;

        let f = self.field;
        let idx = self.idx;
        let is_variant = ctx.impl_type.is_variant();

        let member = self.parent_child.map_or(&f.member, |p| &p.this_member);
        let attr = self
            .parent_child
            .map(|p| ApplicableAttr::ParentChildField(p, ctx.kind))
            .or_else(|| ApplicableAttr::get(&f.attrs, &ctx.kind, ctx.fallible, self.dst_ty));

        // Still required by `attr.get_action_or` / `attr.get_stuff` signatures.
        let child_path_ts = |x: &Member| collect(|s| self.child_field_path(ctx, x, s));
        let right_side_ts = |x: &Member| collect(|s| self.from_right_side(ctx, x, s));
        let variant_or = |i: usize, fallback: &'a Member| -> Member {
            if is_variant { Named(f_ident(i)) } else { fallback.clone() }
        };

        match (member, attr, &ctx.kind, self.hint) {
            // ---------------- no attr, Into ----------------
            (Named(ident), None, OwnedInto | RefInto, Struct | Unspecified) => {
                if ctx.has_post_init {
                    assign(stream, |s| { dot(s, "obj"); ident.to_tokens(s) },
                           |s| { obj(ctx, s); ident.to_tokens(s) });
                } else {
                    init(stream, Some(ident), |s| { obj(ctx, s); ident.to_tokens(s) });
                }
            }
            (Named(ident), None, OwnedIntoExisting | RefIntoExisting, Struct | Unspecified) => {
                assign(stream, |s| { dot(s, "other"); self.field_path(&f.member, s) },
                       |s| { obj(ctx, s); ident.to_tokens(s) });
            }
            (Named(ident), None, OwnedInto | RefInto, Tuple) => {
                init(stream, None, |s| { obj(ctx, s); ident.to_tokens(s) });
            }
            (Named(ident), None, OwnedIntoExisting | RefIntoExisting, Tuple) => {
                assign(stream, |s| { dot(s, "other"); index(s, f.idx) },
                       |s| { obj(ctx, s); ident.to_tokens(s) });
            }

            // ---------------- no attr, From ----------------
            (Named(ident), None, FromOwned | FromRef, Struct | Unspecified | Unit) => {
                if self.has_parent() {
                    init(stream, Some(ident), |s| parent_conv(ctx, s));
                } else {
                    init(stream, Some(ident), |s| self.from_right_side(ctx, &f.member, s));
                }
            }
            (Named(ident), None, FromOwned | FromRef, Tuple) => {
                let src = if is_variant {
                    Named(f_ident(f.idx))
                } else {
                    Unnamed(Index { index: f.idx as u32, span: Span::call_site() })
                };
                init(stream, Some(ident), |s| { obj(ctx, s); self.field_path(&src, s) });
            }
            (Unnamed(i), None, OwnedInto | RefInto, Tuple | Unspecified) => {
                if ctx.has_post_init {
                    assign(stream, |s| { dot(s, "obj"); index(s, idx) },
                           |s| { obj(ctx, s); i.to_tokens(s) });
                } else if is_variant {
                    init(stream, None, |s| { obj(ctx, s); f_ident(i.index as usize).to_tokens(s) });
                } else {
                    init(stream, None, |s| { obj(ctx, s); i.to_tokens(s) });
                }
            }
            (Unnamed(i), None, OwnedIntoExisting | RefIntoExisting, Tuple | Unspecified) => {
                assign(stream, |s| { dot(s, "other"); index(s, f.idx) },
                       |s| { obj(ctx, s); i.to_tokens(s) });
            }
            (Unnamed(i), None, FromOwned | FromRef, Tuple | Unspecified | Unit) => {
                if self.has_parent() {
                    init(stream, None, |s| parent_conv(ctx, s));
                } else {
                    let src = variant_or(i.index as usize, &f.member);
                    init(stream, None, |s| { obj(ctx, s); self.field_path(&src, s) });
                }
            }
            (Unnamed(_), None, _, Struct) => {
                if self.has_parent() {
                    init(stream, None, |s| parent_conv(ctx, s));
                } else {
                    unreachable!("6")
                }
            }

            // ---------------- attr, Into ----------------
            (Named(_), Some(attr), OwnedInto | RefInto, Struct | Unspecified) => {
                let name = attr.get_field_name_or(&f.member);
                let path = child_path_ts(&f.member);
                let rhs = attr.get_action_or(Some(&path), ctx, || path.clone());
                if ctx.has_post_init {
                    assign(stream, |s| { dot(s, "obj"); name.to_tokens(s) }, |s| rhs.to_tokens(s));
                } else {
                    init(stream, Some(name), |s| rhs.to_tokens(s));
                }
            }
            (Named(_), Some(attr), OwnedIntoExisting | RefIntoExisting, Struct | Unspecified) => {
                let path = child_path_ts(&f.member);
                let rhs = attr.get_action_or(Some(&path), ctx, || path.clone());
                assign(stream, |s| { dot(s, "other"); self.field_path(attr.get_field_name_or(&f.member), s) },
                       |s| rhs.to_tokens(s));
            }
            (Named(_), Some(attr), OwnedInto | RefInto, Tuple) => {
                let path = child_path_ts(&f.member);
                let rhs = attr.get_action_or(Some(&path), ctx, || path.clone());
                init(stream, None, |s| rhs.to_tokens(s));
            }
            (Named(_), Some(attr), OwnedIntoExisting | RefIntoExisting, Tuple) => {
                let lhs = Unnamed(Index { index: idx as u32, span: Span::call_site() });
                let path = child_path_ts(&f.member);
                let rhs = attr.get_action_or(Some(&path), ctx, || path.clone());
                assign(stream, |s| { dot(s, "other"); self.field_path(&lhs, s) }, |s| rhs.to_tokens(s));
            }

            // ---------------- attr, From ----------------
            (Named(_), Some(attr), FromOwned | FromRef, Struct | Unspecified | Unit) => {
                let rhs = attr.get_stuff(None, right_side_ts, ctx, || &f.member);
                init(stream, Some(member), |s| rhs.to_tokens(s));
            }
            (Named(ident), Some(attr), FromOwned | FromRef, Tuple) => {
                let or = variant_or(f.idx, &f.member);
                let rhs = attr.get_stuff(None, right_side_ts, ctx, || &or);
                init(stream, Some(ident), |s| rhs.to_tokens(s));
            }
            (Unnamed(i), Some(attr), OwnedInto | RefInto, Tuple | Unspecified) => {
                let src = variant_or(i.index as usize, &f.member);
                let path = child_path_ts(&src);
                let rhs = attr.get_action_or(Some(&path), ctx, || path.clone());
                init(stream, None, |s| rhs.to_tokens(s));
            }
            (Unnamed(_), Some(attr), OwnedIntoExisting | RefIntoExisting, Tuple | Unspecified) => {
                let path = child_path_ts(&f.member);
                let rhs = attr.get_action_or(Some(&path), ctx, || path.clone());
                assign(stream, |s| { dot(s, "other"); self.field_path(attr.get_field_name_or(&f.member), s) },
                       |s| rhs.to_tokens(s));
            }
            (Unnamed(i), Some(attr), OwnedInto | RefInto, Struct) => {
                let name = attr.get_ident();
                let or = if is_variant {
                    f_ident(i.index as usize).into_token_stream()
                } else {
                    child_path_ts(&f.member)
                };
                let rhs = attr.get_action_or(Some(&or), ctx, || or.clone());
                if ctx.has_post_init {
                    assign(stream, |s| { dot(s, "obj"); name.to_tokens(s) }, |s| rhs.to_tokens(s));
                } else {
                    init(stream, Some(name), |s| rhs.to_tokens(s));
                }
            }
            (Unnamed(_), Some(attr), OwnedIntoExisting | RefIntoExisting, Struct) => {
                let path = child_path_ts(&f.member);
                let rhs = attr.get_action_or(Some(&path), ctx, || path.clone());
                assign(stream, |s| { dot(s, "other"); self.field_path(attr.get_ident(), s) },
                       |s| rhs.to_tokens(s));
            }
            (Unnamed(i), Some(attr), FromOwned | FromRef, _) => {
                let or = variant_or(i.index as usize, &f.member);
                let rhs = attr.get_stuff(None, right_side_ts, ctx, || &or);
                init(stream, None, |s| rhs.to_tokens(s));
            }

            (_, _, OwnedInto | RefInto | OwnedIntoExisting | RefIntoExisting, Unit) => {}
        }
    }
}
