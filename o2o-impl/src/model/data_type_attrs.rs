mod trait_attr;
mod ghost_attr;
mod where_attr;
mod child_parents_attr;

pub(crate) use trait_attr::*;
pub(crate) use ghost_attr::*;
pub(crate) use where_attr::*;
pub(crate) use child_parents_attr::*;

use crate::model::*;

#[derive(Default)]
pub(crate) struct DataTypeAttrs {
    pub attrs: Vec<TraitAttr>,
    pub ghosts_attrs: Vec<GhostsAttr>,
    pub where_attrs: Vec<WhereAttr>,
    pub child_parents_attrs: Vec<ChildParentsAttr>,

    pub error_instrs: Vec<DataTypeInstruction>,
}

impl<'a> DataTypeAttrs {
    pub(crate) fn iter_for_kind(&'a self, kind: &'a Kind, fallible: bool) -> impl Iterator<Item = &'a TraitAttr> {
        self.attrs.iter().filter(move |x| x.fallible == fallible && x.applicable_to[kind])
    }

    pub(crate) fn iter_for_kind_core(&'a self, kind: &'a Kind, fallible: bool) -> impl Iterator<Item = &'a TraitAttrCore> {
        self.iter_for_kind(kind, fallible).map(|x| &x.core)
    }

    pub(crate) fn ghosts_attr(&'a self, container_ty: &'a TypePath, kind: &'a Kind) -> Option<&'a StructGhostAttrCore> {
        self.ghosts_attrs.iter()
            .find(|x| x.applicable_to[kind] && x.attr.container_ty.is_some() && x.attr.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.ghosts_attrs.iter().find(|x| x.applicable_to[kind] && x.attr.container_ty.is_none())).map(|x| &x.attr)
    }

    pub(crate) fn where_attr(&'a self, container_ty: &TypePath) -> Option<&'a WhereAttr>{
        self.where_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.where_attrs.iter().find(|x| x.container_ty.is_none()))
    }

    pub(crate) fn child_parents_attr(&'a self, container_ty: &TypePath) -> Option<&'a ChildParentsAttr>{
        self.child_parents_attrs.iter()
            .find(|x| x.container_ty.is_some() && x.container_ty.as_ref().unwrap() == container_ty)
            .or_else(|| self.child_parents_attrs.iter().find(|x| x.container_ty.is_none()))
    }
}

pub(crate) enum DataTypeInstruction {
    Map(TraitAttr),
    Ghosts(GhostsAttr),
    Where(WhereAttr),
    ChildParents(ChildParentsAttr),
    AllowUnknown,

    Misplaced { instr: &'static str, span: Span, own: bool },
    Misnamed { instr: &'static str, span: Span, guess_name: &'static str, own: bool },
    UnrecognizedWithError { instr: String, span: Span },
    Unrecognized,
}