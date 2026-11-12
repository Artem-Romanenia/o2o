use o2o::o2o;
use o2o::traits::FromExisting;

struct EntityDto {
    some_int: i32,
    another_int: i16,
    name: String,
}

#[derive(o2o)]
#[from(EntityDto)]
#[from_existing(EntityDto)]
struct Entity {
    #[ghost({0})]
    id: i64,
    some_int: i32,
    #[from(another_int)]
    different_int: i16,
    #[from_ref(~.clone())]
    name: String,
    #[ghost({false})]
    dirty: bool,
}

#[derive(o2o)]
#[owned_from_existing(EntityDto)]
struct OwnedEntity {
    #[from(~ * 2)]
    some_int: i32,
    #[from(another_int, ~ as i32)]
    different_int: i32,
    name: String,
    #[ghost]
    version: u32,
}

#[derive(o2o)]
#[ref_from_existing(EntityDto)]
struct RefEntity {
    #[from(@.some_int + @.another_int as i32)]
    total: i32,
    #[from(~.len())]
    name: usize,
    #[ghost]
    version: u32,
}

struct ValueDto<T> {
    value: T,
}

#[derive(o2o)]
#[owned_from_existing(ValueDto<T>)]
struct Value<T> {
    value: T,
    #[ghost]
    revision: u32,
}

fn dto() -> EntityDto {
    EntityDto { some_int: 123, another_int: 321, name: String::from("Test") }
}

#[test]
fn named2named_owned() {
    let mut entity = Entity {
        id: 42,
        some_int: 1,
        different_int: 2,
        name: String::from("Old"),
        dirty: true,
    };

    entity.from_existing(dto());

    assert_eq!(42, entity.id);
    assert_eq!(123, entity.some_int);
    assert_eq!(321, entity.different_int);
    assert_eq!("Test", entity.name);
    assert!(entity.dirty);
}

#[test]
fn named2named_ref() {
    let dto = dto();
    let mut entity = Entity {
        id: 42,
        some_int: 1,
        different_int: 2,
        name: String::from("Old"),
        dirty: true,
    };

    entity.from_existing(&dto);

    assert_eq!(42, entity.id);
    assert_eq!(123, entity.some_int);
    assert_eq!(321, entity.different_int);
    assert_eq!("Test", entity.name);
    assert!(entity.dirty);
}

#[test]
fn from_and_from_existing_side_by_side() {
    let entity: Entity = dto().into();

    assert_eq!(0, entity.id);
    assert_eq!(123, entity.some_int);
    assert_eq!(321, entity.different_int);
    assert_eq!("Test", entity.name);
    assert!(!entity.dirty);
}

#[test]
fn owned_only() {
    let mut entity = OwnedEntity { some_int: 1, different_int: 2, name: String::from("Old"), version: 7 };

    entity.from_existing(dto());

    assert_eq!(246, entity.some_int);
    assert_eq!(321, entity.different_int);
    assert_eq!("Test", entity.name);
    assert_eq!(7, entity.version);
}

#[test]
fn ref_only() {
    let dto = dto();
    let mut entity = RefEntity { total: 1, name: 2, version: 7 };

    entity.from_existing(&dto);

    assert_eq!(444, entity.total);
    assert_eq!(4, entity.name);
    assert_eq!(7, entity.version);
}

#[test]
fn generic() {
    let mut value = Value { value: String::from("Old"), revision: 3 };

    value.from_existing(ValueDto { value: String::from("New") });

    assert_eq!("New", value.value);
    assert_eq!(3, value.revision);
}
