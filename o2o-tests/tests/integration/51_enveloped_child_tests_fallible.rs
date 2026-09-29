use o2o::o2o;

struct Entity {
    some_int: i8,
    child: Option<Child>,
}
#[derive(Clone)]
struct Child {
    child_int: i16,
    base: Result<Base, i32>,
}
#[derive(Clone)]
struct Base {
    some_str: String,
    base_int: i16,
}

impl Base  {
    fn as_ok(self) -> Result<Base, i32>{
        Result::Ok(self)
    }
}

fn custom_unwrap(s: Result<Base, i32>) -> Base {
    s.unwrap()
}

struct TupleEntity(i8, Option<TupleChild>);
#[derive(Clone)]
struct TupleChild(i16, Result<TupleBase, i32>);
#[derive(Clone)]
struct TupleBase(String, i16);

impl TupleBase  {
    fn as_ok(self) -> Result<TupleBase, i32>{
        Result::Ok(self)
    }
}

fn custom_unwrap_tuple(s: Result<TupleBase, i32>) -> TupleBase {
    s.unwrap()
}

#[derive(o2o)]
#[try_map(Entity, anyhow::Error)]
#[try_map(TupleEntity as (), anyhow::Error)]
#[child_parents(Entity|
    child: Child => (into: Some(@), from: ~.clone().unwrap()), 
    child.base: Base => (into: @.as_ok(), from: custom_unwrap(~))
)]
#[child_parents(TupleEntity|
    1: TupleChild as () => (into: Some(@), from: ~.clone().unwrap()), 
    1 .1: TupleBase as () => (into: @.as_ok(), from: custom_unwrap_tuple(~))
)]
struct EntityDto {
    some_int: i8,

    #[child(Entity| child)]
    #[child(TupleEntity| 1)]
    #[map(TupleEntity| 0)]
    child_int: i16,

    #[child(Entity| child.base)]
    #[map(Entity| ~.clone())]
    #[child(TupleEntity| 1 .1)]
    #[map(TupleEntity| 0, ~.clone())]
    some_str: String,

    #[child(Entity| child.base)]
    #[child(TupleEntity| 1 .1)]
    #[map(TupleEntity| 1)]
    base_int: i16,
}

#[derive(o2o)]
#[try_map(Entity as {}, anyhow::Error)]
#[try_map(TupleEntity, anyhow::Error)]
#[child_parents(Entity|
    child: Child as {} => (into: Some(@), from: ~.clone().unwrap()), 
    child.base: Base as {} => (into: @.as_ok(), from: custom_unwrap(~))
)]
#[child_parents(TupleEntity|
    1: TupleChild => (into: Some(@), from: ~.clone().unwrap()), 
    1 .1: TupleBase => (into: @.as_ok(), from: custom_unwrap_tuple(~))
)]
struct EntityTupleDto(
    #[map(Entity| some_int)]
    i8,

    #[child(Entity| child)]
    #[map(Entity| child_int)]
    #[child(TupleEntity| 1)]
    #[map(TupleEntity| 0)]
    i16,

    #[child(Entity| child.base)]
    #[map(Entity| some_str, ~.clone())]
    #[child(TupleEntity| 1 .1)]
    #[map(TupleEntity| 0, ~.clone())]
    String,

    #[child(Entity| child.base)]
    #[map(Entity| base_int)]
    #[child(TupleEntity| 1 .1)]
    #[map(TupleEntity| 1)]
    i16,
);

#[test]
fn named2named() {
    let entity_dto = EntityDto {
        some_int: 123,
        child_int: 321,
        some_str: "str".into(),
        base_int: 456,
    };

    let entity: Entity = entity_dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);

    let child = entity.child.unwrap();
    assert_eq!(321, child.child_int);

    let base = child.base.unwrap();
    assert_eq!("str", base.some_str);
    assert_eq!(456, base.base_int);
}

#[test]
fn named2named_reverse() {
    let entity = Entity {
        some_int: 123,
        child: Some(Child {
            child_int: 321,
            base: Base { some_str: "str".into(), base_int: 456 }.as_ok(),
        }),
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.child_int);
    assert_eq!("str", dto.some_str);
    assert_eq!(456, dto.base_int);
}

#[test]
fn named2named_ref() {
    let entity_dto = &EntityDto {
        some_int: 123,
        child_int: 321,
        some_str: "str".into(),
        base_int: 456,
    };

    let entity: Entity = entity_dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);

    let child = entity.child.unwrap();
    assert_eq!(321, child.child_int);

    let base = child.base.unwrap();
    assert_eq!("str", base.some_str);
    assert_eq!(456, base.base_int);
}

