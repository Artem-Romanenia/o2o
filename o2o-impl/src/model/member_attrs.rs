mod member_attr;
mod child_attr;
mod parent_attr;
mod ghost_attr;
mod lit_attr;
mod pat_attr;
mod member_repeat_attr;
mod as_attr;

pub(crate) use member_attr::*;
pub(crate) use child_attr::*;
pub(crate) use parent_attr::*;
pub(crate) use ghost_attr::*;
pub(crate) use lit_attr::*;
pub(crate) use pat_attr::*;
pub(crate) use member_repeat_attr::*;
pub(crate) use as_attr::*;

use crate::model::*;

#[derive(Clone, Default)]
pub(crate) struct MemberAttrs {
    pub attrs: Vec<MemberAttr>,
    pub child_attrs: Vec<ChildAttr>,
    pub parent_attrs: Vec<ParentAttr>,
    pub ghost_attrs: Vec<GhostAttr>,
    pub ghosts_attrs: Vec<GhostsAttr>,
    pub lit_attrs: Vec<LitAttr>,
    pub pat_attrs: Vec<PatAttr>,
    pub repeat: Option<MemberRepeatAttr>,
    pub skip_repeat: bool,
    pub stop_repeat: bool,
    pub type_hint_attrs: Vec<VariantTypeHintAttr>,

    pub error_instrs: Vec<MemberInstruction>,
}

impl<'a> MemberAttrs {
    pub(crate) fn iter_for_kind(&'a self, kind: &'a Kind, fallible: bool) -> impl Iterator<Item = &'a MemberAttr> {
        self.attrs.iter().filter(move |x| x.fallible == fallible && x.applicable_to[kind])
    }

    pub(crate) fn iter_for_kind_core(&'a self, kind: &'a Kind, fallible: bool) -> impl Iterator<Item = &'a MemberAttrCore> {
        self.iter_for_kind(kind, fallible).map(|x| &x.attr)
    }

    pub(crate) fn applicable_field_attr(&'a self, kind: &'a Kind, fallible: bool, container_ty: &TypePath) -> Option<&'a MemberAttr> {
        self.field_attr(kind, fallible, container_ty)
            .or_else(|| if kind == &Kind::OwnedIntoExisting { self.field_attr(&Kind::OwnedInto, fallible, container_ty) } else { None })
            .or_else(|| if kind == &Kind::RefIntoExisting { self.field_attr(&Kind::RefInto, fallible, container_ty) } else { None })
    }

    pub(crate) fn child(&'a self, container_ty: &TypePath) -> Option<&'a ChildAttr>{
        self.child_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.child_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn ghost(&'a self, container_ty: &TypePath, kind: &'a Kind) -> Option<&'a FieldGhostAttrCore>{
        self.ghost_attrs.iter()
            .find(|x| x.applicable_to[kind] && x.attr.container_ty.is_some() && x.attr.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.ghost_attrs.iter().find(|x| x.applicable_to[kind] && x.attr.container_ty.is_none())).map(|x| &x.attr)
    }

    pub(crate) fn lit(&'a self, container_ty: &TypePath) -> Option<&'a LitAttr>{
        self.lit_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.lit_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn pat(&'a self, container_ty: &TypePath) -> Option<&'a PatAttr>{
        self.pat_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.pat_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn type_hint(&'a self, container_ty: &TypePath) -> Option<&'a VariantTypeHintAttr>{
        self.type_hint_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.type_hint_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn has_parent_attr(&'a self, container_ty: &TypePath) -> bool {
        self.parent_attrs.iter().any(|x| x.container_ty.is_none() || x.container_ty.as_ref().unwrap() == container_ty)
    }

    pub(crate) fn has_parameterless_parent_attr(&'a self, container_ty: &TypePath) -> bool {
        self.parent_attrs.iter().any(|x| x.child_fields.is_none() && (x.container_ty.is_none() || x.container_ty.as_ref().unwrap() == container_ty))
    }

    pub(crate) fn parameterized_parent_attr(&'a self, container_ty: &TypePath) -> Option<&'a ParentAttr> {
        self.parent_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty && x.child_fields.is_some())
            .or_else(|| self.parent_attrs.iter().find(|x| x.container_ty.is_none() && x.child_fields.is_some()))
    }

    pub(crate) fn field_attr(&'a self, kind: &'a Kind, fallible: bool, container_ty: &TypePath) -> Option<&'a MemberAttr> {
        self.iter_for_kind(kind, fallible)
            .find(|x| x.attr.container_ty.is_some() && x.attr.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.iter_for_kind(kind, fallible).find(|x| x.attr.container_ty.is_none()))
    }

    pub(crate) fn field_attr_core(&'a self, kind: &'a Kind, fallible: bool, container_ty: &TypePath) -> Option<&'a MemberAttrCore> {
        self.iter_for_kind_core(kind, fallible)
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.iter_for_kind_core(kind, fallible).find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn merge(&'a mut self, other: Self) {
        if self.skip_repeat {
            return;
        }

        if let Some(repeat) = other.repeat {
            if repeat.repeat_for[&MemberAttrType::Attr] {
                self.attrs.extend(other.attrs);
            }
            if repeat.repeat_for[&MemberAttrType::Child] {
                self.child_attrs.extend(other.child_attrs);
            }
            if repeat.repeat_for[&MemberAttrType::Parent] {
                self.parent_attrs.extend(other.parent_attrs);
            }
            if repeat.repeat_for[&MemberAttrType::Ghost] {
                self.ghost_attrs.extend(other.ghost_attrs);
            }
            if repeat.repeat_for[&MemberAttrType::TypeHint] {
                self.type_hint_attrs.extend(other.type_hint_attrs);
            }
        }
    }
}

#[derive(Clone)]
pub(crate) enum MemberInstruction {
    Map(MemberAttr),
    Ghost(GhostAttr),
    Ghosts(GhostsAttr),
    Child(ChildAttr),
    Parent(ParentAttr),
    As(AsAttr),
    Lit(LitAttr),
    Pat(PatAttr),
    VariantTypeHint(VariantTypeHintAttr),
    Repeat(MemberRepeatAttr),
    SkipRepeat,
    StopRepeat,

    Misplaced { instr: &'static str, span: Span, own: bool },
    Misnamed { instr: &'static str, span: Span, guess_name: &'static str, own: bool },
    UnrecognizedWithError { instr: String, span: Span },
    Unrecognized,
}

#[derive(Clone)]
pub(crate) struct VariantTypeHintAttr {
    pub container_ty: Option<TypePath>,
    pub type_hint: TypeHint,
}

impl Parse for VariantTypeHintAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let container_ty = try_parse_container_ident(input, false);
        let type_hint = input.parse()?;
        Ok(VariantTypeHintAttr { container_ty, type_hint })
    }
}