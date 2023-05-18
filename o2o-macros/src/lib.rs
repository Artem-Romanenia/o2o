extern crate o2o_impl;

use proc_macro::TokenStream;
use o2o_impl::expand::derive;
use syn::{parse_macro_input, DeriveInput};

/// ### Object to Object mapper for Rust
/// **o2o** can implement `std::convert::From<T>`, `std::convert::Into<T>`, and custom `o2o::traits::IntoExisting<T>` traits via procedural macro. It can be best explained through examples, so...
///
/// #### Simplest Case
/// 
/// ``` rust
/// use o2o::o2o;
/// 
/// struct Entity {
///     some_int: i32,
///     another_int: i16,
/// }
/// 
/// #[derive(o2o)]
/// #[map(Entity)]
/// struct EntityDto {
///     some_int: i32,
///     another_int: i16,
/// }
/// ```
/// For the above code **o2o** will generate following trait impls:
///
/// ``` rust
/// impl std::convert::From<Entity> for EntityDto { ... }
/// impl std::convert::From<&Entity> for EntityDto { ... }
/// impl std::convert::Into<Entity> for EntityDto { ... }
/// impl std::convert::Into<Entity> for &EntityDto { ... }
/// ```
/// #### Different field name
/// 
/// ``` rust
/// struct Entity {
///     some_int: i32,
///     another_int: i16,
/// }
/// 
/// #[derive(o2o)]
/// #[map(Entity)]
/// struct EntityDto {
///     some_int: i32,
///     #[map(another_int)]
///     different_int: i16,
/// }
/// ```
/// 
/// #### Different field type
/// 
/// ``` rust
/// struct Entity {
///     some_int: i32,
///     value: i16,
/// }
/// 
/// #[derive(o2o)]
/// #[map(Entity)]
/// struct EntityDto {
///     some_int: i32,
///     #[from(value.to_string())] //here `value` is a field of Entity struct
///     #[into(value.parse::<i16>().unwrap())] //here `value` is a field of EntityDto struct
///     value: String,
/// }
/// ```
/// #### Nested structs
/// 
/// ``` rust
/// struct Entity {
///     some_int: i32,
///     child: Child,
/// }
/// struct Child {
///     child_int: i32,
/// }
/// 
/// #[derive(o2o)]
/// #[map_owned(Entity)]
/// struct EntityDto {
///     some_int: i32,
///     #[map(child.into())]
///     child: ChildDto
/// }
/// 
/// #[derive(o2o)]
/// #[map_owned(Child)]
/// struct ChildDto {
///     child_int: i32,
/// }
/// ```
/// 
/// #### Nested collection
/// 
/// ``` rust
/// struct Entity {
///     some_int: i32,
///     children: Vec<Child>,
/// }
/// struct Child {
///     child_int: i32,
/// }
/// 
/// #[derive(o2o)]
/// #[map(Entity)]
/// struct EntityDto {
///     some_int: i32,
///     #[map(children.iter().map(|p|p.into()).collect())]
///     children: Vec<ChildDto>
/// }
/// 
/// #[derive(o2o)]
/// #[map(Child)]
/// struct ChildDto {
///     child_int: i32,
/// }
/// ```
/// 
/// For more examples, visit [github.com](https://github.com/Artem-Romanenia/o2o)
#[proc_macro_derive(o2o, attributes(
    owned_into, ref_into, into, 
    from_owned, from_ref, from, 
    map_owned, map_ref, map, 
    owned_into_existing, ref_into_existing, into_existing,
    child, children, parent, ghost, panic_debug_info, where_clause, o2o))]
pub fn derive_o2o(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}