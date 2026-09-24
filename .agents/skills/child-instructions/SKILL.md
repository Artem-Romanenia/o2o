# `child` and `child_parents` instructions

## Field path notation in `#[child_parents(...)]` / `#[child(...)]`

- Field paths (e.g. `child`, `child.base`, `1`, `1 .1`) are always written in
  the **target type's own native notation** — named idents if the target's
  field is named, numeric index (with a space before a second dot-index, e.g.
  `1 .1`, to avoid the `1.1` float-literal lexing problem) if the target's
  field is a tuple index. This is **independent** of the deriving/DTO
  struct's own shape. E.g. even when the deriving struct is named, if its map
  target is a tuple struct, the child paths for that target are written as
  `1`, `1 .1`, etc.

## `#[child(Target| path)]` + `#[map(Target| selector)]` pairing

- `#[child(Target| path)]` declares "this DTO field belongs under the nested
  struct located at `path` within `Target`".
- `#[map(Target| selector)]` (or `#[map(Target| selector, expr)]`) then picks
  *which field of that nested struct* this DTO field corresponds to, using
  the nested struct's own field notation (name or index).
- The `selector` is **not always inferred automatically** even when shapes/positions
  would seem to line up — every test file that exercises child+tuple
  combinations spells out the selector explicitly (e.g.
  `#[map(TupleEntity| 0)]`). Only named-to-named, or *pure* top-level tuple-to-tuple,
  same-order conversions (no `child_parents` involved) get fully positional/automatic
  matching (see README "Tuple structs" section) — as soon as a field lives
  under a nested child, be explicit.
- When there are **multiple map targets** on the same struct (e.g. both a
  named `Entity` and a tuple `TupleEntity`), every `#[child(...)]`/
  `#[map(...)]` field attribute must be qualified with the container type
  (`#[child(Entity| child)]`, `#[child(TupleEntity| 1)]`) rather than left
  bare, to disambiguate which target it applies to.

## `child_parents` per-child actions (`=> (into: ..., from: ...)`)

- Syntax lets you attach `into:`/`from:` (and presumably `try_into:`/
  `try_from:`-style, gated by `ApplicableTo`) expressions directly to a
  child-type entry:
  ```
  #[child_parents(Entity|
      child: Child => (into: Some(@), from: ~.clone().unwrap()),
      child.base: Base => (into: @.as_ok(), from: custom_unwrap(~))
  )]
  ```
- `@` refers to the constructed/inner value going *into* the parent slot;
  `~` refers to the source value being read *from* the parent slot. Same
  `@`/`~` convention as regular field-level inline expressions elsewhere in
  o2o.
- The `into`/`from` keys use the same applicability matrix as top-level
  instructions (`appl_to` in `attr.rs`): `into` applies to both owned+ref
  "into" directions, `from` applies to both owned+ref "from" directions. This
  is why a single `from: ~.clone().unwrap()` works for both the owned and
  ref-based `From` impls even though cloning is only strictly required for
  the ref path — it's simply reused for both.