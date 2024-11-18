use o2o::traits::TryIntoExisting;

#[derive(o2o::o2o)]
#[try_map(EntityDto, String)]
#[try_into_existing(EntityDto, String)]
#[try_map(TupleEntityDto as (), String)]
#[try_into_existing(TupleEntityDto as (), String)]
struct Entity {
    parent_int: i32,
    #[parent(EntityDto| [parent([map(base_int)] base_int_2, another_base_int)] base: Base, base_entity_int)]
    #[parent(TupleEntityDto| [parent([map(1)] base_int_2, [map(2)]another_base_int)] base: Base, [map(3)] base_entity_int)]
    base: BaseEntity,
    #[parent(EntityDto| child_int, another_child_int)]
    #[parent(TupleEntityDto| [map(4)] child_int, [map(5)] another_child_int)]
    child: Child,
}

#[derive(o2o::o2o)]
#[try_map(TupleEntityDto, String)]
#[try_into_existing(TupleEntityDto, String)]
#[try_map(EntityDto as {}, String)]
#[try_into_existing(EntityDto as {}, String)]
struct TupleEntity(
    #[map(EntityDto| parent_int)] i32, 

    #[parent(TupleEntityDto| [parent([map(1)] 0, [map(2)] 1)] 0: TupleBase, [map(3)] 1)]
    #[parent(EntityDto| [parent([map(base_int)] 0, [map(another_base_int)] 1)] 0: TupleBase, [map(base_entity_int)] 1)]
    TupleBaseEntity,

    #[parent(TupleEntityDto| [map(4)]0, [map(5)] 1)]
    #[parent(EntityDto| [map(child_int)]0, [map(another_child_int)] 1)]
    TupleChild
);

struct BaseEntity {
    base: Base,
    base_entity_int: i32,
}

struct TupleBaseEntity(TupleBase, i32);

struct Base {
    base_int_2: i32,
    another_base_int: i16,
}

struct TupleBase(i32, i16);

struct Child {
    child_int: i32,
    another_child_int: i16,
}

struct TupleChild(i32, i16);

#[derive(Default)]
struct EntityDto {
    parent_int: i32,
    base_int: i32,
    another_base_int: i16,
    base_entity_int: i32,
    child_int: i32,
    another_child_int: i16,
}

#[derive(Default)]
struct TupleEntityDto(i32, i32, i16, i32,  i32, i16);

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

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.parent_int);
    assert_eq!(321, entity.base.base.base_int_2);
    assert_eq!(456, entity.base.base.another_base_int);
    assert_eq!(654, entity.base.base_entity_int);
    assert_eq!(789, entity.child.child_int);
    assert_eq!(987, entity.child.another_child_int);
}

#[test]
fn named2unnamed() {
    let dto = EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        base_entity_int: 654,
        child_int: 789,
        another_child_int: 987,
    };

    let entity: TupleEntity = dto.try_into().unwrap();

    assert_eq!(123, entity.0);
    assert_eq!(321, entity.1 .0 .0);
    assert_eq!(456, entity.1 .0 .1);
    assert_eq!(654, entity.1 .1);
    assert_eq!(789, entity.2 .0);
    assert_eq!(987, entity.2 .1);
}

#[test]
fn named2named_reverse() {
    let entity = Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
        },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
}

#[test]
fn named2unnamed_reverse() {
    let entity = Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base { base_int_2: 321, another_base_int: 456 },
            base_entity_int: 654,
        },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let dto: TupleEntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!(987, dto.5);
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

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(dto.parent_int, entity.parent_int);
    assert_eq!(dto.base_int, entity.base.base.base_int_2);
    assert_eq!(dto.another_base_int, entity.base.base.another_base_int);
    assert_eq!(dto.base_entity_int, entity.base.base_entity_int);
    assert_eq!(dto.child_int, entity.child.child_int);
    assert_eq!(dto.another_child_int, entity.child.another_child_int);
}

#[test]
fn named2unnamed_ref() {
    let dto = &EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        base_entity_int: 654,
        child_int: 789,
        another_child_int: 987,
    };

    let entity: TupleEntity = dto.try_into().unwrap();

    assert_eq!(dto.parent_int, entity.0);
    assert_eq!(dto.base_int, entity.1 .0 .0);
    assert_eq!(dto.another_base_int, entity.1 .0 .1);
    assert_eq!(dto.base_entity_int, entity.1 .1);
    assert_eq!(dto.child_int, entity.2 .0);
    assert_eq!(dto.another_child_int, entity.2 .1);
}

#[test]
fn named2named_reverse_ref() {
    let entity = &Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
        },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(entity.parent_int, dto.parent_int);
    assert_eq!(entity.base.base.base_int_2, dto.base_int);
    assert_eq!(entity.base.base.another_base_int, dto.another_base_int);
    assert_eq!(entity.base.base_entity_int, dto.base_entity_int);
    assert_eq!(entity.child.child_int, dto.child_int);
    assert_eq!(entity.child.another_child_int, dto.another_child_int);
}

