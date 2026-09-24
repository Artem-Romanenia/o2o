# AGENTS.md

This file is the primary source of grounding context for any AI agent working in this repository. You MUST read and follow it before making changes.

## 1. Project Context & Toolchain

- **Language**: Rust, edition **2021**. Do NOT introduce edition-2024-only syntax.
- **License**: Dual MIT OR Apache-2.0. Any new file with substantial code MUST be compatible with this dual license; NEVER add code copied from incompatible-license sources.
- **Workspace**: Cargo workspace defined at the repo root `Cargo.toml`, with members:
  - `o2o-impl` — core proc-macro implementation logic (parsing, AST, validation, code generation). No `proc-macro = true`.
  - `o2o-macros` — the actual `proc-macro = true` crate exposing `#[derive(o2o)]`. Thin wrapper only; MUST NOT contain business logic.
  - `o2o-tests` — integration test crate (not published), exercises the public `o2o` crate end-to-end.
  - Root package `o2o` (`src/`) — the public-facing crate that users depend on; re-exports the derive macro and hosts `traits.rs`.
- **Key dependencies**: `syn` (both `1.0.3` and `>=2.0.0,<4.0.0`, mutually exclusive via `syn1`/`syn2` features), `proc-macro2 "1.0.0"`, `quote "1.0.0"`. Dev/test deps: `criterion "0.4"` (benchmarks), `test-case "3"` (parameterized tests), `anyhow "1.0.86"` (integration tests only).
- **Dual `syn` version support is a REQUIRED, load-bearing feature.** `o2o-impl` and `o2o-macros` both gate code behind `feature = "syn"` (syn v1) and `feature = "syn2"` (syn v2). There is a compile-time guard preventing both being enabled simultaneously — NEVER remove or weaken this guard, and NEVER write code that only compiles under one of the two syn versions unless it is properly feature-gated.
- **`#![no_std]` is REQUIRED on the root `o2o` crate** (`src/lib.rs`). Any change to `src/` MUST preserve `no_std` compatibility — do NOT add `std`-only imports there without an explicit, feature-gated opt-in.
- **Core architecture** — a proc-macro expansion pipeline in `o2o-impl/src`, executed in this strict order:
  1. `kw.rs` — custom keyword tokens (`vars`, `repeat`, `permeate`, `skip_repeat`, `stop_repeat`, `Unit`, `attribute`, `impl_attribute`, `inner_attribute`) via `syn::custom_keyword!`.
  2. `attr.rs` — parses raw `syn::Attribute`s into structured `DataTypeInstruction` / `MemberInstruction` enums and the `Kind`/`FallibleKind` trait-selection model (`from_owned`, `owned_try_into`, etc.).
  3. `ast.rs` — converts `syn::DeriveInput` into o2o's own `Struct`/`Enum`/`Field`/`Variant`/`DataType` AST, attaching parsed attributes to each node.
  4. `validate.rs` — semantic validation of the AST (`validate(&DataType) -> Result<()>`); MUST run before code generation and MUST aggregate/report errors rather than panicking.
  5. `expand.rs` — the `derive(&DeriveInput) -> Result<TokenStream>` entry point; generates `From`/`TryFrom`/`Into`/`TryInto`/`IntoExisting`/`TryIntoExisting` impls.
  - `o2o-macros/src/lib.rs` only calls `o2o_impl::expand::derive` and converts the result to `proc_macro::TokenStream`. Any new logic MUST go in `o2o-impl`, not here.
  - `src/traits.rs` defines the two custom traits (`IntoExisting<T>`, `TryIntoExisting<T>`) that back the `owned_into_existing`/`ref_into_existing` instructions. These are public API — changing their signatures is a breaking change.

## 2. Testing & Quality Requirements

- **Unit tests** for the parsing/expansion pipeline live in `o2o-impl/src/tests.rs`, using the `test_case` crate to feed `quote!{...}` token streams into `expand::derive` and assert on success/failure. Add new pipeline-level tests here when changing `attr.rs`, `ast.rs`, `validate.rs`, or `expand.rs`.
- **Integration tests** live in `o2o-tests/tests/`, one file per feature area, following the established naming convention: `{N}_{short_description}[_fallible][_N].rs` (e.g. [1_bare_top_level_attr_tests.rs](o2o-tests/tests/1_bare_top_level_attr_tests.rs), [1_bare_top_level_attr_tests_fallible.rs](o2o-tests/tests/1_bare_top_level_attr_tests_fallible.rs)). A `_fallible` counterpart tests the `Try*` trait variants of the same feature and MUST be added alongside any new non-fallible test file that exercises a feature with a fallible equivalent.
  - When adding a new o2o instruction or attribute capability, create a NEW numbered test file (next available number) rather than overloading an unrelated existing file.
  - Test files compile real structs/enums with `#[derive(o2o::o2o)]` and assert on the generated conversions — treat these as executable specification/documentation. NEVER weaken or delete an existing test to make a change pass; fix the implementation instead.
