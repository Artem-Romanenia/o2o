## Object to Object mapper for Rust

**o2o** procedural macro is able to generate implementation of 6 kinds of traits:

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

### Examples

#### Simplest Case

``` rust
use o2o::o2o;

struct Entity {
    some_int: i32,
    another_int: i16,
}

#[derive(o2o)]
#[map_owned(Entity)]
struct EntityDto {
    some_int: i32,
    another_int: i16,
}
```
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std::convert::From<Entity> for EntityDto {
      fn from(value : Entity) -> EntityDto {
          EntityDto { 
              some_int : value.some_int, 
              another_int : value.another_int, 
          }
       }
  }

  impl std::convert::Into<Entity> for EntityDto {
      fn into(self) -> Entity {
          Entity {
              some_int : self.some_int,
              another_int : self.another_int,
          }
      }
  }
  ```
</details>

With the above code you should be able to do this:

``` rust
let entity = Entity { some_int: 123, another_int: 321 }
let dto: EntityDto = entity.into();
// and this:
let dto = EntityDto { some_int: 123, another_int: 321 }
let entity: Entity = dto.into();
```

#### Different field name

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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std::convert::From<&Entity> for EntityDto {
      fn from(value : & Entity) -> EntityDto {
          EntityDto { 
              some_int : value.some_int,
              different_int : value.another_int,
          }
      }
  }

  impl o2o::traits::IntoExisting<Entity> for &EntityDto {
      fn into_existing(self, other : &mut Entity) {
          other.some_int = self.some_int; 
          other.another_int = self.different_int;
      }
  }
  ```
</details>

#### Different field type

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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std :: convert :: From < Entity > for EntityDto {
      fn from(value : Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int,
              val: value.val.to_string(),
              str: value.str, // no .clone() needed
          }
      }
  }
  impl std :: convert :: From < & Entity > for EntityDto {
      fn from(value : & Entity) -> EntityDto {
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
              str: self.str, // no .clone() needed
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

#### Nested structs

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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std::convert::From<Entity> for EntityDto {
      fn from(value : Entity) -> EntityDto { 
          EntityDto { 
              some_int: value.some_int, 
              child: value.child.into(), 
          }
      }
  }
  
  impl std::convert::From<Child> for ChildDto {
      fn from(value : Child) -> ChildDto { 
          ChildDto { 
              child_int: value.child_int, 
          }
      }
  }
  ```
</details>

#### Nested collection

``` rust
struct Entity {
    some_int: i32,
    children: Vec<Child>,
}
struct Child {
    child_int: i32,
}

#[derive(o2o)]
#[map(Entity)]
struct EntityDto {
    some_int: i32,
    #[map(children.iter().map(|p|p.into()).collect())]
    children: Vec<ChildDto>
}

#[derive(o2o)]
#[map(Child)]
struct ChildDto {
    child_int: i32,
}
```

#### Composit example

``` rust
struct Employee {
    id: i32,
    full_name: String,
    subordinate_of: Box<Employee>,
    subordinates: Vec<Box<Employee>>
}

#[derive(o2o)]
#[map(Employee)]
struct EmployeeDto {
    #[map(id)]
    employee_id: i32,

    #[map(full_name.clone())]
    full_name: String,

    #[from(|x| Box::new(x.subordinate_of.as_ref().into()))]
    #[into(subordinate_of, |x| Box::new(x.reports_to.as_ref().into()))]
    reports_to: Box<EmployeeDto>,

    #[map(subordinates.iter().map(|p|Box::new(p.as_ref().into())).collect())]
    subordinates: Vec<Box<EmployeeDto>>
}
```

### Inline expressions and closures

So far you could have noticed a couple of different types of arguments that can be passed to member level **o2o** instructions:

``` rust
#[map(id)] //Perhaps member name?
#[from(|x| Box::new(x.subordinate_of.as_ref().into()))] //looks like closure??

//What's this weirdness??? (I call them Inline expressions)
#[map(full_name.clone())]
#[map(subordinates.iter().map(|p|Box::new(p.as_ref().into())).collect())]
```

To better understand how they work, take a look at the code from previous 'composite' example, followed by the code generated by **o2o**:

![Microservices overview](visual-explanation.png)

Notice that `#[map(...)]` member level **o2o** instructions are reflected in all four trait impls. `#[from(...)]` and `#[into(...)]` are only to be found in two respective implementations for `From<T>` and `Into<T>` traits.

### Mapping uneven objects

#### Uneven fields

**o2o** is able to handle scenarios when either of the structs has a field that the other struct doesn't have.

For the scenario where you put **o2o** instructions on a struct that contains extra field:
``` rust
struct Person {
    id: i32,
    full_name: String,
    age: i8,
}

#[derive(o2o)]
#[map(Person)]
struct PersonDto {
    id: i32,
    #[map(full_name.clone())]
    full_name: String,
    age: i8,
    // (|_| None) below provides default value when creating PersonDto from Person
    // It could have been omited if we only needed to create Person from PersonDto
    #[ghost(|_| None)]
    zodiac_sign: Option<ZodiacSign>
}
enum ZodiacSign {}
```

In a reverse case, you need to use a struct level `#[ghost()]` instruction:
``` rust
#[derive(o2o)]
#[map(PersonDto)]
#[ghost(zodiac_sign: |_| { None })]
struct Person {
    id: i32,
    #[map(full_name.clone())]
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

#### Uneven children

**o2o** is also able to handle scenarios where a child struct (or a hierarchy of structs) on one side is flattened on the other:
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
#[from(Car)]
#[into_existing(Car)]
struct CarDto {
    number_of_doors: i8,

    #[child(vehicle)]
    number_of_seats: i16,

    #[child(vehicle.machine)]
    #[map_ref(brand.clone())]
    brand: String,

    #[child(vehicle.machine)]
    year: i16
}
```

The reverse case, where you have to put **o2o** insturctions on the side that has less  information, is slightly tricky:
``` rust
use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(o2o)]
#[map(CarDto)]
struct Car {
    number_of_doors: i8,
    #[parent]
    vehicle: Vehicle
}

#[derive(o2o)]
#[from(CarDto)]
#[into_existing(CarDto)]
struct Vehicle {
    number_of_seats: i16,
    #[parent]
    machine: Machine,
}

#[derive(o2o)]
#[from(CarDto)]
#[into_existing(CarDto)]
struct Machine {
    #[map_ref(brand.clone())]
    brand: String,
    year: i16
}

#[derive(Default)]
struct CarDto {
    number_of_doors: i8,
    number_of_seats: i16,
    brand: String,
    year: i16
}
```

Notice that CarDto has to implement `Default` trait in this case.

### Tuple structs

to be documented...

### Struct kind hints

to be documented...

### Generics

to be documented...

#### Where clauses

to be documented...

### Mapping to multiple structs

to be documented...

### Avoiding proc macro attribute name collisions (alternative instruction syntax)

to be documented...

### #[panic_debug_info] instruction

to be documented...

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