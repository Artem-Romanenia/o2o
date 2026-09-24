# Type hints: `as ()` and `as {}`

## Default shape rule

- o2o assumes the type on the *other side* of a conversion has the **same
  shape** (named-fields vs. tuple/unnamed-fields) as the struct the
  `#[derive(o2o)]` is placed on. E.g. if the deriving struct is a named
  struct, every referenced foreign type (map target, or a child type in
  `child_parents`) is assumed to be named unless hinted otherwise.
- When the actual shape of the *other* type differs from that assumption, add
  an explicit hint right after the type path:
  - `as ()` — the other side is a **tuple struct** (or bare tuple type).
  - `as {}` — the other side is a **named struct**.

## Where hints apply

Hints appear at multiple, independent levels wherever a foreign type path is
written:
- Top-level `#[map(Foo as ())]` / `#[from(Foo as {})]` etc. (container-level
  map instructions).
- Per-entry in `#[child_parents(Foo| child: ChildTy as () => (...), ...)]` —
  the hint is on `ChildTy`, not on `Foo` (the container type at the head of
  the attribute never takes a hint there; its shape is already established by
  the corresponding top-level `#[map(Foo ...)]`/`#[from(Foo ...)]` attribute).
- `#[type_hint(as ())]` / `#[type_hint(as {})]` standalone on enum variants
  (see README "Type hints" section).

## Practical consequence

If the deriving struct is a **tuple struct** and you add `child_parents`
entries pointing at **named** child structs, every such child type needs
`as {}`; child types that are themselves tuple structs need no hint (they
already match the deriving struct's own shape). The symmetric statement holds
when the deriving struct is named and children are tuples (needs `as ()`).
