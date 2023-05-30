Object to Object mapper for Rust<!-- omit from toc --> 
================================
[<img alt="github.com" src="https://github.com/Artem-Romanenia/o2o/workflows/Build/badge.svg" height="25">](https://github.com/Artem-Romanenia/o2o/)
[<img alt="crates.io" src="https://img.shields.io/crates/v/o2o.svg?style=for-the-badge&color=2f4d28&labelColor=f9f7ec&logo=rust&logoColor=black" height="25">](https://crates.io/crates/o2o)

## Content<!-- omit from toc --> 

- [Quick pitch](#quick-pitch)
- [Brief description](#brief-description)
- [Examples](#examples)
  - [Simplest Case](#simplest-case)
  - [Different field name](#different-field-name)
  - [Different field type](#different-field-type)
  - [Nested structs](#nested-structs)
  - [Nested collection](#nested-collection)
  - [Assymetric fields (skipping and providing default values)](#assymetric-fields-skipping-and-providing-default-values)
- [Expressions](#expressions)
  - [Expressions for struct level instructions](#expressions-for-struct-level-instructions)
  - [Expressions for member level instructions](#expressions-for-member-level-instructions)
- [More examples](#more-examples)
  - [Slightly complex example](#slightly-complex-example)
  - [Flatened children](#flatened-children)
  - [Tuple structs](#tuple-structs)
  - [Struct kind hints](#struct-kind-hints)
  - [Generics](#generics)
  - [Where clauses](#where-clauses)
  - [Mapping to multiple structs](#mapping-to-multiple-structs)
  - [Avoiding proc macro attribute name collisions (alternative instruction syntax)](#avoiding-proc-macro-attribute-name-collisions-alternative-instruction-syntax)
  - [Additional o2o instruction available via `#[o2o(...)]` syntax](#additional-o2o-instruction-available-via-o2o-syntax)
    - [Primitive type conversions](#primitive-type-conversions)
    - [Repeat instructions](#repeat-instructions)
    - [Wrapper structs and `#[o2o(wrapper)]` instruction](#wrapper-structs-and-o2owrapper-instruction)
  - [Contributions](#contributions)
    - [License](#license)

## Quick pitch

```rust
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

Writing code like above is not the most exciting or rewarding part of working with Rust. If you're Ok with letting procedural macro write it for you, welcome to the rest of this page.

## Brief description

**o2o** procedural macro is able to generate implementation of 6 kinds of traits (currently for structs only):

``` rust
// #[from_owned()]
impl std::convert::From<A> for B { ... }

// #[from_ref()]
impl std::convert::From<&A> for B { ... }

// #[owned_into()]
impl std::convert::Into<A> for B { ... }

// #[ref_into()]
impl std::convert::Into<A> for &B { ... }

// #[owned_into_existing()]
impl o2o::traits::IntoExisting<A> for B { ... }

// #[ref_into_existing()]
impl o2o::traits::IntoExisting<A> for &B { ... }
```

It also has shortcuts to configure multiple trait implementations with fewer lines of code:

|                              | #[map()] | #[from()]  | #[into()] | #[map_owned()] | #[map_ref()] | #[into_existing()] |
| ---------------------------- | -------- | ---------- | --------- | ---------------| ------------ | -------------------|
| **#[from_owned()]**          | ✔️       | ✔️          | ❌        | ✔️             | ❌           | ❌                |
| **#[from_ref()]**            | ✔️       | ✔️          | ❌        | ❌            | ✔️            | ❌                |
| **#[owned_into()]**          | ✔️       | ❌         | ✔️         | ✔️             | ❌           | ❌                |
| **#[ref_into()]**            | ✔️       | ❌         | ✔️         | ❌            | ✔️            | ❌                |
| **#[owned_into_existing()]** | ❌      | ❌         | ❌        | ❌            | ❌           | ✔️                 |
| **#[ref_into_existing()]**   | ❌      | ❌         | ❌        | ❌            | ❌           | ✔️                 |

With that, let's look at some examples.

## Examples

### Simplest Case

``` rust
use o2o::o2o;

struct Person {
    id: u32,
    name: String,
    age: u8
}

#[derive(o2o)]
#[map_owned(Person)]
struct PersonDto {
    id: u32,
    name: String,
    age: u8
}
```
From here on, generated code is produced by [rust-analyzer: Expand macro recursively](https://rust-analyzer.github.io/manual.html#expand-macro-recursively) command
<details>
  <summary>View generated code</summary>

  ``` rust
  impl std::convert::From<Person> for PersonDto {
      fn from(value: Person) -> PersonDto {
          PersonDto {
              id: value.id,
              name: value.name,
              age: value.age,
          }
      }
  }
  impl std::convert::Into<Person> for PersonDto {
      fn into(self) -> Person {
          Person {
              id: self.id,
              name: self.name,
              age: self.age,
          }
      }
  }
  ```
</details>

With the above code you should be able to do this:

``` rust
let person = Person { id: 123, name: "John".into(), age: 42 };
let dto: PersonDto = person.into();
// and this:
let dto = PersonDto { id: 123, name: "John".into(), age: 42 };
let person: Person = dto.into();
```

### Different field name

``` rust
struct Entity {
    some_int: i32,
    another_int: i16,
}

#[derive(o2o)]
#[from_ref(Entity)]
#[ref_into_existing(Entity)]
struct EntityDto {
    some_int: i32,
    #[map(another_int)]
    different_int: i16,
}
```
<details>
  <summary>View generated code</summary>

  ``` rust
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
  ```
</details>

### Different field type

``` rust
struct Entity {
    some_int: i32,
    val: i16,
    str: String
}

#[derive(o2o)]
#[map(Entity)]
struct EntityDto {
    some_int: i32,
    #[from(~.to_string())] // Tilde allows to append code at the end of the right side of field initialization for From<T> impls
    #[into(~.parse::<i16>().unwrap())] // here it's the same but for Into<T> impls
    val: String,
    // Here Into and From are symmetric, so it has to be only specified once.
    // Note that .clone() is only needed for borrowing impls, so we use #[map_ref()]
    #[map_ref(~.clone())] 
    str: String
}
```
<details>
  <summary>View generated code</summary>

  ``` rust
  impl std::convert::From<Entity> for EntityDto {
      fn from(value: Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int,
              val: value.val.to_string(),
              str: value.str, // no .clone()
          }
      }
  }
  impl std::convert::From<&Entity> for EntityDto {
      fn from(value: &Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int,
              val: value.val.to_string(),
              str: value.str.clone(),
          }
      }
  }
  impl std::convert::Into<Entity> for EntityDto {
      fn into(self) -> Entity {
          Entity {
              some_int: self.some_int,
              val: self.val.parse::<i16>().unwrap(),
              str: self.str, // no .clone()
          }
      }
  }
  impl std::convert::Into<Entity> for &EntityDto {
      fn into(self) -> Entity {
          Entity {
              some_int: self.some_int,
              val: self.val.parse::<i16>().unwrap(),
              str: self.str.clone(),
          }
      }
  }
  ```
</details>

### Nested structs

``` rust
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

  ``` rust
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
    // Here field name as well as type are different, so we pass in field name and tilde inline expression.
    // Also, it doesn't hurt to use member level instruction #[map()], 
    // which is broader than struct level instruction #[map_owned]
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

  ``` rust
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
    // (|| None) below provides default value when creating PersonDto from Person
    // It could have been omited if we only needed to create Person from PersonDto
    #[ghost(|| None)]
    zodiac_sign: Option<ZodiacSign>
}
enum ZodiacSign {}
```
<details>
  <summary>View generated code</summary>

  ``` rust
  impl std::convert::From<Person> for PersonDto {
      fn from(value: Person) -> PersonDto {
          PersonDto {
              id: value.id,
              full_name: value.full_name,
              age: value.age,
              zodiac_sign: (|| None)(),
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

In a reverse case, you need to use a struct level `#[ghost()]` instruction:
``` rust
#[derive(o2o)]
#[map_owned(PersonDto)]
#[ghost(zodiac_sign: || { None })] // #[ghost()] struct level instruction accepts only braced closures.
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

  ``` rust
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
              zodiac_sign: (|| None)(),
          }
      }
  }
  ```
</details>

## Expressions

### Expressions for struct level instructions

```rust
#[ghost(field: { None })]

// field: None,

#[ghost(field: || { None })]

// field: (|| None)(),

#[ghost(field: { @.get_value() })]

// field: self.get_value(),

#[ghost(field: |x| { x.get_value() })]

// field: (|x: &Type| x.get_value())(&self),
```

### Expressions for member level instructions

```rust
#[map({ None })]

// field: None,

#[map(~.clone())]

// field: value.field.clone(),

#[map(@.get_value())]

// field: value.get_value(),

#[map(|x| x.get_value())]

// field: (|x: &Type| x.get_value())(&value),
```

## More examples

### Slightly complex example

``` rust
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
#[ghost(
    // o2o supports closures with one input parameter.
    // This parameter represents instance on the other side of the conversion.
    first_name: |x| {x.get_first_name()},
    last_name: |x| {x.get_last_name()}
)]
struct EmployeeDto {
    #[map(id)]
    employee_id: i32,
    // '@.' is another flavor of 'inline expression'. 
    // @ also represents instance on the other side of the conversion.
    #[ghost(@.get_full_name())]
    full_name: String,

    #[from(|x| Box::new(x.subordinate_of.as_ref().into()))]
    #[into(subordinate_of, |x| Box::new(x.reports_to.as_ref().into()))]
    reports_to: Box<EmployeeDto>,

    #[map(~.iter().map(|p|Box::new(p.as_ref().into())).collect())]
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

  ``` rust
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

  ``` rust
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

  ``` rust
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

  ``` rust
  impl std::convert::Into<CarDto> for Car {
      fn into(self) -> CarDto {
          let mut obj = CarDto {
              number_of_doors: self.number_of_doors,
              ..Default::default()
          };
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
struct TupleEntity(i32, String);

#[derive(o2o)]
#[map_ref(TupleEntity)]
struct TupleEntityDto(i32, #[map_ref(~.clone())] String);
```
<details>
  <summary>View generated code</summary>

  ``` rust
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

  ``` rust
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

### Struct kind hints

By default, **o2o** will suppose that the struct on the other side is the same kind of struct that the original one is. In order to convert between named and tuple structs when you need to place instructions on a tuple side, you`ll need to use Struct Kind Hint:

``` rust
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

  ``` rust
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

  ``` rust
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

  ``` rust
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

  ``` rust
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
struct Entity {
    some_int: i32,
    val: i16,
    str: String
}
// =====================================================================
#[derive(o2o)]
#[map(Entity)]
struct EntityDto {
    some_int: i32,
    #[from(~.to_string())]
    #[into(~.parse::<i16>().unwrap())]
    val: String,
    #[map_ref(~.clone())] 
    str: String
}
// =====================================================================
#[derive(o2o)]
#[o2o(map(Entity))]
struct EntityDto {
    some_int: i32,
    #[o2o(from(~.to_string()))]
    #[o2o(into(~.parse::<i16>().unwrap()))]
    val: String,
    #[o2o(map_ref(~.clone()))] 
    str: String
}
// =====================================================================
#[derive(o2o)]
#[o2o(map(Entity))]
struct EntityDto {
    some_int: i32,
    #[o2o(
        from(~.to_string()),
        into(~.parse::<i16>().unwrap()),
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

  ``` rust
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

```rust
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
#[ghost(vehicle.machine@id: || { 321 })]
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

    #[o2o(repeat)] #[ghost(|| {123})]
    useless_param: i32,
    useless_param_2: i32,
    useless_param_3: i32,
}
```
<details>
  <summary>View generated code</summary>

  ``` rust
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
              useless_param: (|| 123)(),
              useless_param_2: (|| 123)(),
              useless_param_3: (|| 123)(),
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
                      id: (|| 321)(),
                  },
              },
          }
      }
  }
  ```
</details>

#### Wrapper structs and `#[o2o(wrapper)]` instruction

```rust
#[derive(o2o)]
#[map_owned(Vec<u8>)]
struct PayloadWrapper {
    #[o2o(wrapper)]
    payload: Vec<u8>,
}
```
<details>
  <summary>View generated code</summary>

  ``` rust
  impl std::convert::From<Vec<u8>> for PayloadWrapper {
      fn from(value: Vec<u8>) -> PayloadWrapper {
          PayloadWrapper { payload: value }
      }
  }
  impl std::convert::Into<Vec<u8>> for PayloadWrapper {
      fn into(self) -> Vec<u8> {
          self.payload
      }
  }
  ```
</details>

If you need to 'ref' implementation, you may need to provide an inline expression:

```rust
#[derive(o2o)]
#[map(String)]
struct StringWrapper {
    #[o2o(wrapper(~.clone()))]
    str: String
}
```
<details>
  <summary>View generated code</summary>

  ``` rust
  impl std::convert::From<String> for StringWrapper {
      fn from(value: String) -> StringWrapper {
          StringWrapper { str: value }
      }
  }
  impl std::convert::From<&String> for StringWrapper {
      fn from(value: &String) -> StringWrapper {
          StringWrapper { str: value.clone() }
      }
  }
  impl std::convert::Into<String> for StringWrapper {
      fn into(self) -> String {
          self.str
      }
  }
  impl std::convert::Into<String> for &StringWrapper {
      fn into(self) -> String {
          self.str.clone()
      }
  }
  ```
</details>

### Contributions

All issues, questions, pull requests  are extremely welcome.

#### License

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