#[test]
fn named2unnamed_reverse_ref() {
    let entity = &Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base { base_int_2: 321, another_base_int: 456 },
            base_entity_int: 654,
        },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let dto: TupleEntityDto = entity.try_into().unwrap();

    assert_eq!(entity.parent_int, dto.0);
    assert_eq!(entity.base.base.base_int_2, dto.1);
    assert_eq!(entity.base.base.another_base_int, dto.2);
    assert_eq!(entity.base.base_entity_int, dto.3);
    assert_eq!(entity.child.child_int, dto.4);
    assert_eq!(entity.child.another_child_int, dto.5);
}

#[test]
fn unnamed2unnamed() {
    let dto = TupleEntityDto(123, 321, 456, 654, 789, 987);

    let entity: TupleEntity = dto.try_into().unwrap();

    assert_eq!(123, entity.0);
    assert_eq!(321, entity.1 .0 .0);
    assert_eq!(456, entity.1 .0 .1);
    assert_eq!(654, entity.1 .1);
    assert_eq!(789, entity.2 .0);
    assert_eq!(987, entity.2 .1);
}

#[test]
fn unnamed2named() {
    let dto = TupleEntityDto(123, 321, 456, 654, 789, 987);

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(123, entity.parent_int);
    assert_eq!(321, entity.base.base.base_int_2);
    assert_eq!(456, entity.base.base.another_base_int);
    assert_eq!(654, entity.base.base_entity_int);
    assert_eq!(789, entity.child.child_int);
    assert_eq!(987, entity.child.another_child_int);
}

#[test]
fn unnamed2unnamed_reverse() {
    let entity = TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let dto: TupleEntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!(987, dto.5);
}

#[test]
fn unnamed2named_reverse() {
    let entity = TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
}

#[test]
fn unnamed2unnamed_ref() {
    let dto = &TupleEntityDto(123, 321, 456, 654, 789, 987);

    let entity: TupleEntity = dto.try_into().unwrap();

    assert_eq!(dto.0, entity.0);
    assert_eq!(dto.1, entity.1 .0 .0);
    assert_eq!(dto.2, entity.1 .0 .1);
    assert_eq!(dto.3, entity.1 .1);
    assert_eq!(dto.4, entity.2 .0);
    assert_eq!(dto.5, entity.2 .1);
}

#[test]
fn unnamed2named_ref() {
    let dto = &TupleEntityDto(123, 321, 456, 654, 789, 987);

    let entity: Entity = dto.try_into().unwrap();

    assert_eq!(dto.0, entity.parent_int);
    assert_eq!(dto.1, entity.base.base.base_int_2);
    assert_eq!(dto.2, entity.base.base.another_base_int);
    assert_eq!(dto.3, entity.base.base_entity_int);
    assert_eq!(dto.4, entity.child.child_int);
    assert_eq!(dto.5, entity.child.another_child_int);
}

#[test]
fn unnamed2unnamed_reverse_ref() {
    let entity = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let dto: TupleEntityDto = entity.try_into().unwrap();

    assert_eq!(entity.0, dto.0);
    assert_eq!(entity.1 .0 .0, dto.1);
    assert_eq!(entity.1 .0 .1, dto.2);
    assert_eq!(entity.1 .1, dto.3);
    assert_eq!(entity.2 .0, dto.4);
    assert_eq!(entity.2 .1, dto.5);
}

#[test]
fn unnamed2named_reverse_ref() {
    let entity = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let dto: EntityDto = entity.try_into().unwrap();

    assert_eq!(entity.0, dto.parent_int);
    assert_eq!(entity.1 .0 .0, dto.base_int);
    assert_eq!(entity.1 .0 .1, dto.another_base_int);
    assert_eq!(entity.1 .1, dto.base_entity_int);
    assert_eq!(entity.2 .0, dto.child_int);
    assert_eq!(entity.2 .1, dto.another_child_int);
}

#[test]
fn existing_named2named() {
    let entity = Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
        },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let mut dto: EntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
}

#[test]
fn existing_named2unnamed() {
    let entity = Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
        },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let mut dto: TupleEntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!(987, dto.5);
}

#[test]
fn existing_named2named_ref() {
    let entity = &Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
        },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let mut dto: EntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
}

#[test]
fn existing_named2unnamed_ref() {
    let entity = &Entity {
        parent_int: 123,
        base: BaseEntity {
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
            },
            base_entity_int: 654,
        },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let mut dto: TupleEntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!(987, dto.5);
}

#[test]
fn existing_unnamed2unnamed() {
    let entity = TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let mut dto: TupleEntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!(987, dto.5);
}

#[test]
fn existing_unnamed2named() {
    let entity = TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let mut dto: EntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
}

#[test]
fn existing_unnamed2unnamed_ref() {
    let entity = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let mut dto: TupleEntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!(987, dto.5);
}

#[test]
fn existing_unnamed2named_ref() {
    let entity = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let mut dto: EntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!(654, dto.base_entity_int);
    assert_eq!(789, dto.child_int);
    assert_eq!(987, dto.another_child_int);
}