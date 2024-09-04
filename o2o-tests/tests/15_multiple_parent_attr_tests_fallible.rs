use o2o::o2o;
use o2o::traits::TryIntoExisting;

#[derive(o2o)]
#[o2o(try_map(EntityDto, String), try_map(TupleEntityDto, String), try_map(EntityModel, String), try_into_existing(EntityDto, String), try_into_existing(TupleEntityDto, String))]
struct Entity {
    #[map(TupleEntityDto| 0)]
    parent_int: i32,
    #[parent(EntityDto)]
    #[parent(TupleEntityDto)]
    #[map_ref(EntityModel| ~.clone())]
    base: BaseEntity,
    #[parent(EntityDto)]
    #[parent(TupleEntityDto)]
    #[map_ref(EntityModel| ~.clone())]
    child: Child,
}

struct EntityModel {
    parent_int: i32,
    base: BaseEntity,
    child: Child,
}

#[derive(Clone, o2o)]
#[try_from(EntityDto, String)]
#[try_from(TupleEntityDto, String)]
#[try_into_existing(EntityDto, String)]
#[try_into_existing(TupleEntityDto, String)]
struct BaseEntity {
    #[parent]
    base: Base,
    #[map(TupleEntityDto| 3)]
    base_entity_int: i32,
}

#[derive(Clone, o2o)]
#[try_from(EntityDto, String)]
#[try_from(TupleEntityDto, String)]
#[try_into_existing(EntityDto, String)]
#[try_into_existing(TupleEntityDto, String)]
struct Base {
    #[map(EntityDto| base_int)]
    #[map(TupleEntityDto| 1)]
    base_int_2: i32,
    #[from(TupleEntityDto| @.2 as i32)]
    #[into(TupleEntityDto| 2, ~ as i16)]
    another_base_int: i32,
}