#[test]
fn named2named_reverse_ref() {
    let entity = &Entity {
        some_int: 123,
        child: Some(Child {
            child_int: 321,
            base: Base { some_str: "str".into(), base_int: 456 }.as_ok(),
        }),
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.child_int);
    assert_eq!("str", dto.some_str);
    assert_eq!(456, dto.base_int);
}

#[test]
fn named2tuple() {
    let entity_dto = EntityDto {
        some_int: 123,
        child_int: 321,
        some_str: "str".into(),
        base_int: 456,
    };

    let entity: TupleEntity = entity_dto.try_into().unwrap();

    assert_eq!(123, entity.0);

    let child = entity.1.unwrap();
    assert_eq!(321, child.0);

    let base = child.1.unwrap();
    assert_eq!("str", base.0);
    assert_eq!(456, base.1);
}

#[test]
fn named2tuple_reverse() {
    let entity = TupleEntity(123, Some(TupleChild(321, TupleBase("str".into(), 456).as_ok())));

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.child_int);
    assert_eq!("str", dto.some_str);
    assert_eq!(456, dto.base_int);
}

#[test]
fn named2tuple_ref() {
    let entity_dto = &EntityDto {
        some_int: 123,
        child_int: 321,
        some_str: "str".into(),
        base_int: 456,
    };

    let entity: TupleEntity = entity_dto.try_into().unwrap();

    assert_eq!(123, entity.0);

    let child = entity.1.unwrap();
    assert_eq!(321, child.0);

    let base = child.1.unwrap();
    assert_eq!("str", base.0);
    assert_eq!(456, base.1);
}

#[test]
fn named2tuple_reverse_ref() {
    let entity = &TupleEntity(123, Some(TupleChild(321, TupleBase("str".into(), 456).as_ok())));

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.child_int);
    assert_eq!("str", dto.some_str);
    assert_eq!(456, dto.base_int);
}

#[test]
fn tuple2named() {
    let entity_dto = EntityTupleDto(123, 321, "str".into(), 456);

    let entity: Entity = entity_dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);

    let child = entity.child.unwrap();
    assert_eq!(321, child.child_int);

    let base = child.base.unwrap();
    assert_eq!("str", base.some_str);
    assert_eq!(456, base.base_int);
}

#[test]
fn tuple2named_reverse() {
    let entity = Entity {
        some_int: 123,
        child: Some(Child {
            child_int: 321,
            base: Base { some_str: "str".into(), base_int: 456 }.as_ok(),
        }),
    };

    let dto: EntityTupleDto = entity.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!("str", dto.2);
    assert_eq!(456, dto.3);
}

#[test]
fn tuple2named_ref() {
    let entity_dto = &EntityTupleDto(123, 321, "str".into(), 456);

    let entity: Entity = entity_dto.try_into().unwrap();

    assert_eq!(123, entity.some_int);

    let child = entity.child.unwrap();
    assert_eq!(321, child.child_int);

    let base = child.base.unwrap();
    assert_eq!("str", base.some_str);
    assert_eq!(456, base.base_int);
}

#[test]
fn tuple2named_reverse_ref() {
    let entity = &Entity {
        some_int: 123,
        child: Some(Child {
            child_int: 321,
            base: Base { some_str: "str".into(), base_int: 456 }.as_ok(),
        }),
    };

    let dto: EntityTupleDto = entity.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!("str", dto.2);
    assert_eq!(456, dto.3);
}

#[test]
fn tuple2tuple() {
    let entity_dto = EntityTupleDto(123, 321, "str".into(), 456);

    let entity: TupleEntity = entity_dto.try_into().unwrap();

    assert_eq!(123, entity.0);

    let child = entity.1.unwrap();
    assert_eq!(321, child.0);

    let base = child.1.unwrap();
    assert_eq!("str", base.0);
    assert_eq!(456, base.1);
}

#[test]
fn tuple2tuple_reverse() {
    let entity = TupleEntity(123, Some(TupleChild(321, TupleBase("str".into(), 456).as_ok())));

    let dto: EntityTupleDto = entity.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!("str", dto.2);
    assert_eq!(456, dto.3);
}

#[test]
fn tuple2tuple_ref() {
    let entity_dto = &EntityTupleDto(123, 321, "str".into(), 456);

    let entity: TupleEntity = entity_dto.try_into().unwrap();

    assert_eq!(123, entity.0);

    let child = entity.1.unwrap();
    assert_eq!(321, child.0);

    let base = child.1.unwrap();
    assert_eq!("str", base.0);
    assert_eq!(456, base.1);
}

#[test]
fn tuple2tuple_reverse_ref() {
    let entity = &TupleEntity(123, Some(TupleChild(321, TupleBase("str".into(), 456).as_ok())));

    let dto: EntityTupleDto = entity.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!("str", dto.2);
    assert_eq!(456, dto.3);
}
