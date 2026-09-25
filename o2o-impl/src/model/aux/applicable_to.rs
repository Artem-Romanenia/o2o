use std::ops::Index;

use crate::model::Kind;

pub(crate) type ApplicableTo = [bool; 6];

impl Index<&Kind> for ApplicableTo {
    type Output = bool;

    fn index(&self, index: &Kind) -> &Self::Output {
        match index {
            Kind::OwnedInto => &self[0],
            Kind::RefInto => &self[1],
            Kind::FromOwned => &self[2],
            Kind::FromRef => &self[3],
            Kind::OwnedIntoExisting => &self[4],
            Kind::RefIntoExisting => &self[5],
        }
    }
}

pub(crate) fn appl_to(kind_str: &str) -> ApplicableTo {
    [
        appl_owned_into(kind_str),
        appl_ref_into(kind_str),
        appl_from_owned(kind_str),
        appl_from_ref(kind_str),
        appl_owned_into_existing(kind_str),
        appl_ref_into_existing(kind_str),
    ]
}

pub(crate) fn appl_to_ghosts(kind_str: &str) -> ApplicableTo {
    [
        appl_ghosts_owned(kind_str),
        appl_ghosts_ref(kind_str),
        appl_ghosts_owned(kind_str),
        appl_ghosts_ref(kind_str),
        appl_ghosts_owned(kind_str),
        appl_ghosts_ref(kind_str),
    ]
}

pub(crate) fn appl_to_ghost(kind_str: &str) -> ApplicableTo {
    [
        appl_ghost_owned(kind_str),
        appl_ghost_ref(kind_str),
        appl_ghost_owned(kind_str),
        appl_ghost_ref(kind_str),
        appl_ghost_owned(kind_str),
        appl_ghost_ref(kind_str),
    ]
}

fn appl_owned_into(kind_str: &str) -> bool {
    matches!(kind_str, "owned_into" | "into" | "map_owned" | "map" | "owned_try_into" | "try_into" | "try_map_owned" | "try_map")
}
fn appl_ref_into(kind_str: &str) -> bool {
    matches!(kind_str, "ref_into" | "into" | "map_ref" | "map" | "ref_try_into" | "try_into" | "try_map_ref" | "try_map")
}
fn appl_from_owned(kind_str: &str) -> bool {
    matches!(kind_str, "from_owned" | "from" | "map_owned" | "map" | "try_from_owned" | "try_from" | "try_map_owned" | "try_map")
}
fn appl_from_ref(kind_str: &str) -> bool {
    matches!(kind_str, "from_ref" | "from" | "map_ref" | "map" | "try_from_ref" | "try_from" | "try_map_ref" | "try_map")
}
fn appl_owned_into_existing(kind_str: &str) -> bool {
    matches!(kind_str, "owned_into_existing" | "into_existing" | "owned_try_into_existing" | "try_into_existing")
}
fn appl_ref_into_existing(kind_str: &str) -> bool {
    matches!(kind_str, "ref_into_existing" | "into_existing" | "ref_try_into_existing" | "try_into_existing")
}

fn appl_ghosts_owned(kind_str: &str) -> bool {
    matches!(kind_str, "ghosts" | "ghosts_owned")
}
fn appl_ghosts_ref(kind_str: &str) -> bool {
    matches!(kind_str, "ghosts" | "ghosts_ref")
}
fn appl_ghost_owned(kind_str: &str) -> bool {
    matches!(kind_str, "ghost" | "ghost_owned")
}
fn appl_ghost_ref(kind_str: &str) -> bool {
    matches!(kind_str, "ghost" | "ghost_ref")
}
