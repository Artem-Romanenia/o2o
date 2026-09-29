use o2o::o2o;
use o2o::traits::TryIntoExisting;

#[derive(Default)]
struct UnnamedStructDto(i32, i32, f32);

#[derive(Default, o2o)]
// Map works here as long as Rust allows instantiation like TupleStruct { 0: ..., 1: ..., ...}
#[try_map(UnnamedStructDto, anyhow::Error)]
#[try_into_existing(UnnamedStructDto, anyhow::Error)]
struct NamedStruct {
    #[try_map(0)]
    some_int: i32,
    #[map(1)]
    another_int: i32,
    #[map(2)]
    some_float: f32,
}

#[derive(Default)]
struct NamedStructModel {
    some_int: i32,
    another_int: i8,
    some_float: f32,
}

#[derive(o2o)]
#[o2o(try_map(NamedStruct, anyhow::Error), try_map(NamedStructModel, anyhow::Error), try_into_existing(NamedStruct, anyhow::Error), try_into_existing(NamedStructModel, anyhow::Error))]
struct NamedStructDto {
    some_int: i32,
    #[o2o(
        try_from(NamedStruct| @.another_int as i16),
        try_into(NamedStruct| @.another_int as i32),
        from(NamedStructModel| @.another_int as i16),
        into(NamedStructModel| @.another_int as i8),
    )]
    another_int: i16,
    #[o2o(from(@.some_float as f64))]
    #[o2o(into(@.some_float as f32))]
    some_float: f64,
}

struct Parent {
    child: Child,
    parent_int: i32,
}

struct Child {
    child_int: i32,
    another_child_int: i32,
}

#[derive(o2o)]
#[try_map_owned(Parent, anyhow::Error)]
struct ParentDto {
    #[map_owned(@.child.try_into()?)]
    child: ChildDto,
    parent_int: i32,
}

#[derive(o2o)]
#[try_map(Child, anyhow::Error)]
struct ChildDto {
    child_int: i32,
    #[map(another_child_int)]
    diff_another_child_int: i32,
}

#[derive(o2o)]
#[try_from(Parent, anyhow::Error)]
struct ParentDto2 {
    parent_int: i32,
    #[try_from(@.child.child_int)]
    child_int: i32,
    #[from(@.child.another_child_int)]
    another_child_int: i32,
}

struct Person {
    name: String,
    pets: Vec<Pet>,
}

struct Pet {
    age: i8,
    nickname: String,
}

#[derive(o2o)]
#[try_map(Person, anyhow::Error)]
struct PersonDto {
    #[map(@.name.clone())]
    name: String,
    #[map(@.pets.iter().map(|p|p.try_into().unwrap()).collect())]
    pets: Vec<PetDto>,
}

#[derive(o2o)]
#[try_map(Pet, anyhow::Error)]
struct PetDto {
    age: i8,
    #[map(@.nickname.clone())]
    nickname: String,
}

#[test]
fn unnamed2named() {
    let dto = UnnamedStructDto(123, 321, 456.0);

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
    assert_eq!(456.0, named.some_float);
}

#[test]
fn named2unnamed() {
    let named = NamedStruct { some_int: 123, another_int: 321, some_float: 456.0 };

    let dto: UnnamedStructDto = named.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456.0, dto.2);
}

#[test]
fn unnamed2named_ref() {
    let dto = &UnnamedStructDto(123, 321, 456.0);

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(dto.0, named.some_int);
    assert_eq!(dto.1, named.another_int);
    assert_eq!(dto.2, named.some_float);
}

#[test]
fn named2unnamed_ref() {
    let named = &NamedStruct { some_int: 123, another_int: 321, some_float: 456.0 };

    let dto: UnnamedStructDto = named.try_into().unwrap();

    assert_eq!(named.some_int, dto.0);
    assert_eq!(named.another_int, dto.1);
    assert_eq!(named.some_float, dto.2);
}

#[test]
fn named2named_uneven() {
    let p = Parent {
        parent_int: 123,
        child: Child { child_int: 321, another_child_int: 456 },
    };

    let dto: ParentDto2 = p.try_into().unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.child_int);
    assert_eq!(456, dto.another_child_int);
}

