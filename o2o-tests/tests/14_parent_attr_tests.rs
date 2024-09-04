use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(o2o)]
#[map(EntityDto)]
#[into_existing(EntityDto)]
struct Entity {
    parent_int: i32,
    #[parent]
    base: BaseEntity,
    #[parent]
    child: Child,
}

#[derive(o2o)]
#[from(EntityDto)]
#[into_existing(EntityDto)]
struct BaseEntity {
    #[o2o(parent)]
    base: Base,
    base_entity_int: i32,
}

#[derive(o2o)]
#[from(EntityDto)]
#[into_existing(EntityDto)]
struct Base {
    #[map(base_int)]
    base_int_2: i32,
    another_base_int: i32,
}

#[derive(o2o)]
#[from(EntityDto)]
#[into_existing(EntityDto)]
struct Child {
    child_int: i32,
    another_child_int: i32,
}

#[derive(Default)]
struct EntityDto {
    pub parent_int: i32,
    pub base_int: i32,
    pub another_base_int: i32,
    pub base_entity_int: i32,
    pub child_int: i32,
    pub another_child_int: i32,
}

#[derive(o2o)]
#[map(TupleEntityDto)]
#[into_existing(TupleEntityDto)]
struct TupleEntity(i32, #[parent] TupleBaseEntity, #[o2o(parent)] TupleChild);

#[derive(o2o)]
#[from(TupleEntityDto)]
#[into_existing(TupleEntityDto)]
struct TupleBaseEntity(#[parent] TupleBase, #[map(3)] i32);

#[derive(o2o)]
#[from(TupleEntityDto)]
#[into_existing(TupleEntityDto)]
struct TupleBase(#[map(1)] i32, #[map(2)] i16);

#[derive(o2o)]
#[from(TupleEntityDto)]
#[into_existing(TupleEntityDto)]
struct TupleChild(#[map(4)] i32, #[map(5)] i16);

#[derive(Default)]
struct TupleEntityDto(i32, i32, i16, i32, i32, i16);

#[test]
fn named2named() {
    let dto = EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        base_entity_int: 654,
        child_int: 789,
        another_child_int: 987,
    };

    let entity: Entity = dto.into();

    assert_eq!(123, entity.parent_int);
    assert_eq!(321, entity.base.base.base_int_2);
    assert_eq!(456, entity.base.base.another_base_int);
    assert_eq!(654, entity.base.base_entity_int);
    assert_eq!(789, entity.child.child_int);
    assert_eq!(987, entity.child.another_child_int);
}

#[test]
fn named2named_reverse() {
    let entity = Entity {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
}

#[test]
fn named2named_ref() {
    let dto = &EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        base_entity_int: 654,
        child_int: 789,
        another_child_int: 987,
    };

    let entity: Entity = dto.into();

    assert_eq!(dto.parent_int, entity.parent_int);
    assert_eq!(dto.base_int, entity.base.base.base_int_2);
    assert_eq!(dto.another_base_int, entity.base.base.another_base_int);
    assert_eq!(dto.base_entity_int, entity.base.base_entity_int);
    assert_eq!(dto.child_int, entity.child.child_int);
    assert_eq!(dto.another_child_int, entity.child.another_child_int);
}

#[test]
fn named2named_reverse_ref() {
    let entity = &Entity {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let dto: EntityDto = entity.into();

    assert_eq!(entity.parent_int, dto.parent_int);
    assert_eq!(entity.base.base.base_int_2, dto.base_int);
    assert_eq!(entity.base.base.another_base_int, dto.another_base_int);
    assert_eq!(entity.base.base_entity_int, dto.base_entity_int);
    assert_eq!(entity.child.child_int, dto.child_int);
    assert_eq!(entity.child.another_child_int, dto.another_child_int);
}

#[test]
fn unnamed2unnamed() {
    let dto = TupleEntityDto(123, 321, 456, 654, 789, 987);

    let entity: TupleEntity = dto.into();

    assert_eq!(123, entity.0);
    assert_eq!(321, entity.1 .0 .0);
    assert_eq!(456, entity.1 .0 .1);
    assert_eq!(654, entity.1 .1);
    assert_eq!(789, entity.2 .0);
    assert_eq!(987, entity.2 .1);
}

#[test]
fn unnamed2unnamed_reverse() {
    let entity = TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let dto: TupleEntityDto = entity.into();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!(987, dto.5);
}

#[test]
fn unnamed2unnamed_ref() {
    let dto = &TupleEntityDto(123, 321, 456, 654, 789, 987);

    let entity: TupleEntity = dto.into();

    assert_eq!(dto.0, entity.0);
    assert_eq!(dto.1, entity.1 .0 .0);
    assert_eq!(dto.2, entity.1 .0 .1);
    assert_eq!(dto.3, entity.1 .1);
    assert_eq!(dto.4, entity.2 .0);
    assert_eq!(dto.5, entity.2 .1);
}

#[test]
fn unnamed2unnamed_reverse_ref() {
    let entity = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let dto: TupleEntityDto = entity.into();

    assert_eq!(entity.0, dto.0);
    assert_eq!(entity.1 .0 .0, dto.1);
    assert_eq!(entity.1 .0 .1, dto.2);
    assert_eq!(entity.1 .1, dto.3);
    assert_eq!(entity.2 .0, dto.4);
    assert_eq!(entity.2 .1, dto.5);
}

#[test]
fn existing_named2named() {
    let entity = Entity {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let mut dto: EntityDto = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
}

#[test]
fn existing_named2named_ref() {
    let entity = &Entity {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let mut dto: EntityDto = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(entity.parent_int, dto.parent_int);
    assert_eq!(entity.base.base.base_int_2, dto.base_int);
    assert_eq!(entity.base.base.another_base_int, dto.another_base_int);
    assert_eq!(entity.base.base_entity_int, dto.base_entity_int);
    assert_eq!(entity.child.child_int, dto.child_int);
    assert_eq!(entity.child.another_child_int, dto.another_child_int);
}

#[test]
fn existing_unnamed2unnamed() {
    let entity = TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let mut dto: TupleEntityDto = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!(987, dto.5);
}

#[test]
fn existing_unnamed2unnamed_ref() {
    let entity = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let mut dto: TupleEntityDto = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(entity.0, dto.0);
    assert_eq!(entity.1 .0 .0, dto.1);
    assert_eq!(entity.1 .0 .1, dto.2);
    assert_eq!(entity.1 .1, dto.3);
    assert_eq!(entity.2 .0, dto.4);
    assert_eq!(entity.2 .1, dto.5);
}
