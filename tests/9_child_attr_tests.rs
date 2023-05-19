use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(Default)]
struct Entity {
    parent_int: i32,
    base: BaseEntity,
    child: Child,
}

#[derive(Default)]
struct TupleEntity(i32, TupleBaseEntity, TupleChild);

#[derive(Default)]
struct BaseEntity {
    base: Base,
    base_entity_int: i32,
}

#[derive(Default)]
struct TupleBaseEntity(TupleBase, i32);

#[derive(Default)]
struct Base {
    base_int_2: i32,
    another_base_int: i32,
    some_string: String
}

#[derive(Default)]
struct TupleBase(i32, i16);

#[derive(Default)]
struct Child {
    child_int: i32,
    another_child_int: i32,
}

#[derive(Default)]
struct TupleChild(i32, i16);

#[derive(o2o)]
#[o2o(
    map(Entity),
    into_existing(Entity),
    children(Entity| base: BaseEntity, base.base: Base, child: Child),
)]
struct EntityDto {
    parent_int: i32,
    #[child(base.base)] #[map(base_int_2)] base_int: i32,
    #[child(base.base)] another_base_int: i32,
    #[child(base.base)] #[map_ref(~.clone())] some_string: String,
    #[child(base)] base_entity_int: i32,
    #[child(child)] child_int: i32,
    #[child(child)] another_child_int: i32,
}  

#[derive(o2o)]
#[map(TupleEntity)]
#[into_existing(TupleEntity)]
#[children(1: TupleBaseEntity, 1 .0: TupleBase, 2: TupleChild)]
struct TupleEntityDto (
    i32,
    #[o2o(child(1 .0), map(0))] i32,
    #[o2o(child(1 .0), map(1))] i16,
    #[o2o(child(1), map(1))] i32,
    #[o2o(child(2), map(0))] i32,
    #[o2o(child(2), map(1))] i16,
);

#[test]
fn named2named() {
    let dto = EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        some_string: "Test".into(),
        base_entity_int: 654,
        child_int: 789,
        another_child_int: 987
    };

    let entity: Entity = dto.into();

    assert_eq!(123, entity.parent_int);
    assert_eq!(321, entity.base.base.base_int_2);
    assert_eq!(456, entity.base.base.another_base_int);
    assert_eq!("Test", entity.base.base.some_string);
    assert_eq!(654, entity.base.base_entity_int);
    assert_eq!(789, entity.child.child_int);
    assert_eq!(987, entity.child.another_child_int);
}

#[test]
fn named2named_reverse() {
    let entity = Entity {
        parent_int: 123,
        base: BaseEntity { 
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
                some_string: "Test".into()
            },
            base_entity_int: 654
        },
        child: Child {
            child_int: 789,
            another_child_int: 987
        }
    };

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.parent_int);
    assert_eq!(321, dto.base_int);
    assert_eq!(456, dto.another_base_int);
    assert_eq!("Test", dto.some_string);
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
        some_string: "Test".into(),
        base_entity_int: 654,
        child_int: 789,
        another_child_int: 987
    };

    let entity: Entity = dto.into();

    assert_eq!(dto.parent_int, entity.parent_int);
    assert_eq!(dto.base_int, entity.base.base.base_int_2);
    assert_eq!(dto.another_base_int, entity.base.base.another_base_int);
    assert_eq!("Test", entity.base.base.some_string);
    assert_eq!(dto.base_entity_int, entity.base.base_entity_int);
    assert_eq!(dto.child_int, entity.child.child_int);
    assert_eq!(dto.another_child_int, entity.child.another_child_int);
}

#[test]
fn named2named_reverse_ref() {
    let entity = &Entity {
        parent_int: 123,
        base: BaseEntity { 
            base: Base {
                base_int_2: 321,
                another_base_int: 456,
                some_string: "Test".into(),
            },
            base_entity_int: 654
        },
        child: Child {
            child_int: 789,
            another_child_int: 987
        }
    };

    let dto: EntityDto = entity.into();

    assert_eq!(entity.parent_int, dto.parent_int);
    assert_eq!(entity.base.base.base_int_2, dto.base_int);
    assert_eq!(entity.base.base.another_base_int, dto.another_base_int);
    assert_eq!("Test", dto.some_string);
    assert_eq!(entity.base.base_entity_int, dto.base_entity_int);
    assert_eq!(entity.child.child_int, dto.child_int);
    assert_eq!(entity.child.another_child_int, dto.another_child_int);
}