#[test]
fn named2named_uneven_ref() {
    let parent = &Parent {
        parent_int: 123,
        child: Child { child_int: 321, another_child_int: 456 },
    };

    let dto: ParentDto2 = parent.try_into().unwrap();

    assert_eq!(parent.parent_int, dto.parent_int);
    assert_eq!(parent.child.child_int, dto.child_int);
    assert_eq!(parent.child.another_child_int, dto.another_child_int);
}

#[test]
fn named2named_different_types() {
    let dto = NamedStructDto { some_int: 123, another_int: 321, some_float: 456.0 };

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
    assert_eq!(456.0, named.some_float);

    let dto = NamedStructDto { some_int: 123, another_int: 127, some_float: 456.0 };

    let model: NamedStructModel = dto.try_into().unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!(127, model.another_int);
    assert_eq!(456.0, model.some_float);
}

#[test]
fn named2named_different_types_reverse() {
    let named = NamedStruct { some_int: 123, another_int: 321, some_float: 456.0 };

    let dto: NamedStructDto = named.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.another_int);
    assert_eq!(456.0, dto.some_float);

    let model = NamedStructModel { some_int: 123, another_int: 127, some_float: 456.0 };

    let dto: NamedStructDto = model.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(127, dto.another_int);
    assert_eq!(456.0, dto.some_float);
}

#[test]
fn named2named_different_types_ref() {
    let dto = &NamedStructDto { some_int: 123, another_int: 127, some_float: 456.0 };

    let named: NamedStruct = dto.try_into().unwrap();

    assert_eq!(dto.some_int, named.some_int);
    assert_eq!(dto.another_int, named.another_int as i16);
    assert_eq!(dto.some_float, named.some_float as f64);

    let model: NamedStructModel = dto.try_into().unwrap();

    assert_eq!(dto.some_int, model.some_int);
    assert_eq!(dto.another_int, model.another_int as i16);
    assert_eq!(dto.some_float, model.some_float as f64);
}

#[test]
fn named2named_different_types_reverse_ref() {
    let named = &NamedStruct { some_int: 123, another_int: 321, some_float: 456.0 };

    let dto: NamedStructDto = named.try_into().unwrap();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.another_int as i32);
    assert_eq!(named.some_float, dto.some_float as f32);

    let model = &NamedStructModel { some_int: 123, another_int: 127, some_float: 456.0 };

    let dto: NamedStructDto = model.try_into().unwrap();

    assert_eq!(model.some_int, dto.some_int);
    assert_eq!(model.another_int, dto.another_int as i8);
    assert_eq!(model.some_float, dto.some_float as f32);
}

#[test]
fn named2named_child() {
    let p = Parent {
        parent_int: 123,
        child: Child { child_int: 321, another_child_int: 456 },
    };

    let dto: ParentDto = p.try_into().unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.child.child_int);
    assert_eq!(456, dto.child.diff_another_child_int);
}

#[test]
fn named2named_child_reverse() {
    let dto = ParentDto {
        parent_int: 123,
        child: ChildDto { child_int: 321, diff_another_child_int: 456 },
    };

    let parent: Parent = dto.try_into().unwrap();

    assert_eq!(123, parent.parent_int);
    assert_eq!(321, parent.child.child_int);
    assert_eq!(456, parent.child.another_child_int);
}

#[test]
fn named2named_children() {
    let dto = PersonDto {
        name: String::from("John"),
        pets: vec![
            PetDto { age: 5, nickname: String::from("Mr. Dog") },
            PetDto { age: 10, nickname: String::from("Mr. Cat") },
        ],
    };

    let p: Person = dto.try_into().unwrap();

    assert_eq!("John", p.name);
    assert_eq!(2, p.pets.len());
    assert_eq!(5, p.pets[0].age);
    assert_eq!("Mr. Dog", p.pets[0].nickname);
    assert_eq!(10, p.pets[1].age);
    assert_eq!("Mr. Cat", p.pets[1].nickname);
}