- **Benchmarks** live in `o2o-impl/benches/o2o_benchmarks.rs` (Criterion). These are for macro-expansion performance only — do NOT treat them as correctness tests.
- **Feature-matrix testing is REQUIRED only when a non-testing crate was touched.** The repo builds against two mutually exclusive `syn` versions and both `std`/`no_std` re-export configurations. If a change touches `o2o-impl`, `o2o-macros`, or the root `o2o` crate (`src/`), you MUST run [tests.sh](tests.sh) (or the equivalent individual commands below) before considering the change complete:
  ```bash
  cargo test -q -p o2o
  cargo test -q -p o2o --no-default-features --features syn1
  cargo test -q -p o2o --no-default-features --features syn2
  cargo test -q -p o2o-tests --no-default-features --features syn1
  cargo test -q -p o2o-tests --no-default-features --features syn2
  cargo test -q -p o2o-impl --no-default-features --features syn
  cargo test -q -p o2o-impl --no-default-features --features syn2
  ```
  A change that only passes with the default feature set is NOT considered done.
  If a change is confined to `o2o-tests/tests/` (e.g. adding/updating integration test files only), it is sufficient to run just the newly-created/modified test(s) (e.g. `cargo test -q -p o2o-tests --features syn1 <test_name>`) — the full matrix above is not required.
- CI (`.github/workflows/build.yml`) runs `cargo build`, doc tests (default/syn1/syn2), and integration/impl tests across both syn features on every push/PR to `main`. Do NOT introduce changes that only work locally and are not covered by this matrix.
- Formatting is governed by [rustfmt.toml](rustfmt.toml) (wide line limits: `max_width = 300`, `chain_width = 200`, `fn_call_width = 250`, `match_block_trailing_comma = true`). Run `cargo fmt` using this config before finishing; do NOT hand-format against different conventions.
- There is no `clippy.toml` or `#![deny(...)]` lint gate today — do NOT assume strict clippy/lint CI enforcement exists, but also do NOT introduce obvious clippy warnings; keep code idiomatic.

## 3. Guardrails & Safety

- **NEVER** push commits, force-push, or publish to crates.io — this is a published package and such actions affect real downstream consumers.
- **NEVER** enable both the `syn` (v1) and `syn2` features at the same time, and NEVER remove the compile-time guard in `o2o-impl`/`o2o-macros` that prevents this.
- **NEVER** delete, rename, or bypass the pipeline stages (`attr.rs` → `ast.rs` → `validate.rs` → `expand.rs`) or call `expand`-level code generation without going through `validate` first. Validation failures MUST surface as compile errors on the user's derive, not panics inside the proc macro.
- **NEVER** hardcode secrets, tokens, or credentials anywhere in this repo — it is a published, open-source crate on crates.io with public CI; there is no legitimate use for secrets in source or tests.
- **NEVER** delete or gut existing test files in `o2o-tests/tests/` or `o2o-impl/src/tests.rs` to silence a failure — a failing test indicates a regression that MUST be fixed in the implementation, not removed. If a test is genuinely obsolete, this requires explicit user confirmation before deletion.
- **NEVER** remove `#![no_std]` from the root crate, and NEVER add a `std`-only dependency to the root `o2o` package (`src/`) without gating it behind a new, explicit feature.
- **NEVER** break the public API surface (`o2o::traits::IntoExisting`, `TryIntoExisting`, the derive macro's supported attribute syntax documented in [README.md](README.md)) without treating it as a breaking/major version change — this crate is versioned and consumed by external users pinned to `o2o = "0.6.0"`-style version ranges.
- **NEVER** bypass or silence compiler/proc-macro errors by wrapping generated code in blanket `#[allow(...)]` unless there is a precise, justified reason; do not use broad allows to hide real bugs.
- The [README.md](README.md) contains ~50+ `rust` doctest code blocks that are compiled as part of CI doc tests. If you change macro behavior, you MUST update the corresponding README example and its "View generated code" expansion so they stay in sync and doctests keep passing.

## 4. AI Continuous Memory & Skills
- **Context Awareness**: A local knowledge base of project gotchas, hidden rules, and past discoveries lives in the `.agents/` directory. 
- **Pre-flight Requirement**: Before starting any architectural or complex task, you MUST read `.agents/MEMORY.md` to avoid repeating past engineering mistakes.
- **Post-task Requirement**: Upon completing a major feature or resolving a complex bug, you MUST update `.ai/MEMORY.md` and/or create a new skill file in `.agent/skills/` outlining any discovered hidden rules, unique notations, or edge cases.
