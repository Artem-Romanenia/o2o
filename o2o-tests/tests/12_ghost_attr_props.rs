use o2o::o2o;
use o2o::traits::IntoExisting;

#[derive(Default)]
struct EntityModel {
    some_int: i32,
    another_int: i32,
    ghost_int: i32,
    ghost_int_2: i16,
    ghost_float: f32,
}

#[derive(Default)]
struct TupleEntityModel(i32, i16, i32, i16, f32);

#[derive(Default)]
#[derive(o2o)]
#[map(EntityModel)]
#[into_existing(EntityModel)]
#[ghost(ghost_int: |x| { x.some_int }, ghost_int_2: |x| { x.another_int as i16 }, ghost_float: { 456.0 })]
struct Entity {
    some_int: i32,
    another_int: i32,
}

#[derive(o2o)]
#[into(Entity)]
#[into_existing(Entity)]
struct Entity2 {
    some_int: i32,
    another_int: i32,
    #[ghost()]
    _some_float: f32
}

#[derive(Default)]
#[derive(o2o)]
#[map(TupleEntityModel)]
#[into_existing(TupleEntityModel)]
#[o2o(ghost_owned(2: |x| { x.0 }, 3: |x| { x.1 as i16 }, 4: |_| { 456.0 }))]
#[o2o(ghost_ref(2: |x| { x.0 }, 3: |x| { x.1 as i16 }, 4: { 4567.0 }))]
struct TupleEntity (i32, i16);

#[derive(o2o)]
#[map(Entity)]
#[into_existing(Entity)]
struct EntityDto {
    some_int: i32,
    another_int: i32,
    #[o2o(ghost(|x| x.some_int))]
    ghost_int: i32,
    #[o2o(ghost(@.another_int as i16))]
    ghost_int_2: i16,
    #[o2o(ghost_owned(|| 456.0))]
    #[o2o(ghost_ref({4567.0}))]
    ghost_float: f32,
}

#[derive(o2o)]
#[map(TupleEntity)]
#[into_existing(TupleEntity)]
struct TupleEntityDto(
    i32, 
    i16,
    #[ghost(@.0)]
    i32,
    #[ghost(@.1 as i16)]
    i16, 
    #[o2o(ghost_owned({456.0}))]
    #[o2o(ghost_ref(|_| 4567.0))]
    f32
);

#[test]
fn named2named_basic() {
    let named2 = Entity2 {
        some_int: 123,
        another_int: 321,
        _some_float: 456.0
    };

    let named: Entity = named2.into();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
}

#[test]
fn named2named_ref_basic() {
    let named2 = &Entity2 {
        some_int: 123,
        another_int: 321,
        _some_float: 456.0
    };

    let named: Entity = named2.into();

    assert_eq!(named2.some_int, named.some_int);
    assert_eq!(named2.another_int, named.another_int);
}

#[test]
fn named2named() {
    let named = Entity {
        some_int: 123,
        another_int: 321,
    };

    let dto: EntityDto = named.into();

    assert_eq!(123, dto.some_int);
    assert_eq!(321, dto.another_int);
    assert_eq!(123, dto.ghost_int);
    assert_eq!(321, dto.ghost_int_2);
    assert_eq!(456.0, dto.ghost_float);
}

#[test]
fn named2named_2() {
    let named = Entity {
        some_int: 123,
        another_int: 321,
    };

    let model: EntityModel = named.into();

    assert_eq!(123, model.some_int);
    assert_eq!(321, model.another_int);
    assert_eq!(123, model.ghost_int);
    assert_eq!(321, model.ghost_int_2);
    assert_eq!(456.0, model.ghost_float);
}

#[test]
fn named2named_reverse() {
    let dto = EntityDto {
        some_int: 123,
        another_int: 321,
        ghost_int: 456,
        ghost_int_2: 654,
        ghost_float: 789.0
    };

    let named: Entity = dto.into();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
}

#[test]
fn named2named_reverse_2() {
    let model = EntityModel {
        some_int: 123,
        another_int: 321,
        ghost_int: 456,
        ghost_int_2: 654,
        ghost_float: 789.0
    };

    let named: Entity = model.into();

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
}