#[test]
fn named2named_children_reverse() {
    let p = Person {
        name: String::from("John"),
        pets: vec![
            Pet { age: 5, nickname: String::from("Mr. Dog") },
            Pet { age: 10, nickname: String::from("Mr. Cat") },
        ],
    };

    let dto: PersonDto = p.try_into().unwrap();

    assert_eq!("John", dto.name);
    assert_eq!(2, dto.pets.len());
    assert_eq!(5, dto.pets[0].age);
    assert_eq!("Mr. Dog", dto.pets[0].nickname);
    assert_eq!(10, dto.pets[1].age);
    assert_eq!("Mr. Cat", dto.pets[1].nickname);
}

#[test]
fn named2named_children_ref() {
    let dto = &PersonDto {
        name: String::from("John"),
        pets: vec![
            PetDto { age: 5, nickname: String::from("Mr. Dog") },
            PetDto { age: 10, nickname: String::from("Mr. Cat") },
        ],
    };

    let p: Person = dto.try_into().unwrap();

    assert_eq!(dto.name, p.name);
    assert_eq!(2, p.pets.len());
    assert_eq!(dto.pets[0].age, p.pets[0].age);
    assert_eq!(dto.pets[0].nickname, p.pets[0].nickname);
    assert_eq!(dto.pets[1].age, p.pets[1].age);
    assert_eq!(dto.pets[1].nickname, p.pets[1].nickname);
}

#[test]
fn named2named_children_ref_reversed() {
    let p = &Person {
        name: String::from("John"),
        pets: vec![
            Pet { age: 5, nickname: String::from("Mr. Dog") },
            Pet { age: 10, nickname: String::from("Mr. Cat") },
        ],
    };

    let dto: PersonDto = p.try_into().unwrap();

    assert_eq!(dto.name, p.name);
    assert_eq!(2, p.pets.len());
    assert_eq!(dto.pets[0].age, p.pets[0].age);
    assert_eq!(dto.pets[0].nickname, p.pets[0].nickname);
    assert_eq!(dto.pets[1].age, p.pets[1].age);
    assert_eq!(dto.pets[1].nickname, p.pets[1].nickname);
}

#[test]
fn existing_named2unnamed() {
    let named = NamedStruct { some_int: 123, another_int: 321, some_float: 456.0 };

    let mut dto: UnnamedStructDto = Default::default();
    named.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456.0, dto.2);
}

#[test]
fn existing_named2unnamed_ref() {
    let named = &NamedStruct { some_int: 123, another_int: 321, some_float: 456.0 };

    let mut dto: UnnamedStructDto = Default::default();
    named.try_into_existing(&mut dto).unwrap();

    assert_eq!(named.some_int, dto.0);
    assert_eq!(named.another_int, dto.1);
    assert_eq!(named.some_float, dto.2);
}

#[test]
fn existing_named2named_different_types() {
    let dto = NamedStructDto { some_int: 123, another_int: 321, some_float: 456.0 };

    let mut named: NamedStruct = Default::default();
    dto.try_into_existing(&mut named).unwrap();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
    assert_eq!(456.0, named.some_float);

    let dto = NamedStructDto { some_int: 123, another_int: 127, some_float: 456.0 };

    let mut model: NamedStructModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(123, model.some_int);
    assert_eq!(127, model.another_int);
    assert_eq!(456.0, model.some_float);
}

#[test]
fn existing_named2named_different_types_ref() {
    let dto = &NamedStructDto { some_int: 123, another_int: 127, some_float: 456.0 };

    let mut named: NamedStruct = Default::default();
    dto.try_into_existing(&mut named).unwrap();

    assert_eq!(dto.some_int, named.some_int);
    assert_eq!(dto.another_int, named.another_int as i16);
    assert_eq!(dto.some_float, named.some_float as f64);

    let mut model: NamedStructModel = Default::default();
    dto.try_into_existing(&mut model).unwrap();

    assert_eq!(dto.some_int, model.some_int);
    assert_eq!(dto.another_int, model.another_int as i16);
    assert_eq!(dto.some_float, model.some_float as f64);
}