#[test]
fn unnamed2unnamed() {
    let dto = TupleEntityDto(123, 321, 456, 654, 789, 987);

    let entity: TupleEntity = dto.into();

    assert_eq!(123, entity.0);
    assert_eq!(321, entity.1.0.0);
    assert_eq!(456, entity.1.0.1);
    assert_eq!(654, entity.1.1);
    assert_eq!(789, entity.2.0);
    assert_eq!(987, entity.2.1);
}

#[test]
fn unnamed2unnamed_reverse() {
    let entity = TupleEntity(
        123,
        TupleBaseEntity (
            TupleBase (
                321,
                456
            ),
            654
        ),
        TupleChild (
            789,
            987
        )
    );

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
    assert_eq!(dto.1, entity.1.0.0);
    assert_eq!(dto.2, entity.1.0.1);
    assert_eq!(dto.3, entity.1.1);
    assert_eq!(dto.4, entity.2.0);
    assert_eq!(dto.5, entity.2.1);
}

#[test]
fn unnamed2unnamed_reverse_ref() {
    let entity = &TupleEntity(
        123,
        TupleBaseEntity (
            TupleBase (
                321,
                456
            ),
            654
        ),
        TupleChild (
            789,
            987
        )
    );

    let dto: TupleEntityDto = entity.into();

    assert_eq!(entity.0, dto.0);
    assert_eq!(entity.1.0.0, dto.1);
    assert_eq!(entity.1.0.1, dto.2);
    assert_eq!(entity.1.1, dto.3);
    assert_eq!(entity.2.0, dto.4);
    assert_eq!(entity.2.1, dto.5);
}

#[test]
fn existing_named2named() {
    let dto = EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        some_string: "Test".into(),
        base_entity_int: 654,
        child_int: 789,
        another_child_int: 987
    };

    let mut entity: Entity = Default::default();
    dto.into_existing(&mut entity);

    assert_eq!(123, entity.parent_int);
    assert_eq!(321, entity.base.base.base_int_2);
    assert_eq!(456, entity.base.base.another_base_int);
    assert_eq!("Test", entity.base.base.some_string);
    assert_eq!(654, entity.base.base_entity_int);
    assert_eq!(789, entity.child.child_int);
    assert_eq!(987, entity.child.another_child_int);
}

#[test]
fn existing_named2named_ref() {
    let dto = &EntityDto {
        parent_int: 123,
        base_int: 321,
        another_base_int: 456,
        some_string: "Test".into(),
        base_entity_int: 654,
        child_int: 789,
        another_child_int: 987
    };

    let mut entity: Entity = Default::default();
    dto.into_existing(&mut entity);

    assert_eq!(dto.parent_int, entity.parent_int);
    assert_eq!(dto.base_int, entity.base.base.base_int_2);
    assert_eq!(dto.another_base_int, entity.base.base.another_base_int);
    assert_eq!("Test", entity.base.base.some_string);
    assert_eq!(dto.base_entity_int, entity.base.base_entity_int);
    assert_eq!(dto.child_int, entity.child.child_int);
    assert_eq!(dto.another_child_int, entity.child.another_child_int);
}

#[test]
fn existing_unnamed2unnamed() {
    let dto = TupleEntityDto(123, 321, 456, 654, 789, 987);

    let mut entity: TupleEntity = Default::default();
    dto.into_existing(&mut entity);

    assert_eq!(123, entity.0);
    assert_eq!(321, entity.1.0.0);
    assert_eq!(456, entity.1.0.1);
    assert_eq!(654, entity.1.1);
    assert_eq!(789, entity.2.0);
    assert_eq!(987, entity.2.1);
}

#[test]
fn existing_unnamed2unnamed_ref() {
    let dto = &TupleEntityDto(123, 321, 456, 654, 789, 987);

    let mut entity: TupleEntity = Default::default();
    dto.into_existing(&mut entity);

    assert_eq!(dto.0, entity.0);
    assert_eq!(dto.1, entity.1.0.0);
    assert_eq!(dto.2, entity.1.0.1);
    assert_eq!(dto.3, entity.1.1);
    assert_eq!(dto.4, entity.2.0);
    assert_eq!(dto.5, entity.2.1);
}