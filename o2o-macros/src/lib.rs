extern crate o2o_impl;

use proc_macro::TokenStream;
use o2o_impl::expand::derive;
use syn::{parse_macro_input, DeriveInput};

/// ### Object to Object mapper for Rust
/// **o2o** can implement `std::convert::From<T>`, `std::convert::Into<T>`, and custom `o2o::traits::IntoExisting<T>` traits via procedural macro. It can be best explained through examples, so...
///
/// ### Simplest Case
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
/// #[map_owned(Entity)]
/// struct EntityDto {
///     some_int: i32,
///     another_int: i16,
/// }
/// ```
/// <details>
///   <summary>View generated code</summary>
/// 
///   ``` rust
///   impl std::convert::From<Entity> for EntityDto {
///       fn from(value: Entity) -> EntityDto {
///           EntityDto {
///               some_int: value.some_int,
///               another_int: value.another_int,
///           }
///       }
///   }
///   impl std::convert::Into<Entity> for EntityDto {
///       fn into(self) -> Entity {
///           Entity {
///               some_int: self.some_int,
///               another_int: self.another_int,
///           }
///       }
///   }
///   ```
/// </details>
/// 
/// With the above code you should be able to do this:
/// 
/// ``` rust
/// let entity = Entity { some_int: 123, another_int: 321 }
/// let dto: EntityDto = entity.into();
/// // and this:
/// let dto = EntityDto { some_int: 123, another_int: 321 }
/// let entity: Entity = dto.into();
/// ```
/// 
/// ### Different field name
/// 
/// ``` rust
/// struct Entity {
///     some_int: i32,
///     another_int: i16,
/// }
/// 
/// #[derive(o2o)]
/// #[from_ref(Entity)]
/// #[ref_into_existing(Entity)]
/// struct EntityDto {
///     some_int: i32,
///     #[map(another_int)]
///     different_int: i16,
/// }
/// ```
/// <details>
///   <summary>View generated code</summary>
/// 
///   ``` rust
///   impl std::convert::From<&Entity> for EntityDto {
///       fn from(value: &Entity) -> EntityDto {
///           EntityDto {
///               some_int: value.some_int,
///               different_int: value.another_int,
///           }
///       }
///   }
///   impl o2o::traits::IntoExisting<Entity> for &EntityDto {
///       fn into_existing(self, other: &mut Entity) {
///           other.some_int = self.some_int;
///           other.another_int = self.different_int;
///       }
///   }
///   ```
/// </details>
/// 
/// ### Different field type
/// 
/// ``` rust
/// struct Entity {
///     some_int: i32,
///     val: i16,
///     str: String
/// }
/// 
/// #[derive(o2o)]
/// #[map(Entity)]
/// struct EntityDto {
///     some_int: i32,
///     #[from(~.to_string())] // Tilde allows to append code at the end of the right side of field initialization for From<T> impls
///     #[into(~.parse::<i16>().unwrap())] // here it's the same but for Into<T> impls
///     val: String,
///     // Here Into and From are symmetric, so it has to be only specified once.
///     // Note that .clone() is only needed for borrowing impls, so we use #[map_ref()]
///     #[map_ref(~.clone())] 
///     str: String
/// }
/// ```
/// <details>
///   <summary>View generated code</summary>
/// 
///   ``` rust
///   impl std::convert::From<Entity> for EntityDto {
///       fn from(value: Entity) -> EntityDto {
///           EntityDto {
///               some_int: value.some_int,
///               val: value.val.to_string(),
///               str: value.str, // no .clone()
///           }
///       }
///   }
///   impl std::convert::From<&Entity> for EntityDto {
///       fn from(value: &Entity) -> EntityDto {
///           EntityDto {
///               some_int: value.some_int,
///               val: value.val.to_string(),
///               str: value.str.clone(),
///           }
///       }
///   }
///   impl std::convert::Into<Entity> for EntityDto {
///       fn into(self) -> Entity {
///           Entity {
///               some_int: self.some_int,
///               val: self.val.parse::<i16>().unwrap(),
///               str: self.str, // no .clone()
///           }
///       }
///   }
///   impl std::convert::Into<Entity> for &EntityDto {
///       fn into(self) -> Entity {
///           Entity {
///               some_int: self.some_int,
///               val: self.val.parse::<i16>().unwrap(),
///               str: self.str.clone(),
///           }
///       }
///   }
///   ```
/// </details>
/// 
/// ### Nested structs
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
/// #[from_owned(Entity)]
/// struct EntityDto {
///     some_int: i32,
///     #[map(~.into())]
///     child: ChildDto
/// }
/// 
/// #[derive(o2o)]
/// #[from_owned(Child)]
/// struct ChildDto {
///     child_int: i32,
/// }
/// ```
/// <details>
///   <summary>View generated code</summary>
/// 
///   ``` rust
///   impl std::convert::From<Entity> for EntityDto {
///       fn from(value: Entity) -> EntityDto {
///           EntityDto {
///               some_int: value.some_int,
///               child: value.child.into(),
///           }
///       }
///   }
///   
///   impl std::convert::From<Child> for ChildDto {
///       fn from(value: Child) -> ChildDto {
///           ChildDto {
///               child_int: value.child_int,
///           }
///       }
///   }
///   ```
/// </details>
/// 
/// ### Nested collection
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
/// #[map_owned(Entity)]
/// struct EntityDto {
///     some_int: i32,
///     // Here field name as well as type are different, so we pass in field name and tilde inline expression.
///     // Also, it doesn't hurt to use member level instruction #[map()], 
///     // which is broader than struct level instruction #[map_owned]
///     #[map(children, ~.iter().map(|p|p.into()).collect())]
///     children_vec: Vec<ChildDto>
/// }
/// 
/// #[derive(o2o)]
/// #[map_ref(Child)]
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
    child, children, parent, ghost, where_clause, o2o))]
pub fn derive_o2o(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}