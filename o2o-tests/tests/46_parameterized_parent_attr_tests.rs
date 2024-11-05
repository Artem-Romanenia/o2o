use o2o::traits::IntoExisting;

struct Child {
    same_name: i32,
    another_name: i32,
    action: i32,
    action_another_name: i32
}

struct ChildTuple(f32, i16);

#[derive(o2o::o2o)]
#[map(EntityDto)]
#[into_existing(EntityDto)]
struct Entity {
    field: i32,

    #[parent(
        same_name,
        [map(different_name)] another_name,
        [from(~.parse::<i32>().unwrap())] [into(~.to_string())] action,
        [from(action_and_diff_name, ~.parse::<i32>().unwrap())] [into(action_and_diff_name, ~.to_string())] action_another_name
    )]
    child: Child,

    another_field: i32,

    #[parent(EntityDto|
        [from(tuple_str, ~.parse::<f32>().unwrap())] [into(tuple_str, ~.to_string())] 0,
        [map(tuple_field_1)] 1,
    )]
    child_tuple: ChildTuple
}

#[derive(o2o::o2o)]
#[map(EntityDtoTuple)]
#[into_existing(EntityDtoTuple)]
struct EntityTuple(
    #[parent(
        0,
        [into(~.to_string())] [from(~.parse::<i16>().unwrap())] 1
    )]
    ChildTuple,

    #[map(2)] i32,

    #[parent(EntityDtoTuple|
        [map(3)] same_name,
        [map(4)] another_name,
        [from(5, ~.parse::<i32>().unwrap())] [into(5, ~.to_string())] action,
        [from(6, ~.parse::<i32>().unwrap())] [into(6, ~.to_string())] action_another_name
    )]
    Child
);

#[derive(Default)]
struct EntityDto {
    field: i32,

    same_name: i32,
    different_name: i32,
    action: String,
    action_and_diff_name: String,

    another_field: i32,

    tuple_field_1: i16,
    tuple_str: String
}

#[derive(Default)]
struct EntityDtoTuple(f32, String, i32, i32, i32, String, String);

#[test]
fn named2named() {
    let dto = EntityDto {
        field: 123,
        same_name: 321,
        different_name: 456,
        action: "654".into(),
        action_and_diff_name: "789".into(),
        another_field: 987,
        tuple_field_1: 111,
        tuple_str: "333".into()
    };

    let entity: Entity = dto.into();

    assert_eq!(123, entity.field);
    assert_eq!(321, entity.child.same_name);
    assert_eq!(456, entity.child.another_name);
    assert_eq!(654, entity.child.action);
    assert_eq!(789, entity.child.action_another_name);
    assert_eq!(987, entity.another_field);
    assert_eq!(111, entity.child_tuple.1);
    assert_eq!(333.0, entity.child_tuple.0);
}

#[test]
fn named2named_ref() {
    let dto = &EntityDto {
        field: 123,
        same_name: 321,
        different_name: 456,
        action: "654".into(),
        action_and_diff_name: "789".into(),
        another_field: 987,
        tuple_field_1: 111,
        tuple_str: "333.0".into()
    };

    let entity: Entity = dto.into();

    assert_eq!(123, entity.field);
    assert_eq!(321, entity.child.same_name);
    assert_eq!(456, entity.child.another_name);
    assert_eq!(654, entity.child.action);
    assert_eq!(789, entity.child.action_another_name);
    assert_eq!(987, entity.another_field);
    assert_eq!(111, entity.child_tuple.1);
    assert_eq!(333.0, entity.child_tuple.0);
}

#[test]
fn named2named_reverse() {
    let entity = Entity {
        field: 123,
        child: Child {
            same_name: 321,
            another_name: 456,
            action: 654,
            action_another_name: 789,
        },
        another_field: 987,
        child_tuple: ChildTuple(333.0, 111)
    };

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.field);
    assert_eq!(321, dto.same_name);
    assert_eq!(456, dto.different_name);
    assert_eq!("654", dto.action);
    assert_eq!("789", dto.action_and_diff_name);
    assert_eq!(987, dto.another_field);
    assert_eq!(111, dto.tuple_field_1);
    assert_eq!("333", dto.tuple_str);
}

#[test]
fn named2named_reverse_ref() {
    let entity = &Entity {
        field: 123,
        child: Child {
            same_name: 321,
            another_name: 456,
            action: 654,
            action_another_name: 789,
        },
        another_field: 987,
        child_tuple: ChildTuple(333.0, 111)
    };

    let dto: EntityDto = entity.into();

    assert_eq!(123, dto.field);
    assert_eq!(321, dto.same_name);
    assert_eq!(456, dto.different_name);
    assert_eq!("654", dto.action);
    assert_eq!("789", dto.action_and_diff_name);
    assert_eq!(987, dto.another_field);
    assert_eq!(111, dto.tuple_field_1);
    assert_eq!("333", dto.tuple_str);
}

