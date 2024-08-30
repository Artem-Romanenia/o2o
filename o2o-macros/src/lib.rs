extern crate o2o_impl;

use o2o_impl::expand::derive;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// ### Object to Object mapper for Rust
/// **o2o** can implement ...
/// * `::core::convert::From<T>`
/// * `::core::convert::TryFrom<T>`
/// * `::core::convert::Into<T>`
/// * `::core::convert::TryInto<T>`
/// * `o2o::traits::IntoExisting<T>`
/// * `o2o::traits::TryIntoExisting<T>`
///  
/// ... traits via procedural macro.
///
/// ### Basic example
///
/// ``` rust
/// use o2o::o2o;
///
/// struct Person {
///     id: u32,
///     name: String,
///     age: u8
/// }
///
/// #[derive(o2o)]
/// #[from_owned(Person)] // This tells o2o to generate 'From<Person> for PersonDto' implementation
/// #[owned_try_into(Person, std::io::Error)] // This generates 'TryInto<Person> for PersonDto' with type Error = std::io::Error
/// struct PersonDto {
///     id: u32,
///     name: String,
///     age: u8
/// }
///
/// // Applying #[derive(o2o)] on PersonDto allows you to do this:
///
/// let person = Person { id: 123, name: "John".into(), age: 42 };
/// let dto = PersonDto::from(person);
///
/// assert_eq!(dto.id, 123); assert_eq!(dto.name, "John"); assert_eq!(dto.age, 42);
///
/// // and this:
///
/// let dto = PersonDto { id: 321, name: "Jack".into(), age: 23 };
/// let person: Person = dto.try_into().unwrap();
///
/// assert_eq!(person.id, 321); assert_eq!(person.name, "Jack"); assert_eq!(person.age, 23);
///
/// // o2o also supports enums:
///
/// enum Creature {
///     Person(Person),
///     Cat { nickname: String },
///     Dog(String),
///     Other
/// }
///
/// #[derive(o2o)]
/// #[from_owned(Creature)]
/// enum CreatureDto {
///     Person(#[from(~.into())] PersonDto),
///     Cat { nickname: String },
///     Dog(String),
///     Other
/// }
///
/// let creature = Creature::Cat { nickname: "Floppa".into() };
/// let dto: CreatureDto = creature.into();
///
/// if let CreatureDto::Cat { nickname } = dto { assert_eq!(nickname, "Floppa"); } else { assert!(false) }
/// ```
///
/// ### For more examples, visit [github.com](https://github.com/Artem-Romanenia/o2o)
#[proc_macro_derive(
    o2o,
    attributes(
        owned_into,
        ref_into,
        into,
        from_owned,
        from_ref,
        from,
        map_owned,
        map_ref,
        map,
        owned_try_into,
        ref_try_into,
        try_into,
        owned_into_existing,
        ref_into_existing,
        into_existing,
        try_from_owned,
        try_from_ref,
        try_from,
        try_map_owned,
        try_map_ref,
        try_map,
        owned_try_into_existing,
        ref_try_into_existing,
        try_into_existing,
        child,
        children,
        parent,
        ghost,
        ghosts,
        where_clause,
        literal,
        pattern,
        type_hint,
        o2o
    )
)]
// TODO: Research, are there any downsides to having that many attributes?
// (given that all but one are essentially shortcuts and can be avoided with alternative instr syntax)
pub fn derive_o2o(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