#[test]
fn named2named_ref() {
    let named = &Entity {
        some_int: 123,
        another_int: 321,
    };

    let dto: EntityDto = named.into();

    assert_eq!(named.some_int, dto.some_int);
    assert_eq!(named.another_int, dto.another_int);
    assert_eq!(named.some_int, dto.ghost_int);
    assert_eq!(named.another_int as i16, dto.ghost_int_2);
    assert_eq!(4567.0, dto.ghost_float);
}

#[test]
fn named2named_ref_2() {
    let named = &Entity {
        some_int: 123,
        another_int: 321,
    };

    let model: EntityModel = named.into();

    assert_eq!(named.some_int, model.some_int);
    assert_eq!(named.another_int, model.another_int);
    assert_eq!(named.some_int, model.ghost_int);
    assert_eq!(named.another_int as i16, model.ghost_int_2);
    assert_eq!(456.0, model.ghost_float);
}

#[test]
fn named2named_reverse_ref() {
    let dto = &EntityDto {
        some_int: 123,
        another_int: 321,
        ghost_int: 456,
        ghost_int_2: 654,
        ghost_float: 789.0
    };

    let named: Entity = dto.into();

    assert_eq!(dto.some_int, named.some_int);
    assert_eq!(dto.another_int as i32, named.another_int);
}

#[test]
fn named2named_reverse_ref_2() {
    let model = &EntityModel {
        some_int: 123,
        another_int: 321,
        ghost_int: 456,
        ghost_int_2: 654,
        ghost_float: 789.0
    };

    let named: Entity = model.into();

    assert_eq!(model.some_int, named.some_int);
    assert_eq!(model.another_int as i32, named.another_int);
}

#[test]
fn unnamed2unnamed() {
    let entity = TupleEntity (
        123,
        321,
    );

    let dto: TupleEntityDto = entity.into();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(123, dto.2);
    assert_eq!(321, dto.3);
    assert_eq!(456.0, dto.4);
}

#[test]
fn unnamed2unnamed_2() {
    let entity = TupleEntity (
        123,
        321,
    );

    let model: TupleEntityModel = entity.into();

    assert_eq!(123, model.0);
    assert_eq!(321, model.1);
    assert_eq!(123, model.2);
    assert_eq!(321, model.3);
    assert_eq!(456.0, model.4);
}

#[test]
fn unnamed2unnamed_reverse() {
    let dto = TupleEntityDto (
        123,
        321,
        456,
        654,
        789.0
    );

    let entity: TupleEntity = dto.into();

    assert_eq!(123, entity.0);
    assert_eq!(321, entity.1);
}

#[test]
fn unnamed2unnamed_reverse_2() {
    let model = TupleEntityModel (
        123,
        321,
        456,
        654,
        789.0
    );

    let entity: TupleEntity = model.into();

    assert_eq!(123, entity.0);
    assert_eq!(321, entity.1);
}

#[test]
fn unnamed2unnamed_ref() {
    let entity = &TupleEntity (
        123,
        321,
    );

    let dto: TupleEntityDto = entity.into();

    assert_eq!(entity.0, dto.0);
    assert_eq!(entity.1, dto.1);
    assert_eq!(entity.0, dto.2);
    assert_eq!(entity.1 as i16, dto.3);
    assert_eq!(4567.0, dto.4);
}

#[test]
fn unnamed2unnamed_ref_2() {
    let entity = &TupleEntity (
        123,
        321,
    );

    let model: TupleEntityModel = entity.into();

    assert_eq!(entity.0, model.0);
    assert_eq!(entity.1, model.1);
    assert_eq!(entity.0, model.2);
    assert_eq!(entity.1 as i16, model.3);
    assert_eq!(4567.0, model.4);
}

#[test]
fn unnamed2unnamed_reverse_ref() {
    let dto = &TupleEntityDto (
        123,
        321,
        456,
        654,
        789.0
    );

    let entity: TupleEntity = dto.into();

    assert_eq!(dto.0, entity.0);
    assert_eq!(dto.1, entity.1);
}