#[test]
fn named2named_reverse_existing() {
    let entity = Entity {
        field: 123,
        child: Child {
            same_name: 321,
            another_name: 456,
            action: 654,
            action_another_name: 789,
        },
        another_field: 987,
        child_tuple: ChildTuple(333.0, 111)
    };

    let mut dto: EntityDto = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(123, dto.field);
    assert_eq!(321, dto.same_name);
    assert_eq!(456, dto.different_name);
    assert_eq!("654", dto.action);
    assert_eq!("789", dto.action_and_diff_name);
    assert_eq!(987, dto.another_field);
    assert_eq!(111, dto.tuple_field_1);
    assert_eq!("333", dto.tuple_str);
}

#[test]
fn named2named_reverse_ref_existing() {
    let entity = &Entity {
        field: 123,
        child: Child {
            same_name: 321,
            another_name: 456,
            action: 654,
            action_another_name: 789,
        },
        another_field: 987,
        child_tuple: ChildTuple(333.0, 111)
    };

    let mut dto: EntityDto = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(123, dto.field);
    assert_eq!(321, dto.same_name);
    assert_eq!(456, dto.different_name);
    assert_eq!("654", dto.action);
    assert_eq!("789", dto.action_and_diff_name);
    assert_eq!(987, dto.another_field);
    assert_eq!(111, dto.tuple_field_1);
    assert_eq!("333", dto.tuple_str);
}

#[test]
fn unnamed2unnamed() {
    let dto = EntityDtoTuple(123.0, "321".into(), 456, 654, 789, "987".into(), "111".into());

    let entity: EntityTuple = dto.into();

    assert_eq!(123.0, entity.0.0);
    assert_eq!(321, entity.0.1);
    assert_eq!(456, entity.1);
    assert_eq!(654, entity.2.same_name);
    assert_eq!(789, entity.2.another_name);
    assert_eq!(987, entity.2.action);
    assert_eq!(111, entity.2.action_another_name);
}

#[test]
fn unnamed2unnamed_ref() {
    let dto = &EntityDtoTuple(123.0, "321".into(), 456, 654, 789, "987".into(), "111".into());

    let entity: EntityTuple = dto.into();

    assert_eq!(123.0, entity.0.0);
    assert_eq!(321, entity.0.1);
    assert_eq!(456, entity.1);
    assert_eq!(654, entity.2.same_name);
    assert_eq!(789, entity.2.another_name);
    assert_eq!(987, entity.2.action);
    assert_eq!(111, entity.2.action_another_name);
}

#[test]
fn unnamed2unnamed_reverse() {
    let entity = EntityTuple(ChildTuple(123.0, 321), 456, Child {
        same_name: 654,
        another_name: 789,
        action: 987,
        action_another_name: 111
    });

    let dto: EntityDtoTuple = entity.into();

    assert_eq!(123.0, dto.0);
    assert_eq!("321", dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!("987", dto.5);
    assert_eq!("111", dto.6);
}

#[test]
fn unnamed2unnamed_reverse_ref() {
    let entity = &EntityTuple(ChildTuple(123.0, 321), 456, Child {
        same_name: 654,
        another_name: 789,
        action: 987,
        action_another_name: 111
    });

    let dto: EntityDtoTuple = entity.into();

    assert_eq!(123.0, dto.0);
    assert_eq!("321", dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!("987", dto.5);
    assert_eq!("111", dto.6);
}

#[test]
fn unnamed2unnamed_reverse_existing() {
    let entity = EntityTuple(ChildTuple(123.0, 321), 456, Child {
        same_name: 654,
        another_name: 789,
        action: 987,
        action_another_name: 111
    });

    let mut dto: EntityDtoTuple = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(123.0, dto.0);
    assert_eq!("321", dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!("987", dto.5);
    assert_eq!("111", dto.6);
}

#[test]
fn unnamed2unnamed_reverse_ref_existing() {
    let entity = &EntityTuple(ChildTuple(123.0, 321), 456, Child {
        same_name: 654,
        another_name: 789,
        action: 987,
        action_another_name: 111
    });

    let mut dto: EntityDtoTuple = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(123.0, dto.0);
    assert_eq!("321", dto.1);
    assert_eq!(456, dto.2);
    assert_eq!(654, dto.3);
    assert_eq!(789, dto.4);
    assert_eq!("987", dto.5);
    assert_eq!("111", dto.6);
}
