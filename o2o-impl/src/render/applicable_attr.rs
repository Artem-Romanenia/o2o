use crate::model::*;
use crate::render::*;

pub(crate) enum ApplicableAttr<'a> {
    Field(&'a MemberAttrCore),
    Ghost(&'a FieldGhostAttrCore),
    ParentChildField(&'a ParentChildField, Kind),
}

impl<'a> ApplicableAttr<'a> {
    pub(crate) fn get(mem: &'a MemberAttrs, kind: &'a Kind, fallible: bool, container_ty: &TypePath) -> Option<ApplicableAttr<'a>> {
        mem.ghost(container_ty, kind)
            .map(ApplicableAttr::Ghost)
            .or_else(|| mem.field_attr_core(kind, fallible, container_ty)
                .or_else(|| if fallible { mem.field_attr_core(kind, false, container_ty) } else { None })
                .or_else(|| if kind == &Kind::OwnedIntoExisting { mem.field_attr_core(&Kind::OwnedInto, fallible, container_ty) } else { None })
                .or_else(|| if kind == &Kind::OwnedIntoExisting && fallible { mem.field_attr_core(&Kind::OwnedInto, false, container_ty) } else { None })
                .or_else(|| if kind == &Kind::RefIntoExisting { mem.field_attr_core(&Kind::RefInto, fallible, container_ty) } else { None })
                .or_else(|| if kind == &Kind::RefIntoExisting && fallible { mem.field_attr_core(&Kind::RefInto, false, container_ty) } else { None })
                .map(ApplicableAttr::Field))
    }

    pub(crate) fn get_ident(&'a self) -> &'a Member {
        match self {
            ApplicableAttr::Field(MemberAttrCore { member, .. }) => match member {
                Some(val) => val,
                None => unreachable!("8"),
            },
            ApplicableAttr::ParentChildField(p, kind) => {
                let attr = p.get_for_kind(kind);
                match attr.as_ref() {
                    Some(attr) => match attr.that_member.as_ref() {
                        Some(val) => val,
                        None => unreachable!("18"),
                    },
                    None => unreachable!("19")
                }
            }
            ApplicableAttr::Ghost(_) => unreachable!("9"),
        }
    }

    pub(crate) fn has_action(&self) -> bool {
        match self {
            ApplicableAttr::Field(f) => f.action.is_some(),
            ApplicableAttr::Ghost(g) => g.action.is_some(),
            ApplicableAttr::ParentChildField(p, kind) => p.get_for_kind(kind).is_some_and(|x| x.action.is_some()),
        }
    }

    pub(crate) fn get_field_name_or(&'a self, field: &'a Member) -> &'a Member {
        match self {
            ApplicableAttr::Field(MemberAttrCore { member, .. }) => match member {
                Some(val) => val,
                None => field,
            },
            ApplicableAttr::Ghost(_) => unreachable!("10"),
            ApplicableAttr::ParentChildField(p, kind) => {
                let attr = p.get_for_kind(kind);

                match attr.as_ref() {
                    Some(attr) =>  match attr.that_member.as_ref() {
                        Some(val) => val,
                        None => &p.this_member,
                    },
                    None => &p.this_member
                }
            }
        }
    }

    pub(crate) fn get_action_or<F: Fn() -> TokenStream>(&self, field_path: Option<&TokenStream>, ctx: &ImplContext, or: F) -> TokenStream {
        match self {
            ApplicableAttr::Field(MemberAttrCore { action, .. }) => match action {
                Some(val) => render_action(val, field_path, ctx),
                None => or(),
            },
            ApplicableAttr::ParentChildField(p, kind) => {
                let attr = p.get_for_kind(kind);

                match attr.as_ref() {
                    Some(attr) => match attr.action.as_ref() {
                        Some(val) => render_action(val, field_path, ctx),
                        None => or()
                    },
                    None => or()
                }
            }
            ApplicableAttr::Ghost(_) => unreachable!("11"),
        }
    }

    pub(crate) fn get_stuff<F1: Fn(&Member) -> TokenStream, F2: Fn() -> &'a Member>(&self, obj: Option<&TokenStream>, field_path: F1, ctx: &ImplContext, or: F2) -> TokenStream {
        let get_stuff = |member: &Option<Member>, expr: &Option<InlineExpression>| {
            match (member, expr) {
                (Some(ident), Some(expr)) => if let Unnamed(index) = ident {
                        if ctx.impl_type.is_variant() {
                            let ident = Named(format_ident!("f{}", index.index));
                            render_action(expr, Some(&field_path(&ident)), ctx)
                        } else {
                            render_action(expr, Some(&field_path(ident)), ctx)
                        }
                    } else {
                        render_action(expr, Some(&field_path(ident)), ctx)
                    },
                (Some(ident), None) => {
                    let field_path = field_path(ident);
                    quote!(#obj #field_path)
                }
                (None, Some(action)) => render_action(action, Some(&field_path(or())), ctx),
                _ => unreachable!("12"),
            }
        };
        match self {
            ApplicableAttr::Field(MemberAttrCore { member, action, .. }) => get_stuff(member, action),
            ApplicableAttr::ParentChildField(p, kind) => {
                let attr = p.get_for_kind(kind);

                if attr.is_some_and(|x|x.that_member.is_some()) {
                    let attr = attr.unwrap();
                    get_stuff(&attr.that_member, &attr.action)
                } else {
                    get_stuff(&Some(p.this_member.clone()), attr.map_or(&None, |x| &x.action))
                }
            },
            ApplicableAttr::Ghost(ghost_attr) => render_action(ghost_attr.action.as_ref().unwrap(), None, ctx),
        }
    }
}