#[test]
fn unnamed2unnamed_reverse_ref_2() {
    let model = &TupleEntityModel (
        123,
        321,
        456,
        654,
        789.0
    );

    let entity: TupleEntity = model.into();

    assert_eq!(model.0, entity.0);
    assert_eq!(model.1, entity.1);
}

#[test]
fn existing_named2named_basic() {
    let named2 = Entity2 {
        some_int: 123,
        another_int: 321,
        _some_float: 456.0
    };

    let mut named: Entity = Default::default();
    named2.into_existing(&mut named);

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
}

#[test]
fn existing_named2named_ref_basic() {
    let named2 = &Entity2 {
        some_int: 123,
        another_int: 321,
        _some_float: 456.0
    };

    let mut named: Entity = Default::default();
    named2.into_existing(&mut named);

    assert_eq!(named2.some_int, named.some_int);
    assert_eq!(named2.another_int, named.another_int);
}

#[test]
fn existing_named2named() {
    let named = Entity {
        some_int: 123,
        another_int: 321,
    };

    let mut model: EntityModel = Default::default();
    named.into_existing(&mut model);

    assert_eq!(123, model.some_int);
    assert_eq!(321, model.another_int);
    assert_eq!(123, model.ghost_int);
    assert_eq!(321, model.ghost_int_2);
    assert_eq!(456.0, model.ghost_float);
}

#[test]
fn existing_named2named_ref() {
    let named = &Entity {
        some_int: 123,
        another_int: 321,
    };

    let mut model: EntityModel = Default::default();
    named.into_existing(&mut model);

    assert_eq!(named.some_int, model.some_int);
    assert_eq!(named.another_int, model.another_int);
    assert_eq!(named.some_int, model.ghost_int);
    assert_eq!(named.another_int as i16, model.ghost_int_2);
    assert_eq!(456.0, model.ghost_float);
}

#[test]
fn existing_named2named_reverse() {
    let dto = EntityDto {
        some_int: 123,
        another_int: 321,
        ghost_int: 456,
        ghost_int_2: 654,
        ghost_float: 789.0
    };

    let mut named: Entity = Default::default();
    dto.into_existing(&mut named);

    assert_eq!(123, named.some_int);
    assert_eq!(321, named.another_int);
}

#[test]
fn existing_named2named_reverse_ref() {
    let dto = &EntityDto {
        some_int: 123,
        another_int: 321,
        ghost_int: 456,
        ghost_int_2: 654,
        ghost_float: 789.0
    };

    let mut named: Entity = Default::default();
    dto.into_existing(&mut named);

    assert_eq!(dto.some_int, named.some_int);
    assert_eq!(dto.another_int as i32, named.another_int);
}

#[test]
fn existing_unnamed2unnamed() {
    let named = TupleEntity (
        123,
        321,
    );

    let mut model: TupleEntityModel = Default::default();
    named.into_existing(&mut model);

    assert_eq!(123, model.0);
    assert_eq!(321, model.1);
    assert_eq!(123, model.2);
    assert_eq!(321, model.3);
    assert_eq!(456.0, model.4);
}

#[test]
fn existing_unnamed2unnamed_ref() {
    let entity = &TupleEntity (
        123,
        321,
    );

    let mut model: TupleEntityModel = Default::default();
    entity.into_existing(&mut model);

    assert_eq!(entity.0, model.0);
    assert_eq!(entity.1, model.1);
    assert_eq!(entity.0, model.2);
    assert_eq!(entity.1 as i16, model.3);
    assert_eq!(4567.0, model.4);
}

#[test]
fn existing_unnamed2unnamed_reverse() {
    let dto = TupleEntityDto (
        123,
        321,
        456,
        654,
        789.0
    );

    let mut entity: TupleEntity = Default::default();
    dto.into_existing(&mut entity);

    assert_eq!(123, entity.0);
    assert_eq!(321, entity.1);
}

#[test]
fn existing_unnamed2unnamed_reverse_ref() {
    let dto = &TupleEntityDto (
        123,
        321,
        456,
        654,
        789.0
    );

    let mut entity: TupleEntity = Default::default();
    dto.into_existing(&mut entity);

    assert_eq!(dto.0, entity.0);
    assert_eq!(dto.1, entity.1);
}