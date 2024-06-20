﻿Object to Object mapper for Rust. Derive `(Try)From`, and `(Try)Into` traits.<!-- omit from toc --> 
================================
[<img alt="github.com" src="https://github.com/Artem-Romanenia/o2o/workflows/Build/badge.svg" height="25">](https://github.com/Artem-Romanenia/o2o/)
[<img alt="crates.io" src="https://img.shields.io/crates/v/o2o.svg?style=for-the-badge&color=2f4d28&labelColor=f9f7ec&logo=rust&logoColor=black" height="25">](https://crates.io/crates/o2o)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-o2o-444444?style=for-the-badge&labelColor=aaaaaa&logo=docs.rs" height="25">](https://docs.rs/o2o)

## Quick pitch<!-- omit from toc --> 

``` rust ignore
impl From<Person> for PersonDto {
    fn from(value: Person) -> PersonDto {
        PersonDto {
            id: value.id,
            name: value.name,
            age: value.age,
        }
    }
}
```

Writing code like above is not the most exciting or emotionally rewarding part of working with Rust. If you're Ok with letting procedural macro write it for you, welcome to the rest of this page.

## Basic Example<!-- omit from toc --> 

``` rust
use o2o::o2o;

struct Person {
    id: u32,
    name: String,
    age: u8
}

#[derive(o2o)]
#[from_owned(Person)] // This tells o2o to generate 'From<Person> for PersonDto' implementation
#[owned_try_into(Person, std::io::Error)] // This generates 'TryInto<Person> for PersonDto' with type Error = std::io::Error
struct PersonDto {
    id: u32,
    name: String,
    age: u8
}

// Applying #[derive(o2o)] on PersonDto allows you to do this:

let person = Person { id: 123, name: "John".into(), age: 42 };
let dto = PersonDto::from(person);

assert_eq!(dto.id, 123); assert_eq!(dto.name, "John"); assert_eq!(dto.age, 42);

// and this:

let dto = PersonDto { id: 321, name: "Jack".into(), age: 23 };
let person: Person = dto.try_into().unwrap();

assert_eq!(person.id, 321); assert_eq!(person.name, "Jack"); assert_eq!(person.age, 23);

// o2o also supports enums:

enum Creature {
    Person(Person),
    Cat { nickname: String },
    Dog(String),
    Other
}

#[derive(o2o)]
#[from_owned(Creature)]
enum CreatureDto {
    Person(#[from(~.into())] PersonDto),
    Cat { nickname: String },
    Dog(String),
    Other
}

let creature = Creature::Cat { nickname: "Floppa".into() };
let dto: CreatureDto = creature.into();

if let CreatureDto::Cat { nickname } = dto { assert_eq!(nickname, "Floppa"); } else { assert!(false) }
```

And here's the code that `o2o` generates (from here on, generated code is produced by [rust-analyzer: Expand macro recursively](https://rust-analyzer.github.io/manual.html#expand-macro-recursively) command):
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<Person> for PersonDto {
      fn from(value: Person) -> PersonDto {
          PersonDto {
              id: value.id,
              name: value.name,
              age: value.age,
          }
      }
  }
  impl std::convert::TryInto<Person> for PersonDto {
      type Error = std::io::Error;
      fn try_into(self) -> Result<Person, std::io::Error> {
          Ok(Person {
              id: self.id,
              name: self.name,
              age: self.age,
          })
      }
  }

  impl std::convert::From<Creature> for CreatureDto {
      fn from(value: Creature) -> CreatureDto {
          match value {
              Creature::Person(f0) => CreatureDto::Person(f0.into()),
              Creature::Cat { nickname } => CreatureDto::Cat { nickname: nickname },
              Creature::Dog(f0) => CreatureDto::Dog(f0),
              Creature::Other => CreatureDto::Other,
          }
      }
  }
  ```
</details>

## Some milestones<!-- omit from toc --> 

* **v0.4.4** Fallible conversions
* **v0.4.3** Enum-to-primitive type conversions with `#[literal(...)]` and `#[pattern(...)]`
* **v0.4.2** Basic enum conversions
* **...**

## Content<!-- omit from toc --> 

- [Traits and `o2o` *trait instructions*](#traits-and-o2o-trait-instructions)
- [The (not so big) Problem](#the-not-so-big-problem)
- [Inline expressions](#inline-expressions)
- [Struct Examples](#struct-examples)
  - [Different member name](#different-member-name)
  - [Different field type](#different-field-type)
  - [Nested structs](#nested-structs)
  - [Nested collection](#nested-collection)
  - [Assymetric fields (skipping and providing default values)](#assymetric-fields-skipping-and-providing-default-values)
  - [Use struct update syntax (..Default::default())](#use-struct-update-syntax-defaultdefault)
  - [Define helper variables](#define-helper-variables)
  - [Quick return](#quick-return)
  - [Slightly complex example](#slightly-complex-example)
  - [Flatened children](#flatened-children)
  - [Tuple structs](#tuple-structs)
  - [Tuples](#tuples)
  - [Type hints](#type-hints)
  - [Generics](#generics)
  - [Where clauses](#where-clauses)
  - [Mapping to multiple structs](#mapping-to-multiple-structs)
  - [Avoiding proc macro attribute name collisions (alternative instruction syntax)](#avoiding-proc-macro-attribute-name-collisions-alternative-instruction-syntax)
  - [Additional o2o instruction available via `#[o2o(...)]` syntax](#additional-o2o-instruction-available-via-o2o-syntax)
    - [Primitive type conversions](#primitive-type-conversions)
    - [Repeat instructions](#repeat-instructions)
- [Enum Examples](#enum-examples)
  - [Different variant name](#different-variant-name)
  - [Mapping to primitive types](#mapping-to-primitive-types)
    - [Using literals](#using-literals)
    - [Using patterns](#using-patterns)
    - [Using literals and patterns together](#using-literals-and-patterns-together)
- [Contributions](#contributions)
- [License](#license)

## Traits and `o2o` *trait instructions*

To let o2o know what traits you want implemented, you have to use type-level `o2o` *trait instructions* (i.e. proc macro attributes):

``` rust
struct Entity { }

#[derive(o2o::o2o)]
#[from_ref(Entity)] // This tells o2o to generate 'From<&Entity> for EntityDto' implementation
struct EntityDto { }
```

o2o procedural macro is able to generate implementation of 12 kinds of traits:

``` rust ignore
// When applied to a struct B:

// #[from_owned(A)]
impl std::convert::From<A> for B { ... }

// #[try_from_owned(A)]
impl std::convert::TryFrom<A> for B { ... }

// #[from_ref(A)]
impl std::convert::From<&A> for B { ... }

// #[try_from_ref(A)]
impl std::convert::TryFrom<&A> for B { ... }

// #[owned_into(A)]
impl std::convert::Into<A> for B { ... }

// #[try_owned_into(A)]
impl std::convert::TryInto<A> for B { ... }

// #[ref_into(A)]
impl std::convert::Into<A> for &B { ... }

// #[try_ref_into(A)]
impl std::convert::TryInto<A> for &B { ... }

// #[owned_into_existing(A)]
impl o2o::traits::IntoExisting<A> for B { ... }

// #[owned_try_into_existing(A)]
impl o2o::traits::TryIntoExisting<A> for B { ... }

// #[ref_into_existing(A)]
impl o2o::traits::IntoExisting<A> for &B { ... }

// #[ref_try_into_existing(A)]
impl o2o::traits::TryIntoExisting<A> for &B { ... }
```

o2o also has shortcuts to configure multiple trait implementations with fewer lines of code:

|                              | #[map()] | #[from()]  | #[into()] | #[map_owned()] | #[map_ref()] | #[into_existing()] |
| ---------------------------- | -------- | ---------- | --------- | ---------------| ------------ | -------------------|
| **#[from_owned()]**          | ✔️       | ✔️          | ❌        | ✔️             | ❌           | ❌                |
| **#[from_ref()]**            | ✔️       | ✔️          | ❌        | ❌            | ✔️            | ❌                |
| **#[owned_into()]**          | ✔️       | ❌         | ✔️         | ✔️             | ❌           | ❌                |
| **#[ref_into()]**            | ✔️       | ❌         | ✔️         | ❌            | ✔️            | ❌                |
| **#[owned_into_existing()]** | ❌      | ❌         | ❌        | ❌            | ❌           | ✔️                 |
| **#[ref_into_existing()]**   | ❌      | ❌         | ❌        | ❌            | ❌           | ✔️                 |

E.g. following two bits of code are equivalent:

``` rust
struct Entity { }

#[derive(o2o::o2o)]
#[map(Entity)]
struct EntityDto { }
```

``` rust
struct Entity { }

#[derive(o2o::o2o)]
#[from_owned(Entity)]
#[from_ref(Entity)]
#[owned_into(Entity)]
#[ref_into(Entity)]
struct EntityDto { }
```

**Exactly the same shortcuts apply to *fallible* conversions.**

## The (not so big) Problem

This section may be useful for people which are not very familiar with Rust's procedural macros and it explains why some things are done the way they're done.

Being procedural macro, o2o has knowledge only about the side of the mapping where `#[derive(o2o)]` is applied.

``` rust ignore
#[derive(o2o::o2o)]
#[map(Entity)]
struct EntityDto { }
```

In code above, o2o knows everything about `EntityDto`, but it knows nothing about `Entity`. It doens't know if it is a struct, doesn't know what fields it has, doesn't know if it is a struct or a tuple, *it doesn't even know if it exists*.

So unlike mappers from languages like C#, Java, Go etc. that can use reflection to find out what they need to know, `o2o` can only assume things.

For the piece of code above, o2o will assume that:

* `Entity` exists *(duh!)*
* `Entity` is the same data type that `EntityDto` is (in this case a struct)
* `Entity` has exactly the same fields that `EntityDto` has

If o2o is wrong in any of its assumptions, you will have to tell it that.

## Inline expressions

o2o has a concept of Inline Expressions, which can be passed as a parameter to some of the o2o instructions. You can think of inline expression as a closure, which always has two *implicit* params: `|@, ~| {` **...expression body...** `}` or `|@, ~|` **{ ...expression body... }**

* `@` represents the object that is being converted from.
* `~` represents the path to a specific field of the object that is being converted from.

``` rust
struct Entity { some_int: i32 }

#[derive(o2o::o2o)]
#[map_owned(Entity)] // tells o2o to implement 'From<Entity> for EntityDto' and 'Into<Entity> for EntityDto'
struct EntityDto { 
    #[from(~ * 2)] // Let's say for whatever reason we want to multiply 'some_int' by 2 when converting from Entity
    #[into(~ / 2)] // And divide back by 2 when converting into it
    some_int: i32
}
```

This example will be expanded into the following code:

``` rust ignore
impl std::convert::From<Entity> for EntityDto {
    fn from(value: Entity) -> EntityDto {
        EntityDto {
            some_int: value.some_int * 2, // '~' got replaced by 'value.some_int' for From<> implementation
        }
    }
}
impl std::convert::Into<Entity> for EntityDto {
    fn into(self) -> Entity {
        Entity {
            some_int: self.some_int / 2, // '~' got replaced by 'self.some_int' for Into<> implementation
        }
    }
}
```

To achieve the same result, `@` could have been used:

``` rust
struct Entity { some_int: i32 }

#[derive(o2o::o2o)]
#[map_owned(Entity)]
struct EntityDto { 
    #[from(@.some_int * 2)]
    #[into(@.some_int / 2)]
    some_int: i32
}
```

This expands into exactly the same code:

``` rust ignore
impl std::convert::From<Entity> for EntityDto {
    fn from(value: Entity) -> EntityDto {
        EntityDto {
            some_int: value.some_int * 2, // '@' got replaced by 'value' for From<> implementation
        }
    }
}
impl std::convert::Into<Entity> for EntityDto {
    fn into(self) -> Entity {
        Entity {
            some_int: self.some_int / 2, // '@' got replaced by 'self' for Into<> implementation
        }
    }
}
```

You can use `~` for inline expressions that are passed only to member level o2o instructions, while `@` can be used at both member and type level.

So finally, let's look at some examples.

## Struct Examples

### Different member name

``` rust
use o2o::o2o;

struct Entity {
    some_int: i32,
    another_int: i16,
}

enum EntityEnum {
    Entity(Entity),
    SomethingElse { field: i32 }
}

#[derive(o2o)]
#[map_ref(Entity)]
struct EntityDto {
    some_int: i32,
    #[map(another_int)]
    different_int: i16,
}

#[derive(o2o)]
#[map_ref(EntityEnum)]
enum EntityEnumDto {
    #[map(Entity)]
    EntityDto(#[map(~.into())]EntityDto),
    SomethingElse { 
        #[map(field, *~)]
        f: i32 
    }
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<&Entity> for EntityDto {
      fn from(value: &Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int,
              different_int: value.another_int,
          }
      }
  }
  impl o2o::traits::IntoExisting<Entity> for &EntityDto {
      fn into_existing(self, other: &mut Entity) {
          other.some_int = self.some_int;
          other.another_int = self.different_int;
      }
  }

  impl std::convert::From<&EntityEnum> for EntityEnumDto {
      fn from(value: &EntityEnum) -> EntityEnumDto {
          match value {
              EntityEnum::Entity(f0) => EntityEnumDto::EntityDto(f0.into()),
              EntityEnum::SomethingElse { field } => EntityEnumDto::SomethingElse { f: *field },
          }
      }
  }
  impl std::convert::Into<EntityEnum> for &EntityEnumDto {
      fn into(self) -> EntityEnum {
          match self {
              EntityEnumDto::EntityDto(f0) => EntityEnum::Entity(f0.into()),
              EntityEnumDto::SomethingElse { f } => EntityEnum::SomethingElse { field: *f },
          }
      }
  }
  ```
</details>

### Different field type

``` rust
use o2o::o2o;

struct Entity {
    some_int: i32,
    str: String,
    val: i16
}

#[derive(o2o)]
#[from(Entity)]
#[try_into(Entity, std::num::ParseIntError)]
struct EntityDto {
    some_int: i32,
    #[map_ref(@.str.clone())] 
    str: String,
    #[from(~.to_string())]
    #[into(~.parse::<i16>()?)]
    val: String
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<Entity> for EntityDto {
      fn from(value: Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int,
              str: value.str,
              val: value.val.to_string(),
          }
      }
  }
  impl std::convert::From<&Entity> for EntityDto {
      fn from(value: &Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int,
              str: value.str.clone(),
              val: value.val.to_string(),
          }
      }
  }
  impl std::convert::TryInto<Entity> for EntityDto {
      type Error = std::num::ParseIntError;
      fn try_into(self) -> Result<Entity, std::num::ParseIntError> {
          Ok(Entity {
              some_int: self.some_int,
              str: self.str,
              val: self.val.parse::<i16>()?,
          })
      }
  }
  impl std::convert::TryInto<Entity> for &EntityDto {
      type Error = std::num::ParseIntError;
      fn try_into(self) -> Result<Entity, std::num::ParseIntError> {
          Ok(Entity {
              some_int: self.some_int,
              str: self.str.clone(),
              val: self.val.parse::<i16>()?,
          })
      }
  }
  ```
</details>

### Nested structs

``` rust
use o2o::o2o;

struct Entity {
    some_int: i32,
    child: Child,
}
struct Child {
    child_int: i32,
}

#[derive(o2o)]
#[from_owned(Entity)]
struct EntityDto {
    some_int: i32,
    #[map(~.into())]
    child: ChildDto
}

#[derive(o2o)]
#[from_owned(Child)]
struct ChildDto {
    child_int: i32,
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<Entity> for EntityDto {
      fn from(value: Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int,
              child: value.child.into(),
          }
      }
  }
  
  impl std::convert::From<Child> for ChildDto {
      fn from(value: Child) -> ChildDto {
          ChildDto {
              child_int: value.child_int,
          }
      }
  }
  ```
</details>

### Nested collection

``` rust
use o2o::o2o;

struct Entity {
    some_int: i32,
    children: Vec<Child>,
}
struct Child {
    child_int: i32,
}

#[derive(o2o)]
#[map_owned(Entity)]
struct EntityDto {
    some_int: i32,
    #[map(children, ~.iter().map(|p|p.into()).collect())]
    children_vec: Vec<ChildDto>
}

#[derive(o2o)]
#[map_ref(Child)]
struct ChildDto {
    child_int: i32,
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<Entity> for EntityDto {
      fn from(value: Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int,
              children_vec: value.children.iter().map(|p| p.into()).collect(),
          }
      }
  }
  impl std::convert::Into<Entity> for EntityDto {
      fn into(self) -> Entity {
          Entity {
              some_int: self.some_int,
              children: self.children_vec.iter().map(|p| p.into()).collect(),
          }
      }
  }
  impl std::convert::From<&Child> for ChildDto {
      fn from(value: &Child) -> ChildDto {
          ChildDto {
              child_int: value.child_int,
          }
      }
  }
  impl std::convert::Into<Child> for &ChildDto {
      fn into(self) -> Child {
          Child {
              child_int: self.child_int,
          }
      }
  }
  ```
</details>

### Assymetric fields (skipping and providing default values)

**o2o** is able to handle scenarios when either of the structs has a field that the other struct doesn't have.

For the scenario where you put **o2o** instructions on a struct that contains extra field:
``` rust
use o2o::o2o;

struct Person {
    id: i32,
    full_name: String,
    age: i8,
}

#[derive(o2o)]
#[map_owned(Person)]
struct PersonDto {
    id: i32,
    full_name: String,
    age: i8,
    // {None} below provides default value when creating PersonDto from Person
    // It could have been omited if we only needed to create Person from PersonDto
    #[ghost({None})]
    zodiac_sign: Option<ZodiacSign>
}
enum ZodiacSign {}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<Person> for PersonDto {
      fn from(value: Person) -> PersonDto {
          PersonDto {
              id: value.id,
              full_name: value.full_name,
              age: value.age,
              zodiac_sign: None,
          }
      }
  }
  impl std::convert::Into<Person> for PersonDto {
      fn into(self) -> Person {
          Person {
              id: self.id,
              full_name: self.full_name,
              age: self.age,
          }
      }
  }
  ```
</details>

In a reverse case, you need to use a struct level `#[ghosts()]` instruction:
``` rust
use o2o::o2o;

#[derive(o2o)]
#[map_owned(PersonDto)]
#[ghosts(zodiac_sign: {None})]
struct Person {
    id: i32,
    full_name: String,
    age: i8,
}

struct PersonDto {
    id: i32,
    full_name: String,
    age: i8,
    zodiac_sign: Option<ZodiacSign>
}
enum ZodiacSign {}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<PersonDto> for Person {
      fn from(value: PersonDto) -> Person {
          Person {
              id: value.id,
              full_name: value.full_name,
              age: value.age,
          }
      }
  }
  impl std::convert::Into<PersonDto> for Person {
      fn into(self) -> PersonDto {
          PersonDto {
              id: self.id,
              full_name: self.full_name,
              age: self.age,
              zodiac_sign: None,
          }
      }
  }
  ```
</details>

### Use struct update syntax (..Default::default())

``` rust
use o2o::o2o;

#[derive(Default)]
struct Entity {
    some_int: i32,
    some_float: f32
}

#[derive(Default, o2o)]
#[from(Entity| ..get_default())]
#[into(Entity| ..Default::default())]
struct EntityDto {
    some_int: i32,
    #[ghost]
    some_string: String
}

fn get_default() -> EntityDto {
    EntityDto { some_int: 0, some_string: "test".into() }
}
```

<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<&Entity> for EntityDto {
      fn from(value: &Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int,
              ..get_default()
          }
      }
  }
  impl std::convert::Into<Entity> for EntityDto {
      fn into(self) -> Entity {
          Entity {
              some_int: self.some_int,
              ..Default::default()
          }
      }
  }
  ```
</details>

### Define helper variables

``` rust
use o2o::o2o;

struct Person {
    age: i8,
    first_name: String,
    last_name: String
}

#[derive(o2o)]
#[from_owned(Person| vars(first_name: {@.first_name}, last_name: {@.last_name}))]
#[owned_into(Person| vars(first: {"John"}, last: {"Doe"}))]
#[ghosts(first_name: {first.into()}, last_name: {last.into()})]
struct PersonDto {
    age: i8,
    #[ghost({format!("{} {}", first_name, last_name)})]
    full_name: String
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<Person> for PersonDto {
      fn from(value: Person) -> PersonDto {
          let first_name = value.first_name;
          let last_name = value.last_name;
          PersonDto {
              age: value.age,
              full_name: format!("{} {}", first_name, last_name),
          }
      }
  }
  impl std::convert::Into<Person> for PersonDto {
      fn into(self) -> Person {
          let first = "John";
          let last = "Doe";
          Person {
              age: self.age,
              first_name: first.into(),
              last_name: last.into(),
          }
      }
  }
  ```
</details>

### Quick return

**o2o** allows you to bypass most of the logic by specifying quick return inline expression following `return`:

``` rust
use o2o::o2o;

#[derive(o2o)]
#[owned_into(String| return @.0.to_string())]
#[try_from_owned(String, std::num::ParseIntError)]
struct Wrapper(#[from(@.parse::<i32>()?)]i32);
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::TryFrom<String> for Wrapper {
      type Error = std::num::ParseIntError;
      fn try_from(value: String) -> Result<Wrapper, std::num::ParseIntError> {
          Ok(Wrapper(value.parse::<i32>()?))
      }
  }
  impl std::convert::Into<String> for Wrapper {
      fn into(self) -> String {
          self.0.to_string()
      }
  }
  ```
</details>

Quick returns work well with helper variables:

``` rust
use o2o::o2o;

#[derive(o2o)]
#[owned_into(i32| vars(hrs: {@.hours as i32}, mns: {@.minutes as i32}, scs: {@.seconds as i32}), 
    return hrs * 3600 + mns * 60 + scs)]
struct Time {
    hours: i8,
    minutes: i8,
    seconds: i8,
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::Into<i32> for Time {
      fn into(self) -> i32 {
          let hrs = self.hours as i32;
          let mns = self.minutes as i32;
          let scs = self.seconds as i32;
          hrs * 3600 + mns * 60 + scs
      }
  }
  ```
</details>

### Slightly complex example

``` rust
use o2o::o2o;

struct Employee {
    id: i32,
    first_name: String,
    last_name: String,
    subordinate_of: Box<Employee>,
    subordinates: Vec<Box<Employee>>
}
impl Employee {
    fn get_full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[derive(o2o)]
#[map(Employee)]
#[ghosts(
    first_name: {@.get_first_name()},
    last_name: {@.get_last_name()}
)]
struct EmployeeDto {
    #[map(id)]
    employee_id: i32,
    #[ghost(@.get_full_name())]
    full_name: String,

    #[from(Box::new(@.subordinate_of.as_ref().into()))]
    #[into(subordinate_of, Box::new(@.reports_to.as_ref().into()))]
    reports_to: Box<EmployeeDto>,

    #[map(~.iter().map(|p| Box::new(p.as_ref().into())).collect())]
    subordinates: Vec<Box<EmployeeDto>>
}
impl EmployeeDto {
    fn get_first_name(&self) -> String {
        self.full_name.split_whitespace().collect::<Vec<&str>>()[0].into()
    }
    fn get_last_name(&self) -> String {
        self.full_name.split_whitespace().collect::<Vec<&str>>()[1].into()
    }
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<Employee> for EmployeeDto {
      fn from(value: Employee) -> EmployeeDto {
          EmployeeDto {
              employee_id: value.id,
              full_name: value.get_full_name(),
              reports_to: (|x: &Employee| Box::new(x.subordinate_of.as_ref().into()))(&value),
              subordinates: value.subordinates.iter().map(|p| Box::new(p.as_ref().into())).collect(),
          }
      }
  }
  impl std::convert::From<&Employee> for EmployeeDto {
      fn from(value: &Employee) -> EmployeeDto {
          EmployeeDto {
              employee_id: value.id,
              full_name: value.get_full_name(),
              reports_to: (|x: &Employee| Box::new(x.subordinate_of.as_ref().into()))(value),
              subordinates: value.subordinates.iter().map(|p| Box::new(p.as_ref().into())).collect(),
          }
      }
  }
  impl std::convert::Into<Employee> for EmployeeDto {
      fn into(self) -> Employee {
          Employee {
              id: self.employee_id,
              subordinate_of: (|x: &EmployeeDto| Box::new(x.reports_to.as_ref().into()))(&self),
              subordinates: self.subordinates.iter().map(|p| Box::new(p.as_ref().into())).collect(),
              first_name: (|x: &EmployeeDto| x.get_first_name())(&self),
              last_name: self.get_last_name(),
          }
      }
  }
  impl std::convert::Into<Employee> for &EmployeeDto {
      fn into(self) -> Employee {
          Employee {
              id: self.employee_id,
              subordinate_of: (|x: &EmployeeDto| Box::new(x.reports_to.as_ref().into()))(self),
              subordinates: self.subordinates.iter().map(|p| Box::new(p.as_ref().into())).collect(),
              first_name: (|x: &EmployeeDto| x.get_first_name())(self),
              last_name: self.get_last_name(),
          }
      }
  }
  ```
</details>

### Flatened children

When the instructions are put on the side that contains flatened properties, conversion `From<T>` and `IntoExisting<T>` only requires usage of a member level `#[child(...)]` instruction, which accepts a path to the unflatened field (*without* the field name itself).
``` rust
use o2o::o2o;

struct Car {
    number_of_doors: i8,
    vehicle: Vehicle
}
struct Vehicle {
    number_of_seats: i16,
    machine: Machine,
}
struct Machine {
    brand: String,
    year: i16
}

#[derive(o2o)]
#[from_owned(Car)]
#[ref_into_existing(Car)]
struct CarDto {
    number_of_doors: i8,

    #[child(vehicle)]
    number_of_seats: i16,

    #[child(vehicle.machine)]
    #[map_ref(~.clone())]
    brand: String,

    #[child(vehicle.machine)]
    year: i16
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<Car> for CarDto {
      fn from(value: Car) -> CarDto {
          CarDto {
              number_of_doors: value.number_of_doors,
              number_of_seats: value.vehicle.number_of_seats,
              brand: value.vehicle.machine.brand,
              year: value.vehicle.machine.year,
          }
      }
  }
  impl o2o::traits::IntoExisting<Car> for &CarDto {
      fn into_existing(self, other: &mut Car) {
          other.number_of_doors = self.number_of_doors;
          other.vehicle.number_of_seats = self.number_of_seats;
          other.vehicle.machine.brand = self.brand.clone();
          other.vehicle.machine.year = self.year;
      }
  }
  ```
</details>

When you need an `Into<T>` conversion, **o2o** also expects you to provide types for flatened properties via struct level `#[children(...)]` instruction:

``` rust
use o2o::o2o;

struct Car {
    number_of_doors: i8,
    vehicle: Vehicle
}
struct Vehicle {
    number_of_seats: i16,
    machine: Machine,
}
struct Machine {
    brand: String,
    year: i16
}

#[derive(o2o)]
#[owned_into(Car)]
#[children(vehicle: Vehicle, vehicle.machine: Machine)]
struct CarDto {
    number_of_doors: i8,

    #[child(vehicle)]
    number_of_seats: i16,

    #[child(vehicle.machine)]
    brand: String,

    #[child(vehicle.machine)]
    year: i16
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::Into<Car> for CarDto {
      fn into(self) -> Car {
          Car {
              number_of_doors: self.number_of_doors,
              vehicle: Vehicle {
                  number_of_seats: self.number_of_seats,
                  machine: Machine {
                      brand: self.brand,
                      year: self.year,
                  },
              },
          }
      }
  }
  ```
</details>

The reverse case, where you have to put **o2o** insturctions on the side that has field that are being flatened, is slightly tricky:

``` rust
use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(o2o)]
#[owned_into(CarDto)]
struct Car {
    number_of_doors: i8,
    #[parent]
    vehicle: Vehicle
}

#[derive(o2o)]
#[owned_into_existing(CarDto)]
struct Vehicle {
    number_of_seats: i16,
    #[parent]
    machine: Machine,
}

#[derive(o2o)]
#[owned_into_existing(CarDto)]
struct Machine {
    brand: String,
    year: i16
}

// CarDto has to implement `Default` trait in this case.
#[derive(Default)]
struct CarDto {
    number_of_doors: i8,
    number_of_seats: i16,
    brand: String,
    year: i16
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::Into<CarDto> for Car {
      fn into(self) -> CarDto {
          let mut obj: CarDto = Default::default();
          obj.number_of_doors = self.number_of_doors;
          self.vehicle.into_existing(&mut obj);
          obj
      }
  }
  impl o2o::traits::IntoExisting<CarDto> for Vehicle {
      fn into_existing(self, other: &mut CarDto) {
          other.number_of_seats = self.number_of_seats;
          self.machine.into_existing(other);
      }
  }
  impl o2o::traits::IntoExisting<CarDto> for Machine {
      fn into_existing(self, other: &mut CarDto) {
          other.brand = self.brand;
          other.year = self.year;
      }
  }
  ```
</details>

### Tuple structs

``` rust
use o2o::o2o;

struct TupleEntity(i32, String);

#[derive(o2o)]
#[map_ref(TupleEntity)]
struct TupleEntityDto(i32, #[map_ref(~.clone())] String);
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<&TupleEntity> for TupleEntityDto {
      fn from(value: &TupleEntity) -> TupleEntityDto {
          TupleEntityDto(value.0, value.1.clone())
      }
  }
  impl std::convert::Into<TupleEntity> for &TupleEntityDto {
      fn into(self) -> TupleEntity {
          TupleEntity(self.0, self.1.clone())
      }
  }
  ```
</details>

As long as Rust allows following syntax, easy conversion between tuple and named structs can be done if placing **o2o** instructions on named side:

``` rust
use o2o::o2o;

struct TupleEntity(i32, String);

#[derive(o2o)]
#[map_ref(TupleEntity)]
struct EntityDto {
    #[map_ref(0)]
    some_int: i32, 
    #[map_ref(1, ~.clone())]
    some_str: String
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<&TupleEntity> for EntityDto {
      fn from(value: &TupleEntity) -> EntityDto {
          EntityDto {
              some_int: value.0,
              some_str: value.1.clone(),
          }
      }
  }
  impl std::convert::Into<TupleEntity> for &EntityDto {
      fn into(self) -> TupleEntity {
          TupleEntity {
              0: self.some_int,
              1: self.some_str.clone(),
          }
      }
  }
  ```
</details>

### Tuples

``` rust
use o2o::o2o;

#[derive(o2o)]
#[map_ref((i32, String))]
pub struct Entity{
    #[map(0)]
    int: i32,
    #[map(1, ~.clone())]
    string: String,
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<&(i32, String)> for Entity {
      fn from(value: &(i32, String)) -> Entity {
          Entity {
              int: value.0,
              string: value.1.clone(),
          }
      }
  }
  impl std::convert::Into<(i32, String)> for &Entity {
      fn into(self) -> (i32, String) {
          (self.int, self.string.clone())
      }
  }
  ```
</details>

### Type hints

By default, **o2o** will suppose that the struct on the other side is the same kind of type that the original one is. In order to convert between named and tuple structs when you need to place instructions on a tuple side, you`ll need to use Type Hint:

``` rust
use o2o::o2o;

#[derive(o2o)]
#[map_owned(EntityDto as {})]
struct TupleEntity(#[map(some_int)] i32, #[map(some_str)] String);

struct EntityDto{
    some_int: i32, 
    some_str: String
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<EntityDto> for TupleEntity {
      fn from(value: EntityDto) -> TupleEntity {
          TupleEntity(value.some_int, value.some_str)
      }
  }
  impl std::convert::Into<EntityDto> for TupleEntity {
      fn into(self) -> EntityDto {
          EntityDto {
              some_int: self.0,
              some_str: self.1,
          }
      }
  }
  ```
</details>

### Generics

``` rust
use o2o::o2o;

struct Entity<T> {
    some_int: i32,
    something: T,
}

#[derive(o2o)]
#[map_owned(Entity::<f32>)]
struct EntityDto {
    some_int: i32,
    something: f32
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<Entity<f32>> for EntityDto {
      fn from(value: Entity<f32>) -> EntityDto {
          EntityDto {
              some_int: value.some_int,
              something: value.something,
          }
      }
  }
  impl std::convert::Into<Entity<f32>> for EntityDto {
      fn into(self) -> Entity<f32> {
          Entity::<f32> {
              some_int: self.some_int,
              something: self.something,
          }
      }
  }
  ```
</details>

### Where clauses

``` rust
use o2o::o2o;

struct Child<T> {
    child_int: i32,
    something: T,
}

#[derive(o2o)]
#[map_owned(Child::<T>)]
#[where_clause(T: Clone)]
struct ChildDto<T> {
    child_int: i32,
    #[map(something, ~.clone())]
    stuff: T,
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl<T> std::convert::From<Child<T>> for ChildDto<T> where T: Clone, {
      fn from(value: Child<T>) -> ChildDto<T> {
          ChildDto {
              child_int: value.child_int,
              stuff: value.something.clone(),
          }
      }
  }
  impl<T> std::convert::Into<Child<T>> for ChildDto<T> where T: Clone, {
      fn into(self) -> Child<T> {
          Child::<T> {
              child_int: self.child_int,
              something: self.stuff.clone(),
          }
      }
  }
  ```
</details>

### Mapping to multiple structs

``` rust
use o2o::o2o;

struct Person {
    full_name: String,
    age: i32,
    country: String,
}

struct PersonModel {
    full_name: String,
    age: i32,
    place_of_birth: String,
}

#[derive(o2o)]
#[ref_into(Person)]
#[ref_into(PersonModel)]
struct PersonDto {
    // 'Default' member level instruction applies to all types
    #[into(full_name, ~.clone())]
    name: String,
    age: i32,
    // 'Dedicated' member level instruction applies to a specific type only
    #[into(Person| country, ~.clone())]
    #[into(PersonModel| ~.clone())]
    place_of_birth: String,
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::Into<Person> for &PersonDto {
      fn into(self) -> Person {
          Person {
              full_name: self.name.clone(),
              age: self.age,
              country: self.place_of_birth.clone(),
          }
      }
  }
  impl std::convert::Into<PersonModel> for &PersonDto {
      fn into(self) -> PersonModel {
          PersonModel {
              full_name: self.name.clone(),
              age: self.age,
              place_of_birth: self.place_of_birth.clone(),
          }
      }
  }
  ```
</details>

### Avoiding proc macro attribute name collisions (alternative instruction syntax)

**o2o** proc macro declares a lot of attributes, some of which have pretty broad meaning (e.g. from, into, map, child, parent etc.), so if you have to use it with some other proc macro, there is a chance that these attributes can collide and it would not be clear to what proc macro they should apply.
For this scenario, **o2o** supports two alternative syntaxes (syntacies?):

Below, all three variants of **o2o** proc macro application will produce the same generated code:
``` rust
use o2o::o2o;

struct Entity {
    some_int: i32,
    val: i16,
    str: String
}
// =====================================================================
#[derive(o2o)]
#[from(Entity)]
#[try_into(Entity, std::num::ParseIntError)]
struct EntityDto1 {
    some_int: i32,
    #[from(~.to_string())]
    #[into(~.parse::<i16>()?)]
    val: String,
    #[map_ref(~.clone())] 
    str: String
}
// =====================================================================
#[derive(o2o)]
#[o2o(from(Entity))]
#[o2o(try_into(Entity, std::num::ParseIntError))]
struct EntityDto2 {
    some_int: i32,
    #[o2o(from(~.to_string()))]
    #[o2o(into(~.parse::<i16>()?))]
    val: String,
    #[o2o(map_ref(~.clone()))] 
    str: String
}
// =====================================================================
#[derive(o2o)]
#[o2o(
    from(Entity),
    try_into(Entity, std::num::ParseIntError)
)]
struct EntityDto3 {
    some_int: i32,
    #[o2o(
        from(~.to_string()),
        try_into(~.parse::<i16>()?),
    )]
    val: String,
    #[o2o(map_ref(~.clone()))] 
    str: String
}
// =====================================================================
```

This syntax applies to all supported struct and member level instructions.

### Additional o2o instruction available via `#[o2o(...)]` syntax

#### Primitive type conversions

``` rust
use o2o::o2o;

struct Entity {
    some_int: i32,
    some_float: f32
}

#[derive(o2o)]
#[o2o(map_ref(Entity))]
struct EntityDto {
    #[o2o(as_type(i32))]
    some_int: i16,
    #[o2o(as_type(some_float, f32))]
    another_int: i16
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<&Entity> for EntityDto {
      fn from(value: &Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int as i16,
              another_int: value.some_float as i16,
          }
      }
  }
  impl std::convert::Into<Entity> for &EntityDto {
      fn into(self) -> Entity {
          Entity {
              some_int: self.some_int as i32,
              some_float: self.another_int as f32,
          }
      }
  }
  ```
</details>

This will work with all types that support 'as' conversion.

#### Repeat instructions

``` rust
use o2o::o2o;

struct Car {
    number_of_doors: i8,
    vehicle: Vehicle
}
struct Vehicle {
    number_of_seats: i16,
    can_fly: bool,
    needs_driver: bool,
    horsepower: i32,
    top_speed: f32,
    machine: Machine,
}
struct Machine {
    id: i32,
    brand: String,
    year: i16,
    weight: f32,
    length: f32,
    width: f32,
    height: f32,
}

#[derive(o2o)]
#[map_ref(Car)]
#[children(vehicle: Vehicle, vehicle.machine: Machine)]
#[ghosts(vehicle.machine@id: {321})]
struct CarDto {
    number_of_doors: i8,

    // #[o2o(repeat)] will repeat all instructions for this member to the following members, 
    // until there is a #[o2o(stop_repeat)] or the members run out.
    #[o2o(repeat)] #[child(vehicle)]
    number_of_seats: i16,
    can_fly: bool,
    needs_driver: bool,
    horsepower: i32,
    top_speed: f32,
    #[o2o(stop_repeat)]

    // You can also specify what specific types of instructions to repeat
    // (supported values are 'map', 'child', 'parent', 'ghost')
    #[o2o(repeat(child))] #[child(vehicle.machine)]
    #[map(~.clone())]
    brand: String,
    year: i16,
    weight: f32,
    length: f32,
    width: f32,
    height: f32,
    #[o2o(stop_repeat)]

    #[o2o(repeat)] #[ghost({123})]
    useless_param: i32,
    useless_param_2: i32,
    useless_param_3: i32,
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<&Car> for CarDto {
      fn from(value: &Car) -> CarDto {
          CarDto {
              number_of_doors: value.number_of_doors,
              number_of_seats: value.vehicle.number_of_seats,
              can_fly: value.vehicle.can_fly,
              needs_driver: value.vehicle.needs_driver,
              horsepower: value.vehicle.horsepower,
              top_speed: value.vehicle.top_speed,
              brand: value.vehicle.machine.brand.clone(),
              year: value.vehicle.machine.year,
              weight: value.vehicle.machine.weight,
              length: value.vehicle.machine.length,
              width: value.vehicle.machine.width,
              height: value.vehicle.machine.height,
              useless_param: 123,
              useless_param_2: 123,
              useless_param_3: 123,
          }
      }
  }
  impl std::convert::Into<Car> for &CarDto {
      fn into(self) -> Car {
          Car {
              number_of_doors: self.number_of_doors,
              vehicle: Vehicle {
                  number_of_seats: self.number_of_seats,
                  can_fly: self.can_fly,
                  needs_driver: self.needs_driver,
                  horsepower: self.horsepower,
                  top_speed: self.top_speed,
                  machine: Machine {
                      brand: self.brand.clone(),
                      year: self.year,
                      weight: self.weight,
                      length: self.length,
                      width: self.width,
                      height: self.height,
                      id: 321,
                  },
              },
          }
      }
  }
  ```
</details>

## Enum Examples

### Different variant name

``` rust
pub enum Sort {
    ASC,
    DESC,
    None
}

#[derive(o2o::o2o)]
#[map_owned(Sort)]
pub enum SortDto {
    #[map(ASC)] Ascending,
    #[map(DESC)] Descending,
    None
}
```

<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<Sort> for SortDto {
      fn from(value: Sort) -> SortDto {
          match value {
              Sort::ASC => SortDto::Ascending,
              Sort::DESC => SortDto::Descending,
              Sort::None => SortDto::None,
          }
      }
  }
  impl std::convert::Into<Sort> for SortDto {
      fn into(self) -> Sort {
          match self {
              SortDto::Ascending => Sort::ASC,
              SortDto::Descending => Sort::DESC,
              SortDto::None => Sort::None,
          }
      }
  }
  ```
</details>

### Mapping to primitive types

#### Using literals

Literals can be used to produce both `From` and `Into` implementations:

```rust
#[derive(o2o::o2o)]
#[map_owned(i32| _ => panic!("Not supported"))]
enum HttpStatus {
    #[literal(200)]Ok,
    #[literal(201)]Created,
    #[literal(401)]Unauthorized,
    #[literal(403)]Forbidden,
    #[literal(404)]NotFound,
    #[literal(500)]InternalError
}

type StaticStr = &'static str;

#[derive(o2o::o2o)]
#[map_owned(StaticStr| _ => todo!())]
enum Animal {
    #[literal("🐶")] Dog,
    #[literal("🐱")] Cat,
    #[literal("🐵")] Monkey
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<i32> for HttpStatus {
      fn from(value: i32) -> HttpStatus {
          match value {
              200 => HttpStatus::Ok,
              201 => HttpStatus::Created,
              401 => HttpStatus::Unauthorized,
              403 => HttpStatus::Forbidden,
              404 => HttpStatus::NotFound,
              500 => HttpStatus::InternalError,
              _ => panic!("Not supported"),
          }
      }
  }
  impl std::convert::Into<i32> for HttpStatus {
      fn into(self) -> i32 {
          match self {
              HttpStatus::Ok => 200,
              HttpStatus::Created => 201,
              HttpStatus::Unauthorized => 401,
              HttpStatus::Forbidden => 403,
              HttpStatus::NotFound => 404,
              HttpStatus::InternalError => 500,
          }
      }
  }

  impl std::convert::From<StaticStr> for Animal {
      fn from(value: StaticStr) -> Animal {
          match value {
              "🐶" => Animal::Dog,
              "🐱" => Animal::Cat,
              "🐵" => Animal::Monkey,
              _ => todo!(),
          }
      }
  }
  impl std::convert::Into<StaticStr> for Animal {
      fn into(self) -> StaticStr {
          match self {
              Animal::Dog => "🐶",
              Animal::Cat => "🐱",
              Animal::Monkey => "🐵",
          }
      }
  }
  ```
</details>

#### Using patterns

Patterns are only used to produce `From` implementations:

```rust
#[derive(o2o::o2o)]
#[from_owned(i32| _ => panic!())]
enum HttpStatusFamily {
    #[pattern(100..=199)] Information,
    #[pattern(200..=299)] Success,
    #[pattern(300..=399)] Redirection,
    #[pattern(400..=499)] ClientError,
    #[pattern(500..=599)] ServerError,
}

type StaticStr = &'static str;

#[derive(o2o::o2o)]
#[from_owned(StaticStr| _ => todo!())]
enum AnimalKind {
    #[pattern("🐶" | "🐱" | "🐵")]
    Mammal,

    #[pattern("🐟")] 
    Fish,
    
    #[pattern("🐛" | "🐜")]
    Insect
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<i32> for HttpStatusFamily {
      fn from(value: i32) -> HttpStatusFamily {
          match value {
              100..=199 => HttpStatusFamily::Information,
              200..=299 => HttpStatusFamily::Success,
              300..=399 => HttpStatusFamily::Redirection,
              400..=499 => HttpStatusFamily::ClientError,
              500..=599 => HttpStatusFamily::ServerError,
              _ => panic!(),
          }
      }
  }

  impl std::convert::From<StaticStr> for AnimalKind {
      fn from(value: StaticStr) -> AnimalKind {
          match value {
              "🐶" | "🐱" | "🐵" => AnimalKind::Mammal,
              "🐟" => AnimalKind::Fish,
              "🐛" | "🐜" => AnimalKind::Insect,
              _ => todo!(),
          }
      }
  }
  ```
</details>

#### Using literals and patterns together

```rust
#[derive(o2o::o2o)]
#[map_owned(i32)]
enum HttpStatus {
    #[literal(200)] Ok,
    #[literal(404)] NotFound,
    #[literal(500)] InternalError,
    #[pattern(_)] #[into({f0})] Other(#[from(@)] i32)
}

type StaticStr = &'static str;

#[derive(o2o::o2o)]
#[map_owned(StaticStr)]
enum Animal {
    #[literal("🐶")] Dog,
    #[literal("🐱")] Cat,
    #[literal("🐵")] Monkey,
    #[pattern(_)] #[into({name})] Other{ #[from(@)] name: StaticStr }
}
```
<details>
  <summary>View generated code</summary>

  ``` rust ignore
  impl std::convert::From<i32> for HttpStatus {
      fn from(value: i32) -> HttpStatus {
          match value {
              200 => HttpStatus::Ok,
              404 => HttpStatus::NotFound,
              500 => HttpStatus::InternalError,
              _ => HttpStatus::Other(value),
          }
      }
  }
  impl std::convert::Into<i32> for HttpStatus {
      fn into(self) -> i32 {
          match self {
              HttpStatus::Ok => 200,
              HttpStatus::NotFound => 404,
              HttpStatus::InternalError => 500,
              HttpStatus::Other(f0) => f0,
          }
      }
  }

  impl std::convert::From<StaticStr> for Animal {
      fn from(value: StaticStr) -> Animal {
          match value {
              "🐶" => Animal::Dog,
              "🐱" => Animal::Cat,
              "🐵" => Animal::Monkey,
              _ => Animal::Other { name: value },
          }
      }
  }
  impl std::convert::Into<StaticStr> for Animal {
      fn into(self) -> StaticStr {
          match self {
              Animal::Dog => "🐶",
              Animal::Cat => "🐱",
              Animal::Monkey => "🐵",
              Animal::Other { name } => name,
          }
      }
  }
  ```
</details>

## Contributions

All issues, questions, pull requests are extremely welcome.

## License

<sup>
Licensed under either an <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>