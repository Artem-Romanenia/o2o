use o2o::traits::IntoExisting;

struct Child {
    same_name: i32,
    another_name: i32,
    action: i32,
    action_another_name: i32
}

struct ChildTuple(f32, i16);

#[derive(o2o::o2o)]
#[map(EntityDtoTuple as ())]
#[into_existing(EntityDtoTuple as ())]
struct Entity {
    field: i32,

    #[parent(
        [map(1)] same_name,
        [map(2)] another_name,
        [from(3, ~.parse::<i32>().unwrap())] [into(3, ~.to_string())] action,
        [from(4, ~.parse::<i32>().unwrap())] [into(4, ~.to_string())] action_another_name
    )]
    child: Child,

    #[map(5)] another_field: i32,

    #[parent(EntityDtoTuple|
        [map(6)] 0,
        [from(7, ~.parse::<i16>().unwrap())] [into(7, ~.to_string())] 1,
    )]
    child_tuple: ChildTuple
}

#[derive(o2o::o2o)]
#[map(EntityDto as {})]
#[into_existing(EntityDto as {})]
struct EntityTuple(
    #[parent(
        [map(tuple_field_1)] 0,
        [into(tuple_str, ~.to_string())] [from(tuple_str, ~.parse::<i16>().unwrap())] 1
    )]
    ChildTuple,

    #[map(another_field)] i32,

    #[parent(EntityDto|
        same_name,
        [map(different_name)] another_name,
        [from(~.parse::<i32>().unwrap())] [into(~.to_string())] action,
        [from(action_and_diff_name, ~.parse::<i32>().unwrap())] [into(action_and_diff_name, ~.to_string())] action_another_name
    )]
    Child
);

#[derive(Default)]
struct EntityDto {
    same_name: i32,
    different_name: i32,
    action: String,
    action_and_diff_name: String,

    another_field: i32,

    tuple_field_1: f32,
    tuple_str: String
}

#[derive(Default)]
struct EntityDtoTuple(i32, i32, i32, String, String, i32, f32, String);

#[test]
fn named2named() {
    let dto = EntityDtoTuple (123, 321, 456, "654".into(), "789".into(), 987, 111.0, "333".into());

    let entity: Entity = dto.into();

    assert_eq!(123, entity.field);
    assert_eq!(321, entity.child.same_name);
    assert_eq!(456, entity.child.another_name);
    assert_eq!(654, entity.child.action);
    assert_eq!(789, entity.child.action_another_name);
    assert_eq!(987, entity.another_field);
    assert_eq!(111.0, entity.child_tuple.0);
    assert_eq!(333, entity.child_tuple.1);
}

#[test]
fn named2named_ref() {
    let dto = &EntityDtoTuple (123, 321, 456, "654".into(), "789".into(), 987, 111.0, "333".into());

    let entity: Entity = dto.into();

    assert_eq!(123, entity.field);
    assert_eq!(321, entity.child.same_name);
    assert_eq!(456, entity.child.another_name);
    assert_eq!(654, entity.child.action);
    assert_eq!(789, entity.child.action_another_name);
    assert_eq!(987, entity.another_field);
    assert_eq!(111.0, entity.child_tuple.0);
    assert_eq!(333, entity.child_tuple.1);
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
        child_tuple: ChildTuple(111.0, 333)
    };

    let dto: EntityDtoTuple = entity.into();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!("654", dto.3);
    assert_eq!("789", dto.4);
    assert_eq!(987, dto.5);
    assert_eq!(111.0, dto.6);
    assert_eq!("333", dto.7);
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
        child_tuple: ChildTuple(111.0, 333)
    };

    let dto: EntityDtoTuple = entity.into();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!("654", dto.3);
    assert_eq!("789", dto.4);
    assert_eq!(987, dto.5);
    assert_eq!(111.0, dto.6);
    assert_eq!("333", dto.7);
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
        child_tuple: ChildTuple(111.0, 333)
    };

    let mut dto: EntityDtoTuple = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!("654", dto.3);
    assert_eq!("789", dto.4);
    assert_eq!(987, dto.5);
    assert_eq!(111.0, dto.6);
    assert_eq!("333", dto.7);
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
        child_tuple: ChildTuple(111.0, 333)
    };

    let mut dto: EntityDtoTuple = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!("654", dto.3);
    assert_eq!("789", dto.4);
    assert_eq!(987, dto.5);
    assert_eq!(111.0, dto.6);
    assert_eq!("333", dto.7);
}

#[test]
fn unnamed2unnamed() {
    let dto = EntityDto {
        tuple_field_1: 123.0,
        tuple_str: "321".into(),
        another_field: 456,
        same_name: 654,
        different_name: 789,
        action: "987".into(),
        action_and_diff_name: "111".into()
    };

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
    let dto = &EntityDto {
        tuple_field_1: 123.0,
        tuple_str: "321".into(),
        another_field: 456,
        same_name: 654,
        different_name: 789,
        action: "987".into(),
        action_and_diff_name: "111".into()
    };

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
    let entity = Entity {
        field: 123,
        child: Child { same_name: 321, another_name: 456, action: 654, action_another_name: 789 },
        another_field: 987,
        child_tuple: ChildTuple(333.0, 111),
    };

    let dto: EntityDtoTuple = entity.into();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!("654", dto.3);
    assert_eq!("789", dto.4);
    assert_eq!(987, dto.5);
    assert_eq!(333.0, dto.6);
    assert_eq!("111", dto.7);
}

#[test]
fn unnamed2unnamed_reverse_ref() {
    let entity = &Entity {
        field: 123,
        child: Child { same_name: 321, another_name: 456, action: 654, action_another_name: 789 },
        another_field: 987,
        child_tuple: ChildTuple(333.0, 111),
    };

    let dto: EntityDtoTuple = entity.into();

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!("654", dto.3);
    assert_eq!("789", dto.4);
    assert_eq!(987, dto.5);
    assert_eq!(333.0, dto.6);
    assert_eq!("111", dto.7);
}

#[test]
fn unnamed2unnamed_reverse_existing() {
    let entity = Entity {
        field: 123,
        child: Child { same_name: 321, another_name: 456, action: 654, action_another_name: 789 },
        another_field: 987,
        child_tuple: ChildTuple(333.0, 111),
    };

    let mut dto: EntityDtoTuple = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!("654", dto.3);
    assert_eq!("789", dto.4);
    assert_eq!(987, dto.5);
    assert_eq!(333.0, dto.6);
    assert_eq!("111", dto.7);
}

#[test]
fn unnamed2unnamed_reverse_ref_existing() {
    let entity = &Entity {
        field: 123,
        child: Child { same_name: 321, another_name: 456, action: 654, action_another_name: 789 },
        another_field: 987,
        child_tuple: ChildTuple(333.0, 111),
    };

    let mut dto: EntityDtoTuple = Default::default();
    entity.into_existing(&mut dto);

    assert_eq!(123, dto.0);
    assert_eq!(321, dto.1);
    assert_eq!(456, dto.2);
    assert_eq!("654", dto.3);
    assert_eq!("789", dto.4);
    assert_eq!(987, dto.5);
    assert_eq!(333.0, dto.6);
    assert_eq!("111", dto.7);
}