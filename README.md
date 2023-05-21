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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std::convert::From<Entity> for EntityDto {
      fn from(value : Entity) -> EntityDto {
          EntityDto {
              some_int: value.some_int, 
              children_vec: value.children.iter().map(|p|p.into()).collect(),
          }
      }
  } 
  impl std::convert::Into<Entity> for EntityDto {
      fn into(self) -> Entity {
          Entity {
              some_int: self.some_int, 
              children: self.children_vec.iter().map(| p | p.into()).collect(),
          }
      }
  }
  impl std::convert::From<&Child> for ChildDto {
      fn from(value : & Child) -> ChildDto { 
          ChildDto { child_int : value.child_int, } 
      }
  } 
  impl std::convert::Into<Child> for &ChildDto {
      fn into(self) -> Child { 
          Child { child_int : self.child_int, }
      }
  }
  ```
</details>

#### Assymetric fields (skipping and providing default values)

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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std::convert::From<Person> for PersonDto {
      fn from(value : Person) -> PersonDto {
          PersonDto {
              id: value.id, 
              full_name: value.full_name, 
              age: value.age,
              zodiac_sign: (|| None) (),
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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std::convert::From<PersonDto> for Person {
      fn from(value : PersonDto) -> Person {
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
              zodiac_sign: (| | { None }) (),
          }
      }
  }
  ```
</details>

#### Slightly complex example

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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std::convert::From<Employee> for EmployeeDto {
      fn from(value : Employee) -> EmployeeDto {
          EmployeeDto {
              employee_id: value.id,
              full_name: value.get_full_name(),
              reports_to: (|x: &Employee| Box::new(x.subordinate_of.as_ref().into()))(&value),
              subordinates: value.subordinates.iter().map(|p| Box::new(p.as_ref().into())).collect(),
          }
      }
  } 
  impl std::convert::From<&Employee> for EmployeeDto {
      fn from(value : & Employee) -> EmployeeDto {
          EmployeeDto {
              employee_id: value.id,
              full_name: value.get_full_name(),
              reports_to: (|x: &Employee | Box::new(x.subordinate_of.as_ref().into()))(value),
              subordinates: value.subordinates.iter().map(|p| Box::new(p.as_ref().into())).collect(),
          }
      }
  }
  impl std::convert::Into<Employee> for EmployeeDto {
      fn into(self) -> Employee {
          Employee {
              id : self.employee_id,
              subordinate_of: (|x: &EmployeeDto | Box::new(x.reports_to.as_ref().into()))(& self),
              subordinates: self.subordinates.iter().map(|p| Box::new(p.as_ref().into())).collect(), 
              first_name: (|x: &EmployeeDto | { x.get_first_name() })(&self),
              last_name: (|x: &EmployeeDto | { x.get_last_name() })(&self),
          }
      }
  }
  impl std::convert::Into<Employee> for &EmployeeDto {
      fn into(self) -> Employee {
          Employee {
              id : self.employee_id,
              subordinate_of: (|x : &EmployeeDto | Box::new(x.reports_to.as_ref().into()))(self),
              subordinates: self.subordinates.iter().map(|p| Box::new(p.as_ref().into())).collect(),
              first_name: (|x: &EmployeeDto | { x.get_first_name() })(self),
              last_name : (|x: &EmployeeDto | { x.get_last_name() })(self),
          }
      }
  }
  ```
</details>

#### Flatened children

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
#[into_existing(Car)]
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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std::convert::Into<CarDto> for Car {
      fn into(self) -> CarDto {
          let mut obj = CarDto {
              number_of_doors: self.number_of_doors, 
              .. Default::default() 
          };
          self.vehicle.into_existing(& mut obj) ; obj
      }
  }
  impl o2o :: traits :: IntoExisting < CarDto > for Vehicle {
      fn into_existing(self, other : & mut CarDto) {
          other.number_of_seats = self.number_of_seats;
          self.machine.into_existing(other);
      }
  }
  impl o2o :: traits :: IntoExisting < CarDto > for Machine {
      fn into_existing(self, other : & mut CarDto) {
          other.brand = self.brand;
          other.year = self.year;
      }
  }
  ```
</details>

#### Tuple structs

``` rust
struct TupleEntity(i32, String);

#[derive(o2o)]
#[map_ref(TupleEntity)]
struct TupleEntityDto(i32, #[map_ref(~.clone())] String);
```
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std :: convert :: From < & TupleEntity > for TupleEntityDto {
      fn from(value : & TupleEntity) -> TupleEntityDto {
          TupleEntityDto(value.0, value.1.clone(),)
      }
  }
  impl std :: convert :: Into < TupleEntity > for & TupleEntityDto {
      fn into(self) -> TupleEntity {
          TupleEntity(self.0, self.1.clone(),)
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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std::convert::From<&TupleEntity> for EntityDto {
      fn from(value: & TupleEntity) -> EntityDto {
          EntityDto { 
              some_int: value.0,
              some_str: value.1.clone(),
          }
      }
  }
  impl std::convert::Into<TupleEntity> for &EntityDto {
      fn into(self) -> TupleEntity {
          TupleEntity {
              0 : self.some_int,
              1: self.some_str.clone(),
          }
      }
  }
  ```
</details>

#### Struct kind hints

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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std :: convert :: From < EntityDto > for TupleEntity {
      fn from(value : EntityDto) -> TupleEntity {
          TupleEntity(value.some_int, value.some_str,)
      }
  }
  impl std :: convert :: Into < EntityDto > for TupleEntity {
      fn into(self) -> EntityDto { 
          EntityDto {
              some_int : self.0,
              some_str : self.1,
          }
      }
  }
  ```
</details>

#### Generics

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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl std::convert::From<Entity::<f32>> for EntityDto {
      fn from(value: Entity::<f32>) -> EntityDto {
          EntityDto {
              some_int: value.some_int, 
              something: value.something,
          }
      }
  }
  impl std::convert::Into<Entity::<f32>> for EntityDto {
      fn into(self) -> Entity::<f32> {
          Entity::<f32> {
              some_int: self.some_int,
              something: self.something,
          }
      }
  }
  ```
</details>

#### Where clauses

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
<details style="background-color: lightgray; border-radius: 6px">
  <summary style="font-size: 10px">View generated code</summary>

  ``` rust
  impl <T> std::convert::From<Child::<T>> for ChildDto<T> where T: Clone {
      fn from(value: Child::<T>) -> ChildDto<T> {
          ChildDto {
              child_int: value.child_int,
              stuff: value.something.clone(),
          }
      }
  }
  impl <T> std::convert::Into<Child::<T>> for ChildDto <T> where T: Clone {
      fn into(self) -> Child::<T> {
          Child::<T> {
              child_int: self.child_int,
              something: self.stuff.clone(),
          }
      }
  }
  ```
</details>

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