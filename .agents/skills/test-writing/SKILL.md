# Writing integration tests for o2o conversions

## 4-way (named/tuple × named/tuple) test naming pattern

Repo convention (see `o2o-tests/tests/47_deep_parameterized_parent_attr_tests.rs`
and `11_child_attr_mixed_struct_kinds_tests.rs` as an example) names tests by
`<source-shape>2<target-shape>[_reverse][_ref]`, giving 4 variants per
shape-pair:
- `xxx` — forward, owned (`dto.into()`)
- `xxx_reverse` — backward, owned (`entity.into()`)
- `xxx_ref` — forward, from a `&dto`
- `xxx_reverse_ref` — backward, from a `&entity`

For a struct that converts to/from two differently-shaped targets (one named,
one tuple), you end up needing **two DTO structs** (one named, one tuple) to
fully cover all 4 shape-pairs (named↔named, named↔tuple, tuple↔named,
tuple↔tuple) — a single DTO can't be both shapes at once.

## Fast iteration

- `cargo test -p o2o-tests --test <file_stem>` runs a single integration test
  file quickly instead of the whole suite.
- `o2o-tests`' default feature is `syn2` (see its `Cargo.toml`). Passing
  `--features syn1` *without* `--no-default-features` enables syn1 **and**
  the default syn2, tripping the "cannot be enabled at the same time"
  compile_error. Always run `cargo test -p o2o-tests --no-default-features
  --features syn1` (or `syn2`) for either half of the matrix.

## Writing a `_fallible` counterpart from a non-fallible test file

Field-level attributes (`#[map(...)]`, `#[child(...)]`) are parsed
identically regardless of `map`/`try_map` prefix (see `attr.rs`
`MemberInstruction::Map` match arms) — leave them completely unchanged when
deriving the fallible file. Only **container/type-level** instructions need
renaming, and each gains an error-type argument placed after the target type
(and after any `as ()`/`as {}` hint, before the `| vars(...)`/inline-expr
pipe if present):

| non-fallible      | fallible            | example                                              |
|--------------------|---------------------|-------------------------------------------------------|
| `map(Foo)`         | `try_map(Foo, Err)` | `try_map(Foo as (), anyhow::Error)`                    |
| `into(Foo)`        | `try_into(Foo, Err)`| `try_into(Foo as {}, anyhow::Error)`                   |
| `from_owned(Foo)`  | `try_from_owned(Foo, Err)` | `try_from_owned(Foo, anyhow::Error\| vars(...))` |
| `from_ref(Foo)`    | `try_from_ref(Foo, Err)`   | `try_from_ref(Foo, anyhow::Error\| vars(...))`   |
| `into_existing(Foo)` | `try_into_existing(Foo, Err)` | —                                          |

Then in test bodies, swap every `.into()` with `.try_into().unwrap()` and
`.into_existing(...)` with `.try_into_existing(...).unwrap()`; nothing else
in the test bodies needs to change. When the feature under test has no
naturally-fallible field (nothing that would legitimately produce an `Err`),
the repo's convention is to use a placeholder error type — `anyhow::Error`
and `String` both appear across existing `_fallible` files; pick whichever
neighboring/sibling test file in the same numeric group already uses.