#[derive(Clone, o2o)]
#[try_from(EntityDto, String)]
#[try_from(TupleEntityDto, String)]
#[try_into_existing(EntityDto, String)]
#[try_into_existing(TupleEntityDto, String)]
struct Child {
    #[map(TupleEntityDto| 4)]
    child_int: i32,
    #[from(TupleEntityDto| @.5 as i32)]
    #[into(TupleEntityDto| 5, ~ as i16)]
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
#[try_map(TupleEntityDto, String)]
#[try_map(EntityDto as {}, String)]
#[try_into_existing(TupleEntityDto, String)]
#[try_into_existing(EntityDto, String)]
struct TupleEntity(#[map(EntityDto| parent_int)] i32, #[parent] TupleBaseEntity, #[parent] TupleChild);

#[derive(o2o)]
#[try_from(TupleEntityDto, String)]
#[try_from(EntityDto, String)]
#[try_into_existing(TupleEntityDto, String)]
#[try_into_existing(EntityDto, String)]
struct TupleBaseEntity(
    #[parent] TupleBase,
    #[map(TupleEntityDto| 3)]
    #[map(EntityDto| base_entity_int)]
    i32,
);

#[derive(o2o)]
#[try_from(TupleEntityDto, String)]
#[try_from(EntityDto, String)]
#[try_into_existing(TupleEntityDto, String)]
#[try_into_existing(EntityDto, String)]
struct TupleBase(
    #[map(TupleEntityDto| 1)]
    #[map(EntityDto| base_int)]
    i32,
    #[map(TupleEntityDto| 2)]
    #[from(EntityDto| @.another_base_int as i16)]
    #[into(EntityDto| another_base_int, ~ as i32)]
    i16,
);

#[derive(o2o)]
#[try_from(TupleEntityDto, String)]
#[try_from(EntityDto, String)]
#[try_into_existing(TupleEntityDto, String)]
#[try_into_existing(EntityDto, String)]
struct TupleChild(
    #[map(TupleEntityDto| 4)]
    #[map(EntityDto| child_int)]
    i32,
    #[map(TupleEntityDto| 5)]
    #[from(EntityDto| @.another_child_int as i16)]
    #[into(EntityDto| another_child_int, ~ as i32)]
    i16,
);

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
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
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
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
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
    assert_eq!(dto.another_base_int as i16, entity.1 .0 .1);
    assert_eq!(dto.base_entity_int, entity.1 .1);
    assert_eq!(dto.child_int, entity.2 .0);
    assert_eq!(dto.another_child_int as i16, entity.2 .1);
}

#[test]
fn named2named_reverse_ref() {
    let entity = &Entity {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
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
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let dto: TupleEntityDto = entity.try_into().unwrap();

    assert_eq!(entity.parent_int, dto.0);
    assert_eq!(entity.base.base.base_int_2, dto.1);
    assert_eq!(entity.base.base.another_base_int as i16, dto.2);
    assert_eq!(entity.base.base_entity_int, dto.3);
    assert_eq!(entity.child.child_int, dto.4);
    assert_eq!(entity.child.another_child_int as i16, dto.5);
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
    assert_eq!(dto.2 as i32, entity.base.base.another_base_int);
    assert_eq!(dto.3, entity.base.base_entity_int);
    assert_eq!(dto.4, entity.child.child_int);
    assert_eq!(dto.5 as i32, entity.child.another_child_int);
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
    assert_eq!(entity.1 .0 .1 as i32, dto.another_base_int);
    assert_eq!(entity.1 .1, dto.base_entity_int);
    assert_eq!(entity.2 .0, dto.child_int);
    assert_eq!(entity.2 .1 as i32, dto.another_child_int);
}

#[test]
fn named2named_2() {
    let entity = Entity {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let model: EntityModel = entity.try_into().unwrap();

    assert_eq!(123, model.parent_int);
    assert_eq!(321, model.base.base.base_int_2);
    assert_eq!(456, model.base.base.another_base_int);
    assert_eq!(654, model.base.base_entity_int);
    assert_eq!(789, model.child.child_int);
    assert_eq!(987, model.child.another_child_int);
}

#[test]
fn named2named_2_ref() {
    let entity = &Entity {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let model: EntityModel = entity.try_into().unwrap();

    assert_eq!(entity.parent_int, model.parent_int);
    assert_eq!(entity.base.base.base_int_2, model.base.base.base_int_2);
    assert_eq!(entity.base.base.another_base_int, model.base.base.another_base_int);
    assert_eq!(entity.base.base_entity_int, model.base.base_entity_int);
    assert_eq!(entity.child.child_int, model.child.child_int);
    assert_eq!(entity.child.another_child_int, model.child.another_child_int);
}

#[test]
fn named2named_2_reverse() {
    let model = EntityModel {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let entity: Entity = model.try_into().unwrap();

    assert_eq!(123, entity.parent_int);
    assert_eq!(321, entity.base.base.base_int_2);
    assert_eq!(456, entity.base.base.another_base_int);
    assert_eq!(654, entity.base.base_entity_int);
    assert_eq!(789, entity.child.child_int);
    assert_eq!(987, entity.child.another_child_int);
}

#[test]
fn named2named_2_ref_reverse() {
    let model = &EntityModel {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let entity: Entity = model.try_into().unwrap();

    assert_eq!(model.parent_int, entity.parent_int);
    assert_eq!(model.base.base.base_int_2, entity.base.base.base_int_2);
    assert_eq!(model.base.base.another_base_int, entity.base.base.another_base_int);
    assert_eq!(model.base.base_entity_int, entity.base.base_entity_int);
    assert_eq!(model.child.child_int, entity.child.child_int);
    assert_eq!(model.child.another_child_int, entity.child.another_child_int);
}

#[test]
fn existing_named2named() {
    let entity = Entity {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
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
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
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
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let mut dto: EntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(entity.parent_int, dto.parent_int);
    assert_eq!(entity.base.base.base_int_2, dto.base_int);
    assert_eq!(entity.base.base.another_base_int, dto.another_base_int);
    assert_eq!(entity.base.base_entity_int, dto.base_entity_int);
    assert_eq!(entity.child.child_int, dto.child_int);
    assert_eq!(entity.child.another_child_int, dto.another_child_int);
}

#[test]
fn existing_named2unnamed_ref() {
    let entity = &Entity {
        parent_int: 123,
        base: BaseEntity { base: Base { base_int_2: 321, another_base_int: 456 }, base_entity_int: 654 },
        child: Child { child_int: 789, another_child_int: 987 },
    };

    let mut dto: TupleEntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(entity.parent_int, dto.0);
    assert_eq!(entity.base.base.base_int_2, dto.1);
    assert_eq!(entity.base.base.another_base_int as i16, dto.2);
    assert_eq!(entity.base.base_entity_int, dto.3);
    assert_eq!(entity.child.child_int, dto.4);
    assert_eq!(entity.child.another_child_int as i16, dto.5);
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

    assert_eq!(entity.0, dto.0);
    assert_eq!(entity.1 .0 .0, dto.1);
    assert_eq!(entity.1 .0 .1, dto.2);
    assert_eq!(entity.1 .1, dto.3);
    assert_eq!(entity.2 .0, dto.4);
    assert_eq!(entity.2 .1, dto.5);
}

#[test]
fn existing_unnamed2named_ref() {
    let entity = &TupleEntity(123, TupleBaseEntity(TupleBase(321, 456), 654), TupleChild(789, 987));

    let mut dto: EntityDto = Default::default();
    entity.try_into_existing(&mut dto).unwrap();

    assert_eq!(entity.0, dto.parent_int);
    assert_eq!(entity.1 .0 .0, dto.base_int);
    assert_eq!(entity.1 .0 .1 as i32, dto.another_base_int);
    assert_eq!(entity.1 .1, dto.base_entity_int);
    assert_eq!(entity.2 .0, dto.child_int);
    assert_eq!(entity.2 .1 as i32, dto.another_child_int);
